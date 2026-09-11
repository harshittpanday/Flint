use crate::{
    cosmetics,
    error::{AppError, Result},
    paths::AppPaths,
    profiles::{self, FlintClientState, Loader, Profile},
};
use serde::Serialize;
use std::{fs, path::Path};

pub const SUPPORTED_MINECRAFT_VERSION: &str = "1.21.11";
const CLIENT_VERSION: &str = "0.3.0";
const CLIENT_JAR_NAME: &str = "flint-client-0.3.0+mc1.21.11.jar";
const CLIENT_JAR: &[u8] = include_bytes!("../resources/flint-client-0.3.0+mc1.21.11.jar");
const PROTOCOL_VERSION: u8 = 1;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSupport {
    pub supported: bool,
    pub reason: String,
    pub enabled: bool,
    pub client_version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientConfig {
    protocol_version: u8,
    enabled: bool,
    cosmetics: ClientCosmetics,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientCosmetics {
    skin_enabled: bool,
    skin_path: &'static str,
    skin_model: cosmetics::SkinModel,
    cape_enabled: bool,
    cape_path: &'static str,
}

pub fn support(profile: &Profile) -> ClientSupport {
    let (supported, reason) = if profile.loader != Loader::Fabric {
        (false, "Flint Client requires a Fabric profile.".into())
    } else if profile.minecraft_version != SUPPORTED_MINECRAFT_VERSION {
        (
            false,
            format!(
                "Flint Client {CLIENT_VERSION} currently supports Minecraft {SUPPORTED_MINECRAFT_VERSION} only."
            ),
        )
    } else {
        (true, "Compatible with this Fabric profile.".into())
    };
    ClientSupport {
        supported,
        reason,
        enabled: profile.flint_client_state == FlintClientState::Enabled,
        client_version: CLIENT_VERSION.into(),
    }
}

pub fn set_enabled(paths: &AppPaths, profile_id: &str, enabled: bool) -> Result<Profile> {
    let profile = profiles::find(paths, profile_id)?;
    let support = support(&profile);
    if enabled && !support.supported {
        return Err(AppError::new("flint_client_unsupported", support.reason));
    }
    if enabled {
        install(paths, &profile)?;
    } else {
        uninstall(paths, &profile)?;
    }
    profiles::set_client_state(
        paths,
        profile_id,
        if enabled {
            FlintClientState::Enabled
        } else {
            FlintClientState::Disabled
        },
    )
}

pub fn prepare(paths: &AppPaths, profile: &Profile) -> Result<()> {
    if profile.flint_client_state == FlintClientState::Enabled {
        let support = support(profile);
        if !support.supported {
            tracing::warn!(profile_id = %profile.id, "disabling Flint Client after profile compatibility changed");
            uninstall(paths, profile)?;
            profiles::set_client_state(paths, &profile.id, FlintClientState::Disabled)?;
            return Ok(());
        }
        install(paths, profile)?;
    }
    Ok(())
}

fn install(paths: &AppPaths, profile: &Profile) -> Result<()> {
    let game = paths.instance_game(&profile.id);
    let mods = game.join("mods");
    fs::create_dir_all(&mods)?;
    write_atomic(&mods.join(CLIENT_JAR_NAME), CLIENT_JAR)?;

    let cosmetics = cosmetics::load(paths, &profile.id)?;
    let flint = game.join("flint");
    fs::create_dir_all(&flint)?;
    let client_cosmetics = flint.join("cosmetics");
    fs::create_dir_all(&client_cosmetics)?;
    sync_cosmetic(
        cosmetics.skin_path.as_deref(),
        &client_cosmetics.join("skin.png"),
    )?;
    sync_cosmetic(
        cosmetics.cape_path.as_deref(),
        &client_cosmetics.join("cape.png"),
    )?;
    let config = ClientConfig {
        protocol_version: PROTOCOL_VERSION,
        enabled: true,
        cosmetics: ClientCosmetics {
            skin_enabled: cosmetics.skin_enabled && cosmetics.skin_path.is_some(),
            skin_path: "flint/cosmetics/skin.png",
            skin_model: cosmetics.skin_model,
            cape_enabled: cosmetics.cape_enabled && cosmetics.cape_path.is_some(),
            cape_path: "flint/cosmetics/cape.png",
        },
    };
    write_atomic(
        &flint.join("client-v1.json"),
        &serde_json::to_vec_pretty(&config)?,
    )
}

fn sync_cosmetic(source: Option<&str>, target: &Path) -> Result<()> {
    if let Some(source) = source.filter(|source| Path::new(source).is_file()) {
        fs::copy(source, target)?;
    } else if target.is_file() {
        fs::remove_file(target)?;
    }
    Ok(())
}

fn uninstall(paths: &AppPaths, profile: &Profile) -> Result<()> {
    for path in [
        paths
            .instance_game(&profile.id)
            .join("mods")
            .join(CLIENT_JAR_NAME),
        paths
            .instance_game(&profile.id)
            .join("flint")
            .join("client-v1.json"),
    ] {
        if path.is_file() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = path.with_extension("flint-tmp");
    fs::write(&temporary, bytes)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(temporary, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{Preset, ProfileInput};

    fn profile(paths: &AppPaths, version: &str, loader: Loader) -> Profile {
        profiles::save(
            paths,
            ProfileInput {
                id: None,
                name: "Client test".into(),
                username: "Player_1".into(),
                minecraft_version: version.into(),
                fabric_loader_version: (loader == Loader::Fabric).then(|| "0.18.4".into()),
                loader,
                preset: Preset::Vanilla,
                memory_mb: 2048,
            },
        )
        .unwrap()
    }

    #[test]
    fn support_is_intentionally_narrow() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path());
        paths.ensure().unwrap();
        assert!(
            support(&profile(
                &paths,
                SUPPORTED_MINECRAFT_VERSION,
                Loader::Fabric
            ))
            .supported
        );
        assert!(!support(&profile(&paths, "1.21.10", Loader::Fabric)).supported);
        assert!(
            !support(&profile(
                &paths,
                SUPPORTED_MINECRAFT_VERSION,
                Loader::Vanilla
            ))
            .supported
        );
    }

    #[test]
    fn enable_and_disable_touch_only_the_selected_instance() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path());
        paths.ensure().unwrap();
        let first = profile(&paths, SUPPORTED_MINECRAFT_VERSION, Loader::Fabric);
        let second = profile(&paths, SUPPORTED_MINECRAFT_VERSION, Loader::Fabric);
        set_enabled(&paths, &first.id, true).unwrap();
        assert!(paths
            .instance_game(&first.id)
            .join("mods")
            .join(CLIENT_JAR_NAME)
            .is_file());
        assert!(!paths
            .instance_game(&second.id)
            .join("mods")
            .join(CLIENT_JAR_NAME)
            .exists());
        let config =
            fs::read_to_string(paths.instance_game(&first.id).join("flint/client-v1.json"))
                .unwrap();
        assert!(config.contains("flint/cosmetics/skin.png"));
        assert!(!config.contains(temp.path().to_string_lossy().as_ref()));
        set_enabled(&paths, &first.id, false).unwrap();
        assert!(!paths
            .instance_game(&first.id)
            .join("mods")
            .join(CLIENT_JAR_NAME)
            .exists());
        assert_eq!(
            profiles::find(&paths, &first.id)
                .unwrap()
                .flint_client_state,
            FlintClientState::Disabled
        );
    }

    #[test]
    fn embedded_artifact_is_a_remapped_fabric_mod() {
        let reader = std::io::Cursor::new(CLIENT_JAR);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let mut metadata = String::new();
        use std::io::Read;
        archive
            .by_name("fabric.mod.json")
            .unwrap()
            .read_to_string(&mut metadata)
            .unwrap();
        assert!(metadata.contains("\"id\": \"flint-client\""));
        assert!(metadata.contains("\"minecraft\": \"1.21.11\""));
    }
}
