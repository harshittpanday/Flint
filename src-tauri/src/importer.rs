use crate::{
    error::{AppError, Result},
    paths::AppPaths,
    profiles,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

const MAX_IMPORTED_FILE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportCategory {
    Settings,
    Servers,
    ResourcePacks,
    ShaderPacks,
    Configs,
    Mods,
    Worlds,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Compatibility {
    Compatible,
    Unknown,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportItem {
    pub category: ImportCategory,
    pub name: String,
    pub relative_path: String,
    pub compatibility: Compatibility,
    pub detail: String,
    pub selected_by_default: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub source: String,
    pub profile_id: String,
    pub minecraft_version: String,
    pub loader: profiles::Loader,
    pub items: Vec<ImportItem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub source: String,
    pub profile_id: String,
    pub categories: Vec<ImportCategory>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub files_copied: u64,
    pub items_skipped: u64,
}

pub fn preview(paths: &AppPaths, source: &Path, profile_id: &str) -> Result<ImportPreview> {
    let profile = profiles::find(paths, profile_id)?;
    let source = validate_source(paths, source)?;
    let mut items = Vec::new();
    add_file(
        &source,
        "options.txt",
        ImportCategory::Settings,
        "Controls, sensitivity, and video settings",
        &mut items,
    );
    add_file(
        &source,
        "servers.dat",
        ImportCategory::Servers,
        "Multiplayer server list",
        &mut items,
    );
    for (folder, category, label, default) in [
        (
            "resourcepacks",
            ImportCategory::ResourcePacks,
            "Resource packs",
            true,
        ),
        (
            "shaderpacks",
            ImportCategory::ShaderPacks,
            "Shader packs",
            true,
        ),
        ("config", ImportCategory::Configs, "Mod configuration", true),
        ("saves", ImportCategory::Worlds, "Worlds", false),
    ] {
        if source.join(folder).is_dir() {
            items.push(ImportItem {
                category,
                name: label.into(),
                relative_path: folder.into(),
                compatibility: Compatibility::Compatible,
                detail: if default {
                    "Ready to copy into this isolated profile.".into()
                } else {
                    "Optional and off by default because worlds may be large.".into()
                },
                selected_by_default: default,
            });
        }
    }
    let mods = source.join("mods");
    if mods.is_dir() {
        for entry in fs::read_dir(mods)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("jar"))
            {
                items.push(inspect_mod(&path, &profile)?);
            }
        }
    }
    Ok(ImportPreview {
        source: source.to_string_lossy().into_owned(),
        profile_id: profile.id,
        minecraft_version: profile.minecraft_version,
        loader: profile.loader,
        items,
    })
}

pub fn apply(paths: &AppPaths, request: ImportRequest) -> Result<ImportResult> {
    let preview = preview(paths, Path::new(&request.source), &request.profile_id)?;
    let source = PathBuf::from(&preview.source);
    let target = paths.instance_game(&request.profile_id);
    fs::create_dir_all(&target)?;
    let mut files_copied = 0;
    let mut items_skipped = 0;
    for item in preview.items {
        if !request.categories.contains(&item.category)
            || (item.category == ImportCategory::Mods
                && item.compatibility != Compatibility::Compatible)
        {
            items_skipped += 1;
            continue;
        }
        let from = source.join(&item.relative_path);
        let to = target.join(&item.relative_path);
        if from.is_dir() {
            files_copied += copy_tree(&from, &to)?;
        } else {
            copy_file(&from, &to)?;
            files_copied += 1;
        }
    }
    Ok(ImportResult {
        files_copied,
        items_skipped,
    })
}

fn validate_source(paths: &AppPaths, source: &Path) -> Result<PathBuf> {
    let source = source.canonicalize().map_err(|error| {
        AppError::new(
            "import_source_missing",
            "Choose an existing Minecraft installation directory.",
        )
        .with_detail(error.to_string())
    })?;
    if !source.is_dir() {
        return Err(AppError::new(
            "invalid_import_source",
            "The import source must be a directory.",
        ));
    }
    let managed = [
        &paths.instances,
        &paths.profiles,
        &paths.settings,
        &paths.versions,
        &paths.libraries,
        &paths.assets,
        &paths.metadata,
        &paths.logs,
    ];
    if managed.iter().any(|path| {
        path.canonicalize()
            .map(|managed| source.starts_with(managed))
            .unwrap_or(false)
    }) {
        return Err(AppError::new(
            "unsafe_import_source",
            "Choose an external Minecraft installation, not a Flint-managed directory.",
        ));
    }
    let looks_like_minecraft = [
        "options.txt",
        "servers.dat",
        "launcher_profiles.json",
        "mods",
        "resourcepacks",
        "saves",
        "versions",
    ]
    .iter()
    .any(|entry| source.join(entry).exists());
    if !looks_like_minecraft {
        return Err(AppError::new(
            "unrecognized_import_source",
            "This folder does not look like a Minecraft installation.",
        ));
    }
    Ok(source)
}

fn add_file(
    source: &Path,
    filename: &str,
    category: ImportCategory,
    name: &str,
    items: &mut Vec<ImportItem>,
) {
    if source.join(filename).is_file() {
        items.push(ImportItem {
            category,
            name: name.into(),
            relative_path: filename.into(),
            compatibility: Compatibility::Compatible,
            detail: "Ready to copy into this isolated profile.".into(),
            selected_by_default: true,
        });
    }
}

fn inspect_mod(path: &Path, profile: &profiles::Profile) -> Result<ImportItem> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Unknown mod")
        .to_string();
    let mut item = ImportItem {
        category: ImportCategory::Mods,
        name: filename.clone(),
        relative_path: format!("mods/{filename}"),
        compatibility: Compatibility::Unknown,
        detail: "Compatibility could not be established; this mod will not be imported.".into(),
        selected_by_default: false,
    };
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(_) => return Ok(item),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(_) => return Ok(item),
    };
    let metadata = match archive.by_name("fabric.mod.json") {
        Ok(metadata) => metadata,
        Err(_) => return Ok(item),
    };
    let mut json = String::new();
    if metadata
        .take(1024 * 1024)
        .read_to_string(&mut json)
        .is_err()
    {
        return Ok(item);
    }
    let value: serde_json::Value = match serde_json::from_str(&json) {
        Ok(value) => value,
        Err(_) => return Ok(item),
    };
    if let Some(name) = value.get("name").and_then(|value| value.as_str()) {
        item.name = name.to_string();
    }
    if profile.loader != profiles::Loader::Fabric {
        item.detail = "Fabric mods cannot be imported into a Vanilla profile.".into();
        return Ok(item);
    }
    let requirement = value
        .get("depends")
        .and_then(|value| value.get("minecraft"));
    let exact_match = requirement.is_some_and(|requirement| match requirement {
        serde_json::Value::String(value) => value == &profile.minecraft_version,
        serde_json::Value::Array(values) => values
            .iter()
            .any(|value| value.as_str() == Some(&profile.minecraft_version)),
        _ => false,
    });
    if exact_match {
        item.compatibility = Compatibility::Compatible;
        item.detail = format!(
            "Fabric metadata explicitly supports Minecraft {}.",
            profile.minecraft_version
        );
        item.selected_by_default = true;
    }
    Ok(item)
}

fn copy_tree(source: &Path, target: &Path) -> Result<u64> {
    fs::create_dir_all(target)?;
    let mut copied = 0;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let destination = target.join(entry.file_name());
        if file_type.is_dir() {
            copied += copy_tree(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            copy_file(&entry.path(), &destination)?;
            copied += 1;
        }
    }
    Ok(copied)
}

fn copy_file(source: &Path, target: &Path) -> Result<()> {
    if fs::metadata(source)?.len() > MAX_IMPORTED_FILE_BYTES {
        return Err(AppError::new(
            "import_file_too_large",
            format!(
                "{} exceeds Flint's 512 MB per-file import limit.",
                source.display()
            ),
        ));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::{Loader, Preset, ProfileInput};
    use std::io::Write;

    fn profile(paths: &AppPaths, loader: Loader) -> profiles::Profile {
        profiles::save(
            paths,
            ProfileInput {
                id: None,
                name: "Import target".into(),
                username: "Player_1".into(),
                minecraft_version: crate::DEFAULT_VERSION.into(),
                fabric_loader_version: (loader == Loader::Fabric).then(|| "0.16.14".into()),
                loader,
                preset: Preset::Vanilla,
                memory_mb: 2048,
            },
        )
        .unwrap()
    }

    #[test]
    fn import_preserves_source_and_worlds_are_opt_in() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path().join("flint"));
        paths.ensure().unwrap();
        let profile = profile(&paths, Loader::Vanilla);
        let source = temp.path().join("minecraft");
        fs::create_dir_all(source.join("saves/world")).unwrap();
        fs::write(source.join("options.txt"), "sensitivity:0.5").unwrap();
        fs::write(source.join("saves/world/level.dat"), "world").unwrap();
        let before = fs::read(source.join("options.txt")).unwrap();
        let preview = preview(&paths, &source, &profile.id).unwrap();
        assert!(preview
            .items
            .iter()
            .any(|item| item.category == ImportCategory::Worlds && !item.selected_by_default));
        apply(
            &paths,
            ImportRequest {
                source: source.to_string_lossy().into_owned(),
                profile_id: profile.id.clone(),
                categories: vec![ImportCategory::Settings],
            },
        )
        .unwrap();
        assert_eq!(fs::read(source.join("options.txt")).unwrap(), before);
        assert!(paths
            .instance_game(&profile.id)
            .join("options.txt")
            .exists());
        assert!(!paths.instance_game(&profile.id).join("saves").exists());
    }

    #[test]
    fn rejects_flint_managed_source() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path().join("flint"));
        paths.ensure().unwrap();
        let profile = profile(&paths, Loader::Vanilla);
        assert!(preview(&paths, &paths.instance_game(&profile.id), &profile.id).is_err());
    }

    #[test]
    fn unknown_mod_is_never_silently_compatible() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path().join("flint"));
        paths.ensure().unwrap();
        let profile = profile(&paths, Loader::Fabric);
        let source = temp.path().join("minecraft");
        fs::create_dir_all(source.join("mods")).unwrap();
        let file = fs::File::create(source.join("mods/unknown.jar")).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        archive
            .write_all(br#"{"id":"unknown","name":"Unknown","depends":{"minecraft":">=1.20"}}"#)
            .unwrap();
        archive.finish().unwrap();
        let preview = preview(&paths, &source, &profile.id).unwrap();
        let item = preview
            .items
            .iter()
            .find(|item| item.category == ImportCategory::Mods)
            .unwrap();
        assert_eq!(item.compatibility, Compatibility::Unknown);
        assert!(!item.selected_by_default);
    }
}
