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
        return serde_json::from_slice(&tokio::fs::read(&path).await?).map_err(Into::into);
    }
    let client = reqwest::Client::builder()
        .user_agent(concat!("Flint/", env!("CARGO_PKG_VERSION")))
        .build()?;
    match client.get(VERSION_MANIFEST_URL).send().await {
        Ok(response) => {
            let bytes = response.error_for_status()?.bytes().await?;
            let manifest: VersionManifest = serde_json::from_slice(&bytes)?;
            tokio::fs::create_dir_all(&paths.metadata).await?;
            tokio::fs::write(&path, &bytes).await?;
            Ok(manifest)
        }
        Err(error) if path.exists() => {
            tracing::warn!(%error, "using stale Mojang version manifest cache");
            serde_json::from_slice(&tokio::fs::read(path).await?).map_err(Into::into)
        }
        Err(error) => Err(AppError::from(error)),
    }
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
}
