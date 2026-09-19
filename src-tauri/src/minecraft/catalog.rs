use super::metadata::{VersionManifest, VERSION_MANIFEST_URL};
use crate::{
    error::{AppError, Result},
    paths::AppPaths,
};
use serde::Serialize;
use std::time::{Duration, SystemTime};

const MANIFEST_MAX_AGE: Duration = Duration::from_secs(60 * 60);

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftVersion {
    pub id: String,
    pub version_type: String,
    pub release_time: String,
}

pub async fn list(paths: &AppPaths, include_snapshots: bool) -> Result<Vec<MinecraftVersion>> {
    let manifest = load_manifest(paths).await?;
    Ok(manifest
        .versions
        .into_iter()
        .filter(|item| is_visible(&item.version_type, include_snapshots))
        .map(|item| MinecraftVersion {
            id: item.id,
            version_type: item.version_type,
            release_time: item.release_time,
        })
        .collect())
}

fn is_visible(version_type: &str, include_snapshots: bool) -> bool {
    version_type == "release" || (include_snapshots && version_type == "snapshot")
}

pub async fn load_manifest(paths: &AppPaths) -> Result<VersionManifest> {
    let path = paths.metadata.join("version_manifest_v2.json");
    if cache_is_fresh(&path) {
        match read_manifest(&path).await {
            Ok(manifest) => return Ok(manifest),
            Err(error) => tracing::warn!(
                code = error.code,
                detail = ?error.detail,
                target = %path.display(),
                "fresh Mojang version manifest cache is invalid; refreshing it"
            ),
        }
    }
    let client = super::download::client_builder()
        .user_agent(concat!("Flint/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let fetched = async {
        let response = client
            .get(VERSION_MANIFEST_URL)
            .send()
            .await?
            .error_for_status()?;
        let bytes = response.bytes().await?;
        let manifest: VersionManifest = serde_json::from_slice(&bytes)?;
        super::download::store_verified_bytes(
            "Mojang version manifest",
            VERSION_MANIFEST_URL,
            &path,
            &bytes,
        )
        .await?;
        Ok::<VersionManifest, AppError>(manifest)
    }
    .await;
    match fetched {
        Ok(manifest) => Ok(manifest),
        Err(network_error) if path.exists() => match read_manifest(&path).await {
            Ok(manifest) => {
                tracing::warn!(
                    code = network_error.code,
                    detail = ?network_error.detail,
                    "using validated stale Mojang version manifest cache"
                );
                Ok(manifest)
            }
            Err(cache_error) => Err(AppError::new(
                "version_manifest_unavailable",
                "Flint could not download or read Mojang's version manifest.",
            )
            .with_detail(format!(
                "download error: {}; cache error: {}",
                network_error
                    .detail
                    .as_deref()
                    .unwrap_or(&network_error.message),
                cache_error
                    .detail
                    .as_deref()
                    .unwrap_or(&cache_error.message)
            ))),
        },
        Err(error) => Err(error),
    }
}

async fn read_manifest(path: &std::path::Path) -> Result<VersionManifest> {
    serde_json::from_slice(&tokio::fs::read(path).await?).map_err(Into::into)
}

fn cache_is_fresh(path: &std::path::Path) -> bool {
    path.metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age <= MANIFEST_MAX_AGE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_filter_excludes_snapshots() {
        let json = r#"{"versions":[{"id":"26.2","type":"release","url":"https://example/release","sha1":"a","releaseTime":"2026-01-01"},{"id":"26w01a","type":"snapshot","url":"https://example/snapshot","sha1":"b","releaseTime":"2026-01-02"}]}"#;
        let manifest: VersionManifest = serde_json::from_str(json).unwrap();
        let releases = manifest
            .versions
            .iter()
            .filter(|item| item.version_type == "release")
            .collect::<Vec<_>>();
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].id, "26.2");
        assert!(is_visible("snapshot", true));
        assert!(!is_visible("snapshot", false));
        assert!(!is_visible("old_alpha", true));
    }

    #[tokio::test]
    async fn malformed_fresh_cache_is_not_trusted() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("manifest.json");
        std::fs::write(&path, b"truncated").unwrap();
        assert!(cache_is_fresh(&path));
        assert_eq!(
            read_manifest(&path).await.unwrap_err().code,
            "invalid_metadata"
        );
    }
}
