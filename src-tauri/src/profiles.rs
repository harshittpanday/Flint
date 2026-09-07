use crate::{
    error::{AppError, Result},
    paths::AppPaths,
    SUPPORTED_VERSION,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileInput {
    pub id: Option<String>,
    pub name: String,
    pub username: String,
    pub minecraft_version: String,
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
        }
    } else {
        Profile {
            id: Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,
            name: input.name.trim().into(),
            username: input.username,
            minecraft_version: input.minecraft_version,
        }
    };
    profiles.retain(|item| item.id != profile.id);
    profiles.push(profile.clone());
    write(paths, &profiles)?;
    fs::create_dir_all(paths.instance_game(&profile.id))?;
    Ok(profile)
}

pub fn delete(paths: &AppPaths, id: &str) -> Result<()> {
    let mut profiles = list(paths)?;
    let original_len = profiles.len();
    profiles.retain(|profile| profile.id != id);
    if profiles.len() == original_len {
        return Err(AppError::new(
            "profile_not_found",
            "That profile no longer exists.",
        ));
    }
    write(paths, &profiles)
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
    if input.minecraft_version != SUPPORTED_VERSION {
        return Err(AppError::new(
            "unsupported_version",
            format!("Flint Milestone 1 supports Minecraft {SUPPORTED_VERSION} only."),
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
                minecraft_version: SUPPORTED_VERSION.into(),
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
                minecraft_version: SUPPORTED_VERSION.into(),
            },
        );
        assert!(result.is_err());
    }
}
