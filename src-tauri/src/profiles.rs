use crate::{
    error::{AppError, Result},
    paths::AppPaths,
};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub username: String,
    pub minecraft_version: String,
    #[serde(default)]
    pub loader: Loader,
    #[serde(default)]
    pub fabric_loader_version: Option<String>,
    #[serde(default)]
    pub preset: Preset,
    #[serde(default = "default_memory_mb")]
    pub memory_mb: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub last_played_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInput {
    pub id: Option<String>,
    pub name: String,
    pub username: String,
    pub minecraft_version: String,
    #[serde(default)]
    pub loader: Loader,
    #[serde(default)]
    pub fabric_loader_version: Option<String>,
    #[serde(default)]
    pub preset: Preset,
    #[serde(default = "default_memory_mb")]
    pub memory_mb: u32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Loader {
    #[default]
    Vanilla,
    Fabric,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Preset {
    #[default]
    Vanilla,
    Performance,
    Visuals,
    Custom,
}

fn default_memory_mb() -> u32 {
    2048
}

fn file(paths: &AppPaths) -> std::path::PathBuf {
    paths.profiles.join("profiles.json")
}

pub fn list(paths: &AppPaths) -> Result<Vec<Profile>> {
    let path = file(paths);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(path)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

pub fn save(paths: &AppPaths, input: ProfileInput) -> Result<Profile> {
    validate(&input)?;
    let mut profiles = list(paths)?;
    let now = Utc::now();
    let profile = if let Some(id) = input.id {
        let existing = profiles
            .iter()
            .find(|profile| profile.id == id)
            .ok_or_else(|| AppError::new("profile_not_found", "That profile no longer exists."))?;
        Profile {
            id,
            created_at: existing.created_at,
            updated_at: now,
            name: input.name.trim().into(),
            username: input.username,
            minecraft_version: input.minecraft_version,
            loader: input.loader,
            fabric_loader_version: input.fabric_loader_version,
            preset: input.preset,
            memory_mb: input.memory_mb,
            last_played_at: existing.last_played_at,
        }
    } else {
        Profile {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            name: input.name.trim().into(),
            username: input.username,
            minecraft_version: input.minecraft_version,
            loader: input.loader,
            fabric_loader_version: input.fabric_loader_version,
            preset: input.preset,
            memory_mb: input.memory_mb,
            last_played_at: None,
        }
    };
    profiles.retain(|item| item.id != profile.id);
    profiles.push(profile.clone());
    write(paths, &profiles)?;
    fs::create_dir_all(paths.instance_game(&profile.id))?;
    Ok(profile)
}

pub fn delete(paths: &AppPaths, id: &str) -> Result<()> {
    Uuid::parse_str(id).map_err(|_| AppError::new("invalid_profile", "Profile ID is invalid."))?;
    let mut profiles = list(paths)?;
    let original_len = profiles.len();
    profiles.retain(|profile| profile.id != id);
    if profiles.len() == original_len {
        return Err(AppError::new(
            "profile_not_found",
            "That profile no longer exists.",
        ));
    }
    write(paths, &profiles)?;
    let instance = paths.instance(id);
    if instance.exists() {
        fs::remove_dir_all(instance)?;
    }
    Ok(())
}

pub fn duplicate(paths: &AppPaths, id: &str) -> Result<Profile> {
    let source = find(paths, id)?;
    save(
        paths,
        ProfileInput {
            id: None,
            name: format!("{} Copy", source.name).chars().take(40).collect(),
            username: source.username,
            minecraft_version: source.minecraft_version,
            loader: source.loader,
            fabric_loader_version: source.fabric_loader_version,
            preset: source.preset,
            memory_mb: source.memory_mb,
        },
    )
}

pub fn mark_played(paths: &AppPaths, id: &str) -> Result<()> {
    let mut items = list(paths)?;
    let profile = items
        .iter_mut()
        .find(|profile| profile.id == id)
        .ok_or_else(|| AppError::new("profile_not_found", "That profile no longer exists."))?;
    profile.last_played_at = Some(Utc::now());
    write(paths, &items)
}

pub fn find(paths: &AppPaths, id: &str) -> Result<Profile> {
    list(paths)?
        .into_iter()
        .find(|profile| profile.id == id)
        .ok_or_else(|| {
            AppError::new(
                "profile_not_found",
                "Select an existing profile before launching.",
            )
        })
}

fn validate(input: &ProfileInput) -> Result<()> {
    let username = Regex::new(r"^[A-Za-z0-9_]{3,16}$").expect("static regex");
    if input.name.trim().is_empty() || input.name.chars().count() > 40 {
        return Err(AppError::new(
            "invalid_profile",
            "Profile name must be between 1 and 40 characters.",
        ));
    }
    if !username.is_match(&input.username) {
        return Err(AppError::new(
            "invalid_username",
            "Username must be 3–16 characters using letters, numbers, or underscore.",
        ));
    }
    if input.minecraft_version.trim().is_empty() || input.minecraft_version.chars().count() > 80 {
        return Err(AppError::new(
            "invalid_version",
            "Choose a Minecraft version from Mojang's version catalog.",
        ));
    }
    if !(512..=32768).contains(&input.memory_mb) {
        return Err(AppError::new(
            "invalid_memory",
            "Profile memory must be between 512 MB and 32 GB.",
        ));
    }
    if input.loader == Loader::Fabric && input.fabric_loader_version.is_none() {
        return Err(AppError::new(
            "fabric_version_required",
            "Choose a compatible Fabric Loader version.",
        ));
    }
    if input.loader == Loader::Vanilla && input.preset != Preset::Vanilla {
        return Err(AppError::new(
            "fabric_preset_required",
            "Performance, visuals, and custom mod presets require Fabric.",
        ));
    }
    Ok(())
}

fn write(paths: &AppPaths, profiles: &[Profile]) -> Result<()> {
    fs::create_dir_all(&paths.profiles)?;
    let target = file(paths);
    let temporary = paths.profiles.join("profiles.json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(profiles)?)?;
    if target.exists() {
        fs::remove_file(&target)?;
    }
    fs::rename(temporary, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_round_trip_and_instance_creation() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path());
        paths.ensure().unwrap();
        let saved = save(
            &paths,
            ProfileInput {
                id: None,
                name: "Test".into(),
                username: "Player_1".into(),
                minecraft_version: crate::DEFAULT_VERSION.into(),
                loader: Loader::Vanilla,
                fabric_loader_version: None,
                preset: Preset::Vanilla,
                memory_mb: 2048,
            },
        )
        .unwrap();
        assert_eq!(list(&paths).unwrap()[0].username, "Player_1");
        assert!(paths.instance_game(&saved.id).is_dir());
    }

    #[test]
    fn invalid_username_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path());
        paths.ensure().unwrap();
        let result = save(
            &paths,
            ProfileInput {
                id: None,
                name: "Test".into(),
                username: "no spaces".into(),
                minecraft_version: crate::DEFAULT_VERSION.into(),
                loader: Loader::Vanilla,
                fabric_loader_version: None,
                preset: Preset::Vanilla,
                memory_mb: 2048,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_gets_a_new_isolated_instance() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path());
        paths.ensure().unwrap();
        let original = save(
            &paths,
            ProfileInput {
                id: None,
                name: "Original".into(),
                username: "Player_1".into(),
                minecraft_version: crate::DEFAULT_VERSION.into(),
                loader: Loader::Vanilla,
                fabric_loader_version: None,
                preset: Preset::Vanilla,
                memory_mb: 3072,
            },
        )
        .unwrap();
        let copy = duplicate(&paths, &original.id).unwrap();
        assert_ne!(original.id, copy.id);
        assert_ne!(
            paths.instance_game(&original.id),
            paths.instance_game(&copy.id)
        );
        assert_eq!(copy.memory_mb, 3072);
    }
}
