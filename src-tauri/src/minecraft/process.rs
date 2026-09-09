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
    fs::OpenOptions,
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
) -> Result<()> {
    let game_dir = paths.instance_game(&profile.id);
    std::fs::create_dir_all(&game_dir)?;
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
        match child.wait().await {
            Ok(status) if status.success() => {
                emit(&app, "finished", "Minecraft exited normally.", None)
            }
            Ok(status) => emit(
                &app,
                "failed",
                format!("Minecraft exited with status {status}. Check minecraft.log for details."),
                None,
            ),
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
