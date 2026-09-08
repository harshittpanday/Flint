use crate::{
    error::{AppError, Result},
    paths::AppPaths,
};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LauncherSettings {
    pub automatic_java: bool,
    pub manual_java_path: Option<PathBuf>,
    pub default_memory_mb: u32,
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub show_snapshots: bool,
    pub discord_rich_presence: bool,
    pub behavior_while_running: RunningBehavior,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RunningBehavior {
    #[default]
    KeepOpen,
    Minimize,
    Hide,
}

impl Default for LauncherSettings {
    fn default() -> Self {
        Self {
            automatic_java: true,
            manual_java_path: None,
            default_memory_mb: 2048,
            resolution_width: 1280,
            resolution_height: 720,
            show_snapshots: false,
            discord_rich_presence: true,
            behavior_while_running: RunningBehavior::KeepOpen,
        }
    }
}

fn file(paths: &AppPaths) -> PathBuf {
    paths.settings.join("settings.json")
}

pub fn load(paths: &AppPaths) -> Result<LauncherSettings> {
    let path = file(paths);
    if !path.exists() {
        return Ok(LauncherSettings::default());
    }
    let settings: LauncherSettings = serde_json::from_slice(&fs::read(path)?)?;
    validate(&settings)?;
    Ok(settings)
}

pub fn save(paths: &AppPaths, settings: LauncherSettings) -> Result<LauncherSettings> {
    validate(&settings)?;
    fs::create_dir_all(&paths.settings)?;
    let target = file(paths);
    let temporary = paths.settings.join("settings.json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&settings)?)?;
    if target.exists() {
        fs::remove_file(&target)?;
    }
    fs::rename(temporary, target)?;
    Ok(settings)
}

fn validate(settings: &LauncherSettings) -> Result<()> {
    if !(512..=32768).contains(&settings.default_memory_mb) {
        return Err(AppError::new(
            "invalid_memory",
            "Default memory must be between 512 MB and 32 GB.",
        ));
    }
    if !(640..=7680).contains(&settings.resolution_width)
        || !(480..=4320).contains(&settings.resolution_height)
    {
        return Err(AppError::new(
            "invalid_resolution",
            "Minecraft resolution must be between 640×480 and 7680×4320.",
        ));
    }
    if !settings.automatic_java && settings.manual_java_path.is_none() {
        return Err(AppError::new(
            "manual_java_required",
            "Choose a Java executable or enable automatic Java selection.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path());
        paths.ensure().unwrap();
        let mut settings = LauncherSettings::default();
        settings.show_snapshots = true;
        save(&paths, settings).unwrap();
        assert!(load(&paths).unwrap().show_snapshots);
    }
}
