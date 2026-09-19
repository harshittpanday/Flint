use crate::error::{AppError, Result};
use futures::{lock::Mutex as AsyncMutex, StreamExt};
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock, Weak},
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const MAX_ATTEMPTS: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 150;

type DestinationLock = AsyncMutex<()>;
static DESTINATION_LOCKS: OnceLock<Mutex<HashMap<PathBuf, Weak<DestinationLock>>>> =
    OnceLock::new();

pub fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(120))
}

pub async fn ensure(
    client: &reqwest::Client,
    artifact: &str,
    url: &str,
    expected_sha1: &str,
    expected_size: u64,
    path: &Path,
) -> Result<bool> {
    let lock = destination_lock(path);
    let _guard = lock.lock().await;

    let mut validation_attempt = 0;
    let cache_state = loop {
        validation_attempt += 1;
        match validate_file(path, expected_sha1, expected_size).await {
            Ok(state) => break state,
            Err(error)
                if validation_attempt < MAX_ATTEMPTS
                    && retryable_io(&error, "validate cached file") =>
            {
                tracing::warn!(
                    artifact,
                    target = %path.display(),
                    attempt = validation_attempt,
                    %error,
                    "cached artifact was temporarily unavailable"
                );
                tokio::time::sleep(Duration::from_millis(
                    RETRY_BASE_DELAY_MS * u64::from(validation_attempt),
                ))
                .await;
            }
            Err(error) => {
                return Err(artifact_io_error(
                    artifact,
                    url,
                    path,
                    "validate cached file",
                    error,
                ))
            }
        }
    };
    match cache_state {
        CacheState::Valid => {
            tracing::debug!(artifact, target = %path.display(), "reusing verified cached artifact");
            return Ok(false);
        }
        CacheState::Invalid {
            actual_sha1,
            actual_size,
        } => tracing::warn!(
            artifact,
            target = %path.display(),
            expected_sha1,
            actual_sha1,
            expected_size,
            actual_size,
            "cached artifact is invalid; downloading a replacement"
        ),
        CacheState::Missing => {}
    }

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|error| {
            artifact_io_error(artifact, url, path, "create destination directory", error)
        })?;
    }

    let mut last_error = None;
    let mut attempts_used = 0;
    for attempt in 1..=MAX_ATTEMPTS {
        attempts_used = attempt;
        match download_once(client, artifact, url, expected_sha1, expected_size, path).await {
            Ok(()) => return Ok(true),
            Err(failure) => {
                tracing::warn!(
                    artifact,
                    url = %safe_url(url),
                    target = %path.display(),
                    attempt,
                    max_attempts = MAX_ATTEMPTS,
                    retryable = failure.retryable,
                    code = failure.error.code,
                    detail = ?failure.error.detail,
                    "download attempt failed"
                );
                let retryable = failure.retryable;
                last_error = Some(failure.error);
                if !retryable || attempt == MAX_ATTEMPTS {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(
                    RETRY_BASE_DELAY_MS * u64::from(attempt),
                ))
                .await;
            }
        }
    }

    let mut error = last_error.expect("the download attempt loop always runs");
    error.detail = Some(match error.detail.take() {
        Some(detail) => format!("{detail}; attempts: {attempts_used}/{MAX_ATTEMPTS}"),
        None => format!("attempts: {attempts_used}/{MAX_ATTEMPTS}"),
    });
    Err(error)
}

pub async fn store_verified_bytes(
    artifact: &str,
    url: &str,
    path: &Path,
    bytes: &[u8],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|error| {
            artifact_io_error(artifact, url, path, "create destination directory", error)
        })?;
    }
    let temporary = temporary_path(path);
    let result = async {
        let mut file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)
            .await?;
        file.write_all(bytes).await?;
        file.flush().await?;
        file.sync_all().await?;
        drop(file);
        promote_file(&temporary, path)
    }
    .await;
    if let Err(error) = result {
        let _ = tokio::fs::remove_file(&temporary).await;
        return Err(artifact_io_error(
            artifact,
            url,
            path,
            "store validated metadata",
            error,
        ));
    }
    Ok(())
}

async fn download_once(
    client: &reqwest::Client,
    artifact: &str,
    url: &str,
    expected_sha1: &str,
    expected_size: u64,
    path: &Path,
) -> std::result::Result<(), AttemptFailure> {
    tracing::info!(artifact, url = %safe_url(url), target = %path.display(), "downloading artifact");
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| network_failure(artifact, url, path, error))?;
    let status = response.status();
    let final_url = safe_url(response.url().as_str());
    if !status.is_success() {
        return Err(AttemptFailure::new(
            AppError::new("download_http_failed", download_message(artifact)).with_detail(format!(
                "artifact: {artifact}; url: {final_url}; HTTP status: {status}; destination: {}",
                path.display()
            )),
            is_transient_status(status),
        ));
    }

    let temporary = temporary_path(path);
    let result = write_and_promote(
        response,
        artifact,
        url,
        expected_sha1,
        expected_size,
        path,
        &temporary,
    )
    .await;
    if result.is_err() {
        if let Err(cleanup_error) = tokio::fs::remove_file(&temporary).await {
            if cleanup_error.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    artifact,
                    temporary = %temporary.display(),
                    %cleanup_error,
                    "could not remove failed temporary download"
                );
            }
        }
    }
    result
}

async fn write_and_promote(
    response: reqwest::Response,
    artifact: &str,
    url: &str,
    expected_sha1: &str,
    expected_size: u64,
    path: &Path,
    temporary: &Path,
) -> std::result::Result<(), AttemptFailure> {
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)
        .await
        .map_err(|error| io_failure(artifact, url, path, "create temporary file", error))?;
    let mut stream = response.bytes_stream();
    let mut hasher = Sha1::new();
    let mut received = 0_u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| network_failure(artifact, url, path, error))?;
        received = received.saturating_add(chunk.len() as u64);
        if expected_size != 0 && received > expected_size {
            return Err(integrity_failure(
                artifact,
                url,
                path,
                "download_size_mismatch",
                format!("expected {expected_size} bytes, received more than {expected_size}"),
            ));
        }
        hasher.update(&chunk);
        file.write_all(&chunk)
            .await
            .map_err(|error| io_failure(artifact, url, path, "write temporary file", error))?;
    }
    file.flush()
        .await
        .map_err(|error| io_failure(artifact, url, path, "flush temporary file", error))?;
    file.sync_all()
        .await
        .map_err(|error| io_failure(artifact, url, path, "sync temporary file", error))?;
    drop(file);

    let on_disk_size = tokio::fs::metadata(temporary)
        .await
        .map_err(|error| io_failure(artifact, url, path, "inspect temporary file", error))?
        .len();
    if on_disk_size != received || (expected_size != 0 && received != expected_size) {
        let expected = if expected_size == 0 {
            "unspecified".to_string()
        } else {
            expected_size.to_string()
        };
        return Err(integrity_failure(
            artifact,
            url,
            path,
            "download_size_mismatch",
            format!(
                "expected size {expected}, received {received} bytes, temporary file contains {on_disk_size} bytes"
            ),
        ));
    }
    let actual_sha1 = hex::encode(hasher.finalize());
    if !actual_sha1.eq_ignore_ascii_case(expected_sha1) {
        return Err(integrity_failure(
            artifact,
            url,
            path,
            "download_hash_mismatch",
            format!("expected SHA-1 {expected_sha1}, received {actual_sha1}"),
        ));
    }

    promote_file(temporary, path)
        .map_err(|error| io_failure(artifact, url, path, "promote temporary file", error))?;
    Ok(())
}

fn destination_lock(path: &Path) -> Arc<DestinationLock> {
    let locks = DESTINATION_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut locks = locks
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    locks.retain(|_, lock| lock.strong_count() > 0);
    if let Some(lock) = locks.get(path).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(DestinationLock::new(()));
    locks.insert(path.to_path_buf(), Arc::downgrade(&lock));
    lock
}

#[derive(Debug, PartialEq, Eq)]
enum CacheState {
    Missing,
    Valid,
    Invalid {
        actual_sha1: String,
        actual_size: u64,
    },
}

async fn validate_file(
    path: &Path,
    expected_sha1: &str,
    expected_size: u64,
) -> std::io::Result<CacheState> {
    let metadata = match tokio::fs::metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CacheState::Missing)
        }
        Err(error) => return Err(error.into()),
    };
    let actual_size = metadata.len();
    let actual_sha1 = sha1_file(path).await?;
    if (expected_size == 0 || actual_size == expected_size)
        && actual_sha1.eq_ignore_ascii_case(expected_sha1)
    {
        Ok(CacheState::Valid)
    } else {
        Ok(CacheState::Invalid {
            actual_sha1,
            actual_size,
        })
    }
}

async fn sha1_file(path: &Path) -> std::io::Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let mut hasher = Sha1::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn temporary_path(path: &Path) -> PathBuf {
    let filename = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    path.with_file_name(format!(
        ".{filename}.flint-download-{}",
        uuid::Uuid::new_v4()
    ))
}

#[cfg(windows)]
fn promote_file(temporary: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source = temporary
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let target = destination
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let succeeded = unsafe {
        MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if succeeded == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn promote_file(temporary: &Path, destination: &Path) -> std::io::Result<()> {
    std::fs::rename(temporary, destination)
}

fn network_failure(
    artifact: &str,
    url: &str,
    path: &Path,
    error: reqwest::Error,
) -> AttemptFailure {
    let retryable = !error.is_builder()
        && !error.is_redirect()
        && (error.is_timeout()
            || error.is_connect()
            || error.is_body()
            || error.is_decode()
            || error.is_request());
    AttemptFailure::new(
        AppError::new("download_network_failed", download_message(artifact)).with_detail(format!(
            "artifact: {artifact}; url: {}; destination: {}; network error: {error}",
            safe_url(url),
            path.display()
        )),
        retryable,
    )
}

fn integrity_failure(
    artifact: &str,
    url: &str,
    path: &Path,
    code: &'static str,
    reason: String,
) -> AttemptFailure {
    AttemptFailure::new(
        AppError::new(code, download_message(artifact)).with_detail(format!(
            "artifact: {artifact}; url: {}; destination: {}; integrity error: {reason}",
            safe_url(url),
            path.display()
        )),
        true,
    )
}

fn io_failure(
    artifact: &str,
    url: &str,
    path: &Path,
    operation: &str,
    error: std::io::Error,
) -> AttemptFailure {
    let retryable = retryable_io(&error, operation);
    AttemptFailure::new(
        artifact_io_error(artifact, url, path, operation, error),
        retryable,
    )
}

fn retryable_io(error: &std::io::Error, operation: &str) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::Interrupted
            | std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::Other
    ) || (error.kind() == std::io::ErrorKind::PermissionDenied
        && matches!(operation, "validate cached file" | "promote temporary file"))
}

fn artifact_io_error(
    artifact: &str,
    url: &str,
    path: &Path,
    operation: &str,
    error: std::io::Error,
) -> AppError {
    AppError::new("download_io_failed", download_message(artifact)).with_detail(format!(
        "artifact: {artifact}; url: {}; destination: {}; filesystem operation: {operation}; error: {error}",
        safe_url(url),
        path.display()
    ))
}

fn download_message(artifact: &str) -> String {
    format!(
        "Failed to download a required Minecraft file. Artifact: {artifact}. Retry the launch. See launcher logs for details."
    )
}

fn safe_url(url: &str) -> String {
    reqwest::Url::parse(url)
        .map(|mut parsed| {
            parsed.set_query(None);
            parsed.set_fragment(None);
            let _ = parsed.set_username("");
            let _ = parsed.set_password(None);
            parsed.to_string()
        })
        .unwrap_or_else(|_| "<invalid URL>".to_string())
}

fn is_transient_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 425 | 429 | 500..=599)
}

struct AttemptFailure {
    error: AppError,
    retryable: bool,
}

impl AttemptFailure {
    fn new(error: AppError, retryable: bool) -> Self {
        Self { error, retryable }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::{Shutdown, TcpListener},
        sync::{
            atomic::{AtomicBool, AtomicUsize, Ordering},
            Arc,
        },
        thread,
    };

    struct TestServer {
        url: String,
        requests: Arc<AtomicUsize>,
        stop: Arc<AtomicBool>,
        thread: Option<thread::JoinHandle<()>>,
    }

    impl TestServer {
        fn new(responses: Vec<(u16, Vec<u8>, Option<usize>)>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let url = format!("http://{}/artifact", listener.local_addr().unwrap());
            let requests = Arc::new(AtomicUsize::new(0));
            let stop = Arc::new(AtomicBool::new(false));
            let request_count = requests.clone();
            let stop_signal = stop.clone();
            let thread = thread::spawn(move || {
                while !stop_signal.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let mut request = [0_u8; 2048];
                            let _ = stream.read(&mut request);
                            let index = request_count.fetch_add(1, Ordering::SeqCst);
                            let (status, body, advertised) = responses
                                .get(index)
                                .or_else(|| responses.last())
                                .expect("at least one response");
                            let reason = if *status == 200 { "OK" } else { "Error" };
                            let length = advertised.unwrap_or(body.len());
                            let headers = format!(
                                "HTTP/1.0 {status} {reason}\r\nContent-Length: {length}\r\nConnection: close\r\n\r\n"
                            );
                            let _ = stream.write_all(headers.as_bytes());
                            let _ = stream.write_all(body);
                            let _ = stream.flush();
                            let _ = stream.shutdown(Shutdown::Write);
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                        }
                        Err(_) => break,
                    }
                }
            });
            Self {
                url,
                requests,
                stop,
                thread: Some(thread),
            }
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    fn sha1(bytes: &[u8]) -> String {
        hex::encode(Sha1::digest(bytes))
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .pool_max_idle_per_host(0)
            .build()
            .unwrap()
    }

    fn temporary_files(directory: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(directory)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.to_string_lossy().contains(".flint-download-"))
            .collect()
    }

    #[tokio::test]
    async fn clean_install_downloads_into_missing_directories() {
        let body = b"minecraft";
        let server = TestServer::new(vec![(200, body.to_vec(), None)]);
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("new directory").join("client.jar");
        assert!(ensure(
            &client(),
            "client JAR",
            &server.url,
            &sha1(body),
            body.len() as u64,
            &target
        )
        .await
        .unwrap());
        assert_eq!(std::fs::read(target).unwrap(), body);
        assert_eq!(server.requests.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn valid_cached_artifact_is_reused_without_network() {
        let body = b"cached";
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("cached.jar");
        std::fs::write(&target, body).unwrap();
        assert!(!ensure(
            &client(),
            "cached library",
            "http://127.0.0.1:1/nope",
            &sha1(body),
            body.len() as u64,
            &target
        )
        .await
        .unwrap());
    }

    #[tokio::test]
    async fn corrupt_and_zero_byte_cached_artifacts_are_replaced() {
        for cached in [b"corrupt".as_slice(), b"".as_slice()] {
            let body = b"valid";
            let server = TestServer::new(vec![(200, body.to_vec(), None)]);
            let root = tempfile::tempdir().unwrap();
            let target = root.path().join("library.jar");
            std::fs::write(&target, cached).unwrap();
            assert!(ensure(
                &client(),
                "library test:artifact:1",
                &server.url,
                &sha1(body),
                body.len() as u64,
                &target
            )
            .await
            .unwrap());
            assert_eq!(std::fs::read(target).unwrap(), body);
        }
    }

    #[tokio::test]
    async fn interrupted_download_leaves_no_final_or_temporary_artifact() {
        let body = b"cut";
        let server = TestServer::new(vec![(200, body.to_vec(), Some(20))]);
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("asset");
        let error = ensure(
            &client(),
            "asset abc",
            &server.url,
            &sha1(b"complete"),
            8,
            &target,
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "download_network_failed");
        assert!(!target.exists());
        assert!(temporary_files(root.path()).is_empty());
    }

    #[tokio::test]
    async fn checksum_mismatch_is_rejected_and_temporary_file_is_removed() {
        let server = TestServer::new(vec![(200, b"wrong".to_vec(), None)]);
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("client.jar");
        let error = ensure(
            &client(),
            "client JAR",
            &server.url,
            &sha1(b"right"),
            5,
            &target,
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "download_hash_mismatch");
        assert!(!target.exists());
        assert!(temporary_files(root.path()).is_empty());
    }

    #[test]
    fn failed_promotion_preserves_existing_destination() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("client.jar");
        std::fs::write(&destination, b"existing").unwrap();
        assert!(promote_file(&root.path().join("missing.tmp"), &destination).is_err());
        assert_eq!(std::fs::read(destination).unwrap(), b"existing");
    }

    #[tokio::test]
    async fn permanent_http_failure_is_not_retried_and_is_actionable() {
        let server = TestServer::new(vec![(404, b"missing".to_vec(), None)]);
        let root = tempfile::tempdir().unwrap();
        let error = ensure(
            &client(),
            "library example:test:1",
            &server.url,
            &sha1(b"x"),
            1,
            &root.path().join("test.jar"),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, "download_http_failed");
        assert!(error.message.contains("library example:test:1"));
        assert!(error.detail.as_deref().unwrap().contains("404"));
        assert_eq!(server.requests.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn transient_http_failure_uses_bounded_retries_then_succeeds() {
        let body = b"success";
        let server = TestServer::new(vec![(503, Vec::new(), None), (200, body.to_vec(), None)]);
        let root = tempfile::tempdir().unwrap();
        assert!(ensure(
            &client(),
            "asset retry",
            &server.url,
            &sha1(body),
            body.len() as u64,
            &root.path().join("asset")
        )
        .await
        .unwrap());
        assert!((2..=MAX_ATTEMPTS as usize).contains(&server.requests.load(Ordering::SeqCst)));
    }

    #[tokio::test]
    async fn concurrent_requests_for_same_artifact_are_serialized_without_corruption() {
        let body = b"shared";
        let server = TestServer::new(vec![(200, body.to_vec(), None)]);
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("shared.jar");
        let client = client();
        let expected_sha1 = sha1(body);
        let first = ensure(
            &client,
            "shared library",
            &server.url,
            &expected_sha1,
            body.len() as u64,
            &target,
        );
        let second = ensure(
            &client,
            "shared library",
            &server.url,
            &expected_sha1,
            body.len() as u64,
            &target,
        );
        let (first, second) = tokio::join!(first, second);
        assert_ne!(first.unwrap(), second.unwrap());
        assert_eq!(std::fs::read(target).unwrap(), body);
    }

    #[tokio::test]
    async fn unicode_and_space_destination_is_supported() {
        let body = b"portable";
        let server = TestServer::new(vec![(200, body.to_vec(), None)]);
        let root = tempfile::tempdir().unwrap();
        let target = root.path().join("User Name 测试").join("asset file");
        ensure(
            &client(),
            "unicode asset",
            &server.url,
            &sha1(body),
            body.len() as u64,
            &target,
        )
        .await
        .unwrap();
        assert_eq!(std::fs::read(target).unwrap(), body);
    }
}
