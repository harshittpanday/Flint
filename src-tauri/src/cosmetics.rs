use crate::{
    error::{AppError, Result},
    paths::AppPaths,
    profiles,
};
use serde::{Deserialize, Serialize};
use std::{fs, io::BufReader, path::Path};

const MAX_COSMETIC_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkinModel {
    #[default]
    Classic,
    Slim,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCosmetics {
    pub skin_path: Option<String>,
    pub cape_path: Option<String>,
    pub skin_model: SkinModel,
    pub cape_enabled: bool,
}

#[derive(Clone, Copy)]
pub enum CosmeticKind {
    Skin,
    Cape,
}

fn directory(paths: &AppPaths, profile_id: &str) -> std::path::PathBuf {
    paths.instance(profile_id).join("flint").join("cosmetics")
}

fn settings_file(paths: &AppPaths, profile_id: &str) -> std::path::PathBuf {
    directory(paths, profile_id).join("cosmetics.json")
}

pub fn load(paths: &AppPaths, profile_id: &str) -> Result<ProfileCosmetics> {
    profiles::find(paths, profile_id)?;
    let file = settings_file(paths, profile_id);
    if !file.exists() {
        return Ok(ProfileCosmetics::default());
    }
    serde_json::from_slice(&fs::read(file)?).map_err(Into::into)
}

pub fn save(
    paths: &AppPaths,
    profile_id: &str,
    cosmetics: ProfileCosmetics,
) -> Result<ProfileCosmetics> {
    profiles::find(paths, profile_id)?;
    let folder = directory(paths, profile_id);
    fs::create_dir_all(&folder)?;
    let temporary = folder.join("cosmetics.json.tmp");
    fs::write(&temporary, serde_json::to_vec_pretty(&cosmetics)?)?;
    let target = settings_file(paths, profile_id);
    if target.exists() {
        fs::remove_file(&target)?;
    }
    fs::rename(temporary, target)?;
    Ok(cosmetics)
}

pub fn import(
    paths: &AppPaths,
    profile_id: &str,
    source: &Path,
    kind: CosmeticKind,
) -> Result<ProfileCosmetics> {
    profiles::find(paths, profile_id)?;
    let source = source.canonicalize().map_err(|error| {
        AppError::new("cosmetic_missing", "Choose an existing PNG image.")
            .with_detail(error.to_string())
    })?;
    if !source.is_file()
        || !source
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("png"))
    {
        return Err(AppError::new(
            "invalid_cosmetic_format",
            "Cosmetics must be PNG images.",
        ));
    }
    if fs::metadata(&source)?.len() > MAX_COSMETIC_BYTES {
        return Err(AppError::new(
            "cosmetic_too_large",
            "Cosmetic images must be 2 MB or smaller.",
        ));
    }
    let (width, height) = png_dimensions(&source)?;
    let valid = match kind {
        CosmeticKind::Skin => width == 64 && matches!(height, 32 | 64),
        CosmeticKind::Cape => matches!((width, height), (64, 32) | (64, 64)),
    };
    if !valid {
        let expected = match kind {
            CosmeticKind::Skin => "64×64 (or legacy 64×32)",
            CosmeticKind::Cape => "64×32 or 64×64",
        };
        return Err(AppError::new(
            "invalid_cosmetic_dimensions",
            format!("This image is {width}×{height}; expected {expected}."),
        ));
    }

    let folder = directory(paths, profile_id);
    fs::create_dir_all(&folder)?;
    let filename = match kind {
        CosmeticKind::Skin => "skin.png",
        CosmeticKind::Cape => "cape.png",
    };
    let target = folder.join(filename);
    fs::copy(source, &target)?;
    let mut cosmetics = load(paths, profile_id)?;
    match kind {
        CosmeticKind::Skin => cosmetics.skin_path = Some(target.to_string_lossy().into_owned()),
        CosmeticKind::Cape => {
            cosmetics.cape_path = Some(target.to_string_lossy().into_owned());
            cosmetics.cape_enabled = true;
        }
    }
    save(paths, profile_id, cosmetics)
}

pub fn remove(paths: &AppPaths, profile_id: &str, kind: CosmeticKind) -> Result<ProfileCosmetics> {
    let mut cosmetics = load(paths, profile_id)?;
    let filename = match kind {
        CosmeticKind::Skin => {
            cosmetics.skin_path = None;
            "skin.png"
        }
        CosmeticKind::Cape => {
            cosmetics.cape_path = None;
            cosmetics.cape_enabled = false;
            "cape.png"
        }
    };
    let target = directory(paths, profile_id).join(filename);
    if target.exists() {
        fs::remove_file(target)?;
    }
    save(paths, profile_id, cosmetics)
}

pub fn read(paths: &AppPaths, profile_id: &str, kind: CosmeticKind) -> Result<Vec<u8>> {
    profiles::find(paths, profile_id)?;
    let filename = match kind {
        CosmeticKind::Skin => "skin.png",
        CosmeticKind::Cape => "cape.png",
    };
    let path = directory(paths, profile_id).join(filename);
    if !path.is_file() {
        return Err(AppError::new(
            "cosmetic_missing",
            "No local cosmetic is selected.",
        ));
    }
    fs::read(path).map_err(Into::into)
}

fn png_dimensions(path: &Path) -> Result<(u32, u32)> {
    let decoder = png::Decoder::new(BufReader::new(fs::File::open(path)?));
    let reader = decoder.read_info().map_err(|error| {
        AppError::new(
            "invalid_cosmetic_png",
            "The selected image is not a valid PNG.",
        )
        .with_detail(error.to_string())
    })?;
    Ok((reader.info().width, reader.info().height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{Loader, Preset, ProfileInput};

    fn profile(paths: &AppPaths, name: &str) -> profiles::Profile {
        profiles::save(
            paths,
            ProfileInput {
                id: None,
                name: name.into(),
                username: "Player_1".into(),
                minecraft_version: crate::DEFAULT_VERSION.into(),
                loader: Loader::Vanilla,
                fabric_loader_version: None,
                preset: Preset::Vanilla,
                memory_mb: 2048,
            },
        )
        .unwrap()
    }

    fn make_png(path: &Path, width: u32, height: u32) {
        let file = fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer
            .write_image_data(&vec![0; (width * height * 4) as usize])
            .unwrap();
    }

    #[test]
    fn validates_skin_dimensions_and_preserves_source() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path().join("flint"));
        paths.ensure().unwrap();
        let profile = profile(&paths, "Cosmetics");
        let source = temp.path().join("skin.png");
        make_png(&source, 64, 64);
        let before = fs::read(&source).unwrap();
        let saved = import(&paths, &profile.id, &source, CosmeticKind::Skin).unwrap();
        assert!(saved.skin_path.is_some());
        assert_eq!(fs::read(&source).unwrap(), before);
        assert!(import(&paths, &profile.id, &source, CosmeticKind::Cape).is_ok());
    }

    #[test]
    fn rejects_invalid_skin_and_isolates_profile_state() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path().join("flint"));
        paths.ensure().unwrap();
        let first = profile(&paths, "One");
        let second = profile(&paths, "Two");
        let invalid = temp.path().join("invalid.png");
        make_png(&invalid, 128, 128);
        assert!(import(&paths, &first.id, &invalid, CosmeticKind::Skin).is_err());
        let valid = temp.path().join("valid.png");
        make_png(&valid, 64, 64);
        import(&paths, &first.id, &valid, CosmeticKind::Skin).unwrap();
        assert!(load(&paths, &first.id).unwrap().skin_path.is_some());
        assert!(load(&paths, &second.id).unwrap().skin_path.is_none());
    }
}
