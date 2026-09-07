mod error;
mod java;
mod minecraft;
mod paths;
mod profiles;

use error::{AppError, Result};
use minecraft::{arguments, install};
use paths::AppPaths;
use profiles::{Profile, ProfileInput};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{AppHandle, Manager, State};

pub const SUPPORTED_VERSION: &str = "26.2";
pub const REQUIRED_JAVA_MAJOR: u32 = 25;

pub struct LauncherState {
    pub busy: AtomicBool,
}

#[tauri::command]
fn list_profiles(paths: State<'_, AppPaths>) -> Result<Vec<Profile>> {
    profiles::list(&paths)
}

#[tauri::command]
fn save_profile(paths: State<'_, AppPaths>, profile: ProfileInput) -> Result<Profile> {
    profiles::save(&paths, profile)
}

#[tauri::command]
fn delete_profile(paths: State<'_, AppPaths>, id: String) -> Result<()> {
    profiles::delete(&paths, &id)
}

#[tauri::command]
fn detect_java() -> Result<java::JavaInfo> {
    java::detect(REQUIRED_JAVA_MAJOR)
}

#[tauri::command]
async fn launch_minecraft(
    app: AppHandle,
    paths: State<'_, AppPaths>,
    state: State<'_, Arc<LauncherState>>,
    profile_id: String,
) -> Result<()> {
    if state
        .busy
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(AppError::new(
            "launch_in_progress",
            "Minecraft is already being prepared or is running.",
        ));
    }
    let shared_state = state.inner().clone();
    let result = async {
        let profile = profiles::find(&paths, &profile_id)?;
        let java = java::detect(REQUIRED_JAVA_MAJOR)?;
        install::emit(
            &app,
            "preparing",
            format!(
                "Using Java {} from {}.",
                java.major_version,
                java.path.display()
            ),
            None,
        );
        let prepared = install::prepare(&app, &paths, &profile.minecraft_version).await?;
        let game_dir = paths.instance_game(&profile.id);
        let launch_arguments = arguments::build(&prepared, &paths, &profile, &game_dir)?;
        install::emit(
            &app,
            "launching",
            "Starting the Minecraft Java process…",
            Some(1.0),
        );
        minecraft::process::launch(
            app.clone(),
            shared_state,
            &paths,
            &profile,
            &java,
            prepared,
            launch_arguments,
        )
    }
    .await;
    if let Err(error) = &result {
        tracing::error!(code = error.code, detail = ?error.detail, "launch failed");
        state.busy.store(false, Ordering::Release);
    }
    result
}

pub fn run() {
    let paths = AppPaths::discover().expect("Windows application data directory is required");
    paths
        .ensure()
        .expect("Flint application directories could not be created");
    let log_appender = tracing_appender::rolling::daily(&paths.logs, "flint.log");
    let (writer, guard) = tracing_appender::non_blocking(log_appender);
    tracing_subscriber::fmt()
        .with_env_filter("flint=info")
        .with_writer(writer)
        .init();
    tauri::Builder::default()
        .manage(paths)
        .manage(Arc::new(LauncherState {
            busy: AtomicBool::new(false),
        }))
        .setup(move |app| {
            app.manage(guard);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            save_profile,
            delete_profile,
            detect_java,
            launch_minecraft
        ])
        .run(tauri::generate_context!())
        .expect("error while running Flint");
}
