use super::download;
use crate::{
    error::{AppError, Result},
    paths::AppPaths,
    profiles::{Loader, Preset, Profile},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

const API: &str = "https://api.modrinth.com/v2";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"))]
pub struct ModProject {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    hits: Vec<ModProject>,
}

#[derive(Debug, Deserialize)]
struct ProjectDetail {
    id: String,
    slug: String,
    title: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ModVersion {
    id: String,
    project_id: String,
    name: String,
    version_number: String,
    #[serde(default)]
    dependencies: Vec<Dependency>,
    files: Vec<ModFile>,
}

#[derive(Clone, Debug, Deserialize)]
struct Dependency {
    version_id: Option<String>,
    project_id: Option<String>,
    dependency_type: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ModFile {
    hashes: Hashes,
    url: String,
    filename: String,
    primary: bool,
    size: u64,
}

#[derive(Clone, Debug, Deserialize)]
struct Hashes {
    sha1: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub version_number: String,
    pub filename: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetMod {
    pub project_id: String,
    pub slug: String,
    pub title: String,
    pub version_number: String,
}

pub async fn search(query: &str, game_version: &str) -> Result<Vec<ModProject>> {
    let facets = serde_json::to_string(&vec![
        vec!["project_type:mod".to_string()],
        vec!["categories:fabric".to_string()],
        vec![format!("versions:{game_version}")],
    ])?;
    let response: SearchResponse = client()?
        .get(format!("{API}/search"))
        .query(&[("query", query), ("facets", &facets), ("limit", "20")])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(response.hits)
}

pub async fn preview_preset(game_version: &str, preset: &Preset) -> Result<Vec<PresetMod>> {
    let slugs: &[&str] = match preset {
        Preset::Vanilla | Preset::Custom => &[],
        Preset::Performance => &["sodium", "lithium", "entityculling"],
        Preset::Visuals => &["iris"],
    };
    let mut result = Vec::new();
    for slug in slugs {
        let project = get_project(slug).await?;
        let version = compatible_version(slug, game_version).await?;
        result.push(PresetMod {
            project_id: project.id,
            slug: project.slug,
            title: project.title,
            version_number: version.version_number,
        });
    }
    Ok(result)
}

pub async fn apply_preset(paths: &AppPaths, profile: &Profile) -> Result<Vec<InstalledMod>> {
    let preview = preview_preset(&profile.minecraft_version, &profile.preset).await?;
    for item in preview {
        install(paths, profile, &item.project_id).await?;
    }
    list_installed(paths, &profile.id)
}

pub async fn install(
    paths: &AppPaths,
    profile: &Profile,
    project_id: &str,
) -> Result<Vec<InstalledMod>> {
    ensure_fabric(profile)?;
    let mut queue = vec![compatible_version(project_id, &profile.minecraft_version).await?];
    let mut visited = std::collections::HashSet::new();
    while let Some(version) = queue.pop() {
        if !visited.insert(version.project_id.clone()) {
            continue;
        }
        for dependency in &version.dependencies {
            if dependency.dependency_type != "required" {
                continue;
            }
            let required = if let Some(version_id) = &dependency.version_id {
                get_version(version_id).await?
            } else if let Some(project_id) = &dependency.project_id {
                compatible_version(project_id, &profile.minecraft_version).await?
            } else {
                continue;
            };
            queue.push(required);
        }
        install_version(paths, profile, version).await?;
    }
    list_installed(paths, &profile.id)
}

pub fn list_installed(paths: &AppPaths, profile_id: &str) -> Result<Vec<InstalledMod>> {
    let path = manifest_path(paths, profile_id);
    if !path.exists() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&std::fs::read(path)?).map_err(Into::into)
}

pub fn remove(paths: &AppPaths, profile_id: &str, project_id: &str) -> Result<Vec<InstalledMod>> {
    let mut installed = list_installed(paths, profile_id)?;
    let removed = installed
        .iter()
        .filter(|item| item.project_id == project_id)
        .cloned()
        .collect::<Vec<_>>();
    if removed.is_empty() {
        return Err(AppError::new(
            "mod_not_found",
            "That managed mod is not installed.",
        ));
    }
    for item in removed {
        validate_filename(&item.filename)?;
        let target = paths
            .instance_game(profile_id)
            .join("mods")
            .join(item.filename);
        if target.exists() {
            std::fs::remove_file(target)?;
        }
    }
    installed.retain(|item| item.project_id != project_id);
    write_manifest(paths, profile_id, &installed)?;
    Ok(installed)
}

async fn compatible_version(project_id: &str, game_version: &str) -> Result<ModVersion> {
    let loaders = serde_json::to_string(&["fabric"])?;
    let game_versions = serde_json::to_string(&[game_version])?;
    let versions: Vec<ModVersion> = client()?
        .get(format!("{API}/project/{project_id}/version"))
        .query(&[
            ("loaders", loaders.as_str()),
            ("game_versions", game_versions.as_str()),
            ("include_changelog", "false"),
        ])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    versions.into_iter().next().ok_or_else(|| {
        AppError::new(
            "mod_incompatible",
            format!("No Fabric version of this mod supports Minecraft {game_version}."),
        )
    })
}

async fn get_project(id: &str) -> Result<ProjectDetail> {
    client()?
        .get(format!("{API}/project/{id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .map_err(Into::into)
}

async fn get_version(id: &str) -> Result<ModVersion> {
    client()?
        .get(format!("{API}/version/{id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .map_err(Into::into)
}

async fn install_version(paths: &AppPaths, profile: &Profile, version: ModVersion) -> Result<()> {
    let file = version
        .files
        .iter()
        .find(|file| file.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            AppError::new(
                "mod_file_missing",
                "This Modrinth version has no download file.",
            )
        })?;
    validate_filename(&file.filename)?;
    let target = paths
        .instance_game(&profile.id)
        .join("mods")
        .join(&file.filename);
    download::ensure(&client()?, &file.url, &file.hashes.sha1, file.size, &target).await?;
    let mut installed = list_installed(paths, &profile.id)?;
    if let Some(previous) = installed
        .iter()
        .find(|item| item.project_id == version.project_id)
    {
        if previous.filename != file.filename {
            validate_filename(&previous.filename)?;
            let old = paths
                .instance_game(&profile.id)
                .join("mods")
                .join(&previous.filename);
            if old.exists() {
                std::fs::remove_file(old)?;
            }
        }
    }
    installed.retain(|item| item.project_id != version.project_id);
    installed.push(InstalledMod {
        project_id: version.project_id,
        version_id: version.id,
        name: version.name,
        version_number: version.version_number,
        filename: file.filename.clone(),
    });
    write_manifest(paths, &profile.id, &installed)
}

fn ensure_fabric(profile: &Profile) -> Result<()> {
    if profile.loader != Loader::Fabric {
        return Err(AppError::new(
            "fabric_profile_required",
            "Mods can only be installed into a Fabric profile.",
        ));
    }
    Ok(())
}

fn validate_filename(filename: &str) -> Result<()> {
    if Path::new(filename)
        .file_name()
        .and_then(|name| name.to_str())
        != Some(filename)
        || !filename.to_ascii_lowercase().ends_with(".jar")
    {
        return Err(AppError::new(
            "invalid_mod_file",
            "Modrinth returned an unsafe mod filename.",
        ));
    }
    Ok(())
}

fn manifest_path(paths: &AppPaths, profile_id: &str) -> std::path::PathBuf {
    paths.instance(profile_id).join("managed-mods.json")
}

fn write_manifest(paths: &AppPaths, profile_id: &str, installed: &[InstalledMod]) -> Result<()> {
    let target = manifest_path(paths, profile_id);
    let parent = target
        .parent()
        .ok_or_else(|| AppError::new("invalid_path", "Invalid instance path."))?;
    std::fs::create_dir_all(parent)?;
    let temporary = parent.join("managed-mods.json.tmp");
    std::fs::write(&temporary, serde_json::to_vec_pretty(installed)?)?;
    if target.exists() {
        std::fs::remove_file(&target)?;
    }
    std::fs::rename(temporary, target)?;
    Ok(())
}

fn client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(concat!(
            "Flint/",
            env!("CARGO_PKG_VERSION"),
            " (desktop launcher)"
        ))
        .build()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_mod_files() {
        assert!(validate_filename("sodium.jar").is_ok());
        assert!(validate_filename("../sodium.jar").is_err());
        assert!(validate_filename("readme.txt").is_err());
    }

    #[test]
    fn managed_mods_are_profile_specific() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path());
        paths.ensure().unwrap();
        write_manifest(
            &paths,
            "one",
            &[InstalledMod {
                project_id: "a".into(),
                version_id: "v".into(),
                name: "A".into(),
                version_number: "1".into(),
                filename: "a.jar".into(),
            }],
        )
        .unwrap();
        assert_eq!(list_installed(&paths, "one").unwrap().len(), 1);
        assert!(list_installed(&paths, "two").unwrap().is_empty());
    }
}
