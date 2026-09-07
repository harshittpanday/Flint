use crate::error::{AppError, Result};
use sha1::{Digest, Sha1};
use std::path::Path;

pub async fn ensure(
    client: &reqwest::Client,
    url: &str,
    expected_sha1: &str,
    expected_size: u64,
    path: &Path,
) -> Result<bool> {
    if file_is_valid(path, expected_sha1, expected_size).await? {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tracing::info!(url, target = %path.display(), "downloading file");
    let response = client.get(url).send().await?.error_for_status()?;
    let bytes = response.bytes().await?;
    if expected_size != 0 && bytes.len() as u64 != expected_size {
        return Err(AppError::new(
            "download_size_mismatch",
            "A Minecraft download was incomplete.",
        )
        .with_detail(format!(
            "{}: expected {expected_size} bytes, received {}",
            path.display(),
            bytes.len()
        )));
    }
    let actual = hex::encode(Sha1::digest(&bytes));
    if !actual.eq_ignore_ascii_case(expected_sha1) {
        return Err(AppError::new(
            "download_hash_mismatch",
            "A Minecraft download failed integrity verification.",
        )
        .with_detail(format!(
            "{}: expected {expected_sha1}, received {actual}",
            path.display()
        )));
    }
    let temporary = path.with_extension("flint-download");
    tokio::fs::write(&temporary, &bytes).await?;
    if path.exists() {
        tokio::fs::remove_file(path).await?;
    }
    tokio::fs::rename(temporary, path).await?;
    Ok(true)
}

async fn file_is_valid(path: &Path, expected_sha1: &str, expected_size: u64) -> Result<bool> {
    let Ok(metadata) = tokio::fs::metadata(path).await else {
        return Ok(false);
    };
    if expected_size != 0 && metadata.len() != expected_size {
        return Ok(false);
    }
    let bytes = tokio::fs::read(path).await?;
    Ok(hex::encode(Sha1::digest(bytes)).eq_ignore_ascii_case(expected_sha1))
}
