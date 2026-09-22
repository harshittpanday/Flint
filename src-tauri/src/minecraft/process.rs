use super::{
    arguments::LaunchArguments,
    install::{emit, PreparedVersion},
};
use crate::{
    error::{AppError, Result},
    java::JavaInfo,
    paths::AppPaths,
    profiles::Profile,
    LauncherState,
};
use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{self, OpenOptions},
    path::Path,
    process::Stdio,
    sync::{atomic::Ordering, Arc},
};
use tauri::AppHandle;

pub fn launch(
    app: AppHandle,
    state: Arc<LauncherState>,
    paths: &AppPaths,
    profile: &Profile,
    java: &JavaInfo,
    prepared: PreparedVersion,
    arguments: LaunchArguments,
    autoauth_session: Option<crate::autoauth::AutoAuthSession>,
) -> Result<()> {
    let game_dir = paths.instance_game(&profile.id);
    std::fs::create_dir_all(&game_dir)?;
    let crash_dir = game_dir.join("crash-reports");
    let existing_crashes = crash_report_names(&crash_dir);
    let game_log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(paths.logs.join("minecraft.log"))?;
    let error_log = game_log.try_clone()?;
    let mut command = crate::process_command::tokio_command(&java.path);
    command
        .args(&arguments.jvm)
        .arg(&prepared.metadata.main_class)
        .args(&arguments.game)
        .current_dir(&game_dir)
        .stdout(Stdio::from(game_log))
        .stderr(Stdio::from(error_log));
    if let Some(session) = &autoauth_session {
        session.configure(&mut command);
    }
    tracing::info!(java = %java.path.display(), main_class = %prepared.metadata.main_class, game_dir = %game_dir.display(), "starting Minecraft process");
    let mut child = command.spawn().map_err(|error| {
        AppError::new(
            "process_start_failed",
            "Java was found, but Minecraft could not be started.",
        )
        .with_detail(error.to_string())
    })?;
    emit(
        &app,
        "running",
        "Minecraft is running. Game output is being written to minecraft.log.",
        None,
    );
    if let Ok(settings) = crate::settings::load(paths) {
        if let Ok(mut presence) = state.presence.lock() {
            presence.update(
                settings.discord_rich_presence,
                crate::presence::PresenceState::Playing(profile.minecraft_version.clone()),
            );
        }
    }
    let presence_enabled = crate::settings::load(paths)
        .map(|settings| settings.discord_rich_presence)
        .unwrap_or(false);
    tokio::spawn(async move {
        let _autoauth_session = autoauth_session;
        match child.wait().await {
            Ok(status) if status.success() => {
                emit(&app, "finished", "Minecraft exited normally.", None)
            }
            Ok(status) => {
                let message = new_crash_summary(&crash_dir, &existing_crashes).unwrap_or_else(|| {
                    format!("Minecraft exited with status {status}. Check minecraft.log for details.")
                });
                emit(&app, "failed", message, None);
            }
            Err(error) => emit(
                &app,
                "failed",
                format!("Flint could not monitor Minecraft: {error}"),
                None,
            ),
        }
        if let Ok(mut presence) = state.presence.lock() {
            presence.update(presence_enabled, crate::presence::PresenceState::Browsing);
        }
        state.busy.store(false, Ordering::Release);
    });
    Ok(())
}

fn crash_report_names(dir: &Path) -> HashSet<OsString> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok().map(|entry| entry.file_name()))
        .collect()
}

fn new_crash_summary(dir: &Path, existing: &HashSet<OsString>) -> Option<String> {
    let report = fs::read_dir(dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| !existing.contains(&entry.file_name()))
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .max_by_key(|(modified, _)| *modified)?
        .1;
    let contents = fs::read_to_string(&report).ok()?;
    let description = contents
        .lines()
        .find_map(|line| line.strip_prefix("Description: "))
        .unwrap_or("Unexpected error");
    let exception = contents
        .lines()
        .find(|line| line.starts_with("java.") || line.starts_with("org."))
        .and_then(|line| line.split([':', ' ']).next())
        .and_then(|name| name.rsplit('.').next())
        .unwrap_or("Unknown exception");
    let filename = report.file_name()?.to_string_lossy();
    Some(format!(
        "Minecraft crashed: {description} ({exception}). See crash-reports/{filename} and minecraft.log."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_new_crash_cause_without_reusing_an_old_report() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("crash-reports");
        fs::create_dir(&dir).unwrap();
        fs::write(
            dir.join("old.txt"),
            "Description: Old error\njava.lang.OldException: stale",
        )
        .unwrap();
        let before = crash_report_names(&dir);
        assert!(new_crash_summary(&dir, &before).is_none());
        fs::write(
            dir.join("crash-new-client.txt"),
            "---- Minecraft Crash Report ----\nDescription: Unexpected error\n\njava.lang.NullPointerException: missing sound\n",
        )
        .unwrap();
        assert_eq!(
            new_crash_summary(&dir, &before).unwrap(),
            "Minecraft crashed: Unexpected error (NullPointerException). See crash-reports/crash-new-client.txt and minecraft.log."
        );
    }
}
