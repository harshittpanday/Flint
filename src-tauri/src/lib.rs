mod cosmetics;
mod error;
mod importer;
mod java;
mod minecraft;
mod paths;
mod presence;
mod process_command;
mod profiles;
mod settings;

use error::{AppError, Result};
use minecraft::{arguments, catalog, fabric, install, modrinth};
use paths::AppPaths;
use profiles::{Profile, ProfileInput};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tauri::{AppHandle, Manager, State};

pub const DEFAULT_VERSION: &str = "26.2";

pub struct LauncherState {
    pub busy: AtomicBool,
    pub presence: Mutex<presence::Presence>,
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
fn duplicate_profile(paths: State<'_, AppPaths>, id: String) -> Result<Profile> {
    profiles::duplicate(&paths, &id)
}

#[tauri::command]
fn set_flint_client_state(
    paths: State<'_, AppPaths>,
    id: String,
    state: profiles::FlintClientState,
) -> Result<Profile> {
    profiles::set_client_state(&paths, &id, state)
}

#[tauri::command]
fn get_profile_cosmetics(
    paths: State<'_, AppPaths>,
    profile_id: String,
) -> Result<cosmetics::ProfileCosmetics> {
    cosmetics::load(&paths, &profile_id)
}

#[tauri::command]
fn save_profile_cosmetics(
    paths: State<'_, AppPaths>,
    profile_id: String,
    cosmetics: cosmetics::ProfileCosmetics,
) -> Result<cosmetics::ProfileCosmetics> {
    cosmetics::save(&paths, &profile_id, cosmetics)
}

#[tauri::command]
fn import_profile_cosmetic(
    paths: State<'_, AppPaths>,
    profile_id: String,
    source: String,
    kind: String,
) -> Result<cosmetics::ProfileCosmetics> {
    let kind = match kind.as_str() {
        "skin" => cosmetics::CosmeticKind::Skin,
        "cape" => cosmetics::CosmeticKind::Cape,
        _ => {
            return Err(AppError::new(
                "invalid_cosmetic_kind",
                "Choose a skin or cape.",
            ))
        }
    };
    cosmetics::import(&paths, &profile_id, std::path::Path::new(&source), kind)
}

#[tauri::command]
fn remove_profile_cosmetic(
    paths: State<'_, AppPaths>,
    profile_id: String,
    kind: String,
) -> Result<cosmetics::ProfileCosmetics> {
    let kind = match kind.as_str() {
        "skin" => cosmetics::CosmeticKind::Skin,
        "cape" => cosmetics::CosmeticKind::Cape,
        _ => {
            return Err(AppError::new(
                "invalid_cosmetic_kind",
                "Choose a skin or cape.",
            ))
        }
    };
    cosmetics::remove(&paths, &profile_id, kind)
}

#[tauri::command]
fn read_profile_cosmetic(
    paths: State<'_, AppPaths>,
    profile_id: String,
    kind: String,
) -> Result<Vec<u8>> {
    let kind = match kind.as_str() {
        "skin" => cosmetics::CosmeticKind::Skin,
        "cape" => cosmetics::CosmeticKind::Cape,
        _ => {
            return Err(AppError::new(
                "invalid_cosmetic_kind",
                "Choose a skin or cape.",
            ))
        }
    };
    cosmetics::read(&paths, &profile_id, kind)
}

#[tauri::command]
async fn preview_existing_setup(
    paths: State<'_, AppPaths>,
    source: String,
    profile_id: String,
) -> Result<importer::ImportPreview> {
    let paths = paths.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        importer::preview(&paths, std::path::Path::new(&source), &profile_id)
    })
    .await
    .map_err(|error| {
        AppError::new("import_scan_failed", "The setup scan stopped unexpectedly.")
            .with_detail(error.to_string())
    })?
}

#[tauri::command]
async fn import_existing_setup(
    paths: State<'_, AppPaths>,
    request: importer::ImportRequest,
) -> Result<importer::ImportResult> {
    let paths = paths.inner().clone();
    tauri::async_runtime::spawn_blocking(move || importer::apply(&paths, request))
        .await
        .map_err(|error| {
            AppError::new("import_failed", "The setup import stopped unexpectedly.")
                .with_detail(error.to_string())
        })?
}

#[tauri::command]
async fn list_minecraft_versions(
    paths: State<'_, AppPaths>,
    include_snapshots: bool,
) -> Result<Vec<catalog::MinecraftVersion>> {
    catalog::list(&paths, include_snapshots).await
}

#[tauri::command]
async fn list_fabric_loaders(game_version: String) -> Result<Vec<fabric::FabricLoaderVersion>> {
    fabric::list_loaders(&game_version).await
}

#[tauri::command]
async fn search_mods(
    paths: State<'_, AppPaths>,
    profile_id: String,
    query: String,
) -> Result<Vec<modrinth::ModProject>> {
    let profile = profiles::find(&paths, &profile_id)?;
    if profile.loader != profiles::Loader::Fabric {
        return Err(AppError::new(
            "fabric_profile_required",
            "Select a Fabric profile before searching for mods.",
        ));
    }
    modrinth::search(query.trim(), &profile.minecraft_version).await
}

#[tauri::command]
async fn preview_preset(
    game_version: String,
    preset: profiles::Preset,
) -> Result<Vec<modrinth::PresetMod>> {
    modrinth::preview_preset(&game_version, &preset).await
}

#[tauri::command]
async fn apply_profile_preset(
    paths: State<'_, AppPaths>,
    profile_id: String,
) -> Result<Vec<modrinth::InstalledMod>> {
    let profile = profiles::find(&paths, &profile_id)?;
    modrinth::apply_preset(&paths, &profile).await
}

#[tauri::command]
fn list_installed_mods(
    paths: State<'_, AppPaths>,
    profile_id: String,
) -> Result<Vec<modrinth::InstalledMod>> {
    profiles::find(&paths, &profile_id)?;
    modrinth::list_installed(&paths, &profile_id)
}

#[tauri::command]
async fn install_mod(
    paths: State<'_, AppPaths>,
    profile_id: String,
    project_id: String,
) -> Result<Vec<modrinth::InstalledMod>> {
    let profile = profiles::find(&paths, &profile_id)?;
    modrinth::install(&paths, &profile, &project_id).await
}

#[tauri::command]
fn remove_mod(
    paths: State<'_, AppPaths>,
    profile_id: String,
    project_id: String,
) -> Result<Vec<modrinth::InstalledMod>> {
    profiles::find(&paths, &profile_id)?;
    modrinth::remove(&paths, &profile_id, &project_id)
}

#[tauri::command]
fn get_settings(paths: State<'_, AppPaths>) -> Result<settings::LauncherSettings> {
    settings::load(&paths)
}

#[tauri::command]
fn save_settings(
    paths: State<'_, AppPaths>,
    state: State<'_, Arc<LauncherState>>,
    settings: settings::LauncherSettings,
) -> Result<settings::LauncherSettings> {
    let saved = settings::save(&paths, settings)?;
    if let Ok(mut presence) = state.presence.lock() {
        presence.update(
            saved.discord_rich_presence,
            presence::PresenceState::Browsing,
        );
    }
    Ok(saved)
}

#[tauri::command]
fn list_java_runtimes() -> Vec<java::JavaInfo> {
    java::list()
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
        let launcher_settings = settings::load(&paths)?;
        if let Ok(mut presence) = state.presence.lock() {
            presence.update(
                launcher_settings.discord_rich_presence,
                presence::PresenceState::Preparing,
            );
        }
        install::emit(&app, "preparing", "Reading Mojang version metadata…", None);
        let resolved = install::resolve(&paths, &profile.minecraft_version).await?;
        let required_java = resolved
            .metadata
            .java_version
            .as_ref()
            .map(|version| version.major_version)
            .ok_or_else(|| {
                AppError::new(
                    "java_requirement_missing",
                    format!(
                        "Minecraft {} does not declare its required Java runtime in Mojang metadata.",
                        profile.minecraft_version
                    ),
                )
            })?;
        let manual_java = (!launcher_settings.automatic_java)
            .then_some(launcher_settings.manual_java_path.as_deref())
            .flatten();
        let java = java::detect(required_java, manual_java)?;
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
        let mut prepared = install::prepare(&app, &paths, resolved).await?;
        if let Ok(mut presence) = state.presence.lock() {
            presence.update(
                launcher_settings.discord_rich_presence,
                presence::PresenceState::Downloading(profile.minecraft_version.clone()),
            );
        }
        if profile.loader == profiles::Loader::Fabric {
            let loader_version = profile.fabric_loader_version.as_deref().ok_or_else(|| {
                AppError::new("fabric_version_required", "Choose a Fabric Loader version.")
            })?;
            install::emit(
                &app,
                "downloading",
                format!("Preparing Fabric Loader {loader_version}…"),
                Some(0.98),
            );
            fabric::apply(
                &paths,
                &profile.minecraft_version,
                loader_version,
                &mut prepared,
            )
            .await?;
        }
        let game_dir = paths.instance_game(&profile.id);
        let launch_arguments = arguments::build(
            &prepared,
            &paths,
            &profile,
            &game_dir,
            (
                launcher_settings.resolution_width,
                launcher_settings.resolution_height,
            ),
        )?;
        install::emit(
            &app,
            "launching",
            "Starting the Minecraft Java process…",
            Some(1.0),
        );
        if let Ok(mut presence) = state.presence.lock() {
            presence.update(
                launcher_settings.discord_rich_presence,
                presence::PresenceState::Launching(profile.minecraft_version.clone()),
            );
        }
        minecraft::process::launch(
            app.clone(),
            shared_state,
            &paths,
            &profile,
            &java,
            prepared,
            launch_arguments,
        )?;
        profiles::mark_played(&paths, &profile.id)
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
        .plugin(tauri_plugin_dialog::init())
        .manage(paths)
        .manage(Arc::new(LauncherState {
            busy: AtomicBool::new(false),
            presence: Mutex::new(presence::Presence::new()),
        }))
        .setup(move |app| {
            app.manage(guard);
            let paths = app.state::<AppPaths>();
            let state = app.state::<Arc<LauncherState>>();
            let enabled = settings::load(&paths)
                .map(|settings| settings.discord_rich_presence)
                .unwrap_or(false);
            if let Ok(mut presence) = state.presence.lock() {
                presence.update(enabled, presence::PresenceState::Browsing);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_profiles,
            save_profile,
            delete_profile,
            duplicate_profile,
            set_flint_client_state,
            get_profile_cosmetics,
            save_profile_cosmetics,
            import_profile_cosmetic,
            remove_profile_cosmetic,
            read_profile_cosmetic,
            preview_existing_setup,
            import_existing_setup,
            list_minecraft_versions,
            list_fabric_loaders,
            search_mods,
            preview_preset,
            apply_profile_preset,
            list_installed_mods,
            install_mod,
            remove_mod,
            get_settings,
            save_settings,
            list_java_runtimes,
            launch_minecraft
        ])
        .run(tauri::generate_context!())
        .expect("error while running Flint");
}
