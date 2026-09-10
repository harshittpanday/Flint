use crate::{
    error::{AppError, Result},
    minecraft::modrinth,
    paths::AppPaths,
    profiles,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
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
    Resolvable,
    Incompatible,
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
    pub mod_id: Option<String>,
    pub mod_version: Option<String>,
    pub environment: Option<String>,
    pub resolution: Option<modrinth::ImportedModResolution>,
    #[serde(skip_serializing)]
    fabric_api_module: bool,
    #[serde(skip_serializing)]
    required_mod_ids: Vec<String>,
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
    pub mods_reinstalled: u64,
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
                mod_id: None,
                mod_version: None,
                environment: None,
                resolution: None,
                fabric_api_module: false,
                required_mod_ids: Vec::new(),
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
    validate_local_dependencies(&mut items);
    Ok(ImportPreview {
        source: source.to_string_lossy().into_owned(),
        profile_id: profile.id,
        minecraft_version: profile.minecraft_version,
        loader: profile.loader,
        items,
    })
}

pub async fn preview_resolved(
    paths: &AppPaths,
    source: &Path,
    profile_id: &str,
) -> Result<ImportPreview> {
    let mut preview = preview(paths, source, profile_id)?;
    let source = PathBuf::from(&preview.source);

    let module_indexes = preview
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            (item.fabric_api_module && item.mod_id.as_deref() != Some("fabric-api"))
                .then_some(index)
        })
        .collect::<Vec<_>>();
    let main_fabric_api = preview
        .items
        .iter()
        .position(|item| item.mod_id.as_deref() == Some("fabric-api"));
    if let Some(main) = main_fabric_api.filter(|_| !module_indexes.is_empty()) {
        preview.items[main].detail.push_str(&format!(
            " {} separate Fabric API module JARs were ignored to prevent duplicates.",
            module_indexes.len()
        ));
        for &index in module_indexes.iter().rev() {
            preview.items.remove(index);
        }
    } else if let Some((&first, rest)) = module_indexes.split_first() {
        match modrinth::resolve_imported_project("P7dR8mSH", &preview.minecraft_version).await {
            Ok(Some(resolution)) => {
                let item = &mut preview.items[first];
                item.name = "Fabric API".into();
                item.compatibility = Compatibility::Resolvable;
                item.detail = format!(
                    "{} Fabric API module JARs will be replaced by one compatible Modrinth installation ({}).",
                    module_indexes.len(), resolution.version_number
                );
                item.selected_by_default = true;
                item.resolution = Some(resolution);
                for &index in rest.iter().rev() {
                    preview.items.remove(index);
                }
            }
            Ok(None) => {
                preview.items[first].compatibility = Compatibility::Incompatible;
                preview.items[first].detail = format!(
                    "Fabric API was identified, but Modrinth has no Fabric release for Minecraft {}.",
                    preview.minecraft_version
                );
                for &index in rest.iter().rev() {
                    preview.items.remove(index);
                }
            }
            Err(error) => tracing::warn!(%error, "Fabric API import resolution was unavailable"),
        }
    }

    for item in preview.items.iter_mut().filter(|item| {
        item.category == ImportCategory::Mods
            && matches!(
                item.compatibility,
                Compatibility::Unknown | Compatibility::Incompatible
            )
            && !item.fabric_api_module
    }) {
        let path = source.join(&item.relative_path);
        let hash = match sha1_file(&path) {
            Ok(hash) => hash,
            Err(error) => {
                tracing::warn!(%error, path = %path.display(), "could not hash imported mod");
                continue;
            }
        };
        match modrinth::resolve_imported_hash(&hash, &preview.minecraft_version).await {
            Ok(Some(matched)) => apply_hash_match(item, matched, &preview.minecraft_version),
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(%error, path = %path.display(), "Modrinth import lookup was unavailable")
            }
        }
    }
    Ok(preview)
}

fn apply_hash_match(
    item: &mut ImportItem,
    matched: modrinth::ImportedHashMatch,
    minecraft_version: &str,
) {
    if let Some(resolution) = matched.compatible {
        item.compatibility = Compatibility::Resolvable;
        item.name = resolution.title.clone();
        item.detail = format!(
            "Identified by SHA-1 as {} and can be reinstalled for Minecraft {} ({}).",
            resolution.title, minecraft_version, resolution.version_number
        );
        item.selected_by_default = true;
        item.resolution = Some(resolution);
    } else {
        item.compatibility = Compatibility::Incompatible;
        item.detail = format!(
            "Identified by SHA-1 as {}, but no compatible Fabric release exists for Minecraft {}.",
            matched.title, minecraft_version
        );
    }
}

pub async fn apply(paths: &AppPaths, request: ImportRequest) -> Result<ImportResult> {
    let preview = preview_resolved(paths, Path::new(&request.source), &request.profile_id).await?;
    let profile = profiles::find(paths, &request.profile_id)?;
    let source = PathBuf::from(&preview.source);
    let target = paths.instance_game(&request.profile_id);
    fs::create_dir_all(&target)?;
    let mut files_copied = 0;
    let mut mods_reinstalled = 0;
    let mut items_skipped = 0;
    let mut installed_projects = std::collections::HashSet::new();
    for item in preview.items {
        if !request.categories.contains(&item.category) {
            items_skipped += 1;
            continue;
        }
        if item.category == ImportCategory::Mods {
            match item.compatibility {
                Compatibility::Resolvable => {
                    let Some(resolution) = item.resolution else {
                        items_skipped += 1;
                        continue;
                    };
                    if installed_projects.insert(resolution.project_id.clone()) {
                        modrinth::install(paths, &profile, &resolution.project_id).await?;
                        mods_reinstalled += 1;
                    }
                    continue;
                }
                Compatibility::Compatible => {}
                Compatibility::Unknown | Compatibility::Incompatible => {
                    items_skipped += 1;
                    continue;
                }
            }
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
        mods_reinstalled,
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
            mod_id: None,
            mod_version: None,
            environment: None,
            resolution: None,
            fabric_api_module: false,
            required_mod_ids: Vec::new(),
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
        mod_id: None,
        mod_version: None,
        environment: None,
        resolution: None,
        fabric_api_module: false,
        required_mod_ids: Vec::new(),
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
    item.mod_id = value
        .get("id")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    item.mod_version = value
        .get("version")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    item.environment = value
        .get("environment")
        .and_then(|value| value.as_str())
        .map(str::to_owned);
    item.fabric_api_module = value
        .get("custom")
        .and_then(|custom| custom.get("fabric-api:module-lifecycle"))
        .is_some();
    if let Some(name) = value.get("name").and_then(|value| value.as_str()) {
        item.name = name.to_string();
    }
    if profile.loader != profiles::Loader::Fabric {
        item.compatibility = Compatibility::Incompatible;
        item.detail = "Fabric mods cannot be imported into a Vanilla profile.".into();
        return Ok(item);
    }

    if item.environment.as_deref() == Some("server") {
        item.compatibility = Compatibility::Incompatible;
        item.detail = "This JAR declares a server-only Fabric environment.".into();
        return Ok(item);
    }
    if !matches!(
        item.environment.as_deref(),
        None | Some("*") | Some("client")
    ) {
        item.detail =
            "The Fabric environment declaration is not recognized; review required.".into();
        return Ok(item);
    }

    let depends = value.get("depends").and_then(|value| value.as_object());
    let minecraft = depends
        .and_then(|depends| depends.get("minecraft"))
        .map(|requirement| evaluate_requirement(requirement, &profile.minecraft_version))
        .unwrap_or(PredicateOutcome::Unknown);
    let loader = depends
        .and_then(|depends| depends.get("fabricloader"))
        .and_then(|requirement| {
            profile
                .fabric_loader_version
                .as_deref()
                .map(|version| evaluate_requirement(requirement, version))
        })
        .unwrap_or(PredicateOutcome::Matches);
    item.required_mod_ids = depends
        .into_iter()
        .flat_map(|entries| entries.keys())
        .filter(|id| !matches!(id.as_str(), "minecraft" | "fabricloader" | "java"))
        .cloned()
        .collect();
    let required_count = item.required_mod_ids.len();
    let optional_count = ["recommends", "suggests"]
        .iter()
        .filter_map(|key| value.get(key).and_then(|entry| entry.as_object()))
        .map(serde_json::Map::len)
        .sum::<usize>();
    let identity = format!(
        "{}{}{}",
        item.mod_id.as_deref().unwrap_or("unknown id"),
        item.mod_version
            .as_deref()
            .map(|version| format!(" {version}"))
            .unwrap_or_default(),
        item.environment
            .as_deref()
            .map(|environment| format!(" · {environment}"))
            .unwrap_or_default()
    );
    match (minecraft, loader) {
        (PredicateOutcome::Matches, PredicateOutcome::Matches) => {
            item.compatibility = Compatibility::Compatible;
            item.detail = format!(
                "{identity} supports Minecraft {} ({} required and {} optional mod dependencies declared).",
                profile.minecraft_version, required_count, optional_count
            );
            item.selected_by_default = !item.fabric_api_module;
        }
        (PredicateOutcome::Excludes, _) => {
            item.compatibility = Compatibility::Incompatible;
            item.detail = format!(
                "{identity} explicitly excludes Minecraft {}.",
                profile.minecraft_version
            );
        }
        (_, PredicateOutcome::Excludes) => {
            item.compatibility = Compatibility::Incompatible;
            item.detail = format!(
                "{identity} is incompatible with Fabric Loader {}.",
                profile
                    .fabric_loader_version
                    .as_deref()
                    .unwrap_or("unknown")
            );
        }
        _ => {
            item.detail = format!(
                "{identity} has an absent or unsupported version predicate; review required."
            );
        }
    }
    Ok(item)
}

fn validate_local_dependencies(items: &mut [ImportItem]) {
    let available = items
        .iter()
        .filter_map(|item| item.mod_id.clone())
        .collect::<std::collections::HashSet<_>>();
    for item in items.iter_mut().filter(|item| {
        item.category == ImportCategory::Mods && item.compatibility == Compatibility::Compatible
    }) {
        let missing = item
            .required_mod_ids
            .iter()
            .filter(|id| !available.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            item.compatibility = Compatibility::Unknown;
            item.selected_by_default = false;
            item.detail = format!(
                "Version metadata matches, but required dependencies are not present in the scan: {}. Flint will try a Modrinth reinstall.",
                missing.join(", ")
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PredicateOutcome {
    Matches,
    Excludes,
    Unknown,
}

fn evaluate_requirement(requirement: &serde_json::Value, target: &str) -> PredicateOutcome {
    match requirement {
        serde_json::Value::String(predicate) => evaluate_predicate(predicate, target),
        serde_json::Value::Array(predicates) if !predicates.is_empty() => combine_or(
            predicates
                .iter()
                .map(|predicate| evaluate_requirement(predicate, target)),
        ),
        _ => PredicateOutcome::Unknown,
    }
}

fn evaluate_predicate(predicate: &str, target: &str) -> PredicateOutcome {
    let predicate = predicate.trim();
    if predicate.is_empty() {
        return PredicateOutcome::Unknown;
    }
    if predicate == target || predicate == "*" {
        return PredicateOutcome::Matches;
    }
    combine_or(
        predicate
            .split("||")
            .map(|branch| evaluate_branch(branch.trim(), target)),
    )
}

fn combine_or(outcomes: impl Iterator<Item = PredicateOutcome>) -> PredicateOutcome {
    let outcomes = outcomes.collect::<Vec<_>>();
    if outcomes
        .iter()
        .any(|outcome| *outcome == PredicateOutcome::Matches)
    {
        PredicateOutcome::Matches
    } else if !outcomes.is_empty()
        && outcomes
            .iter()
            .all(|outcome| *outcome == PredicateOutcome::Excludes)
    {
        PredicateOutcome::Excludes
    } else {
        PredicateOutcome::Unknown
    }
}

fn evaluate_branch(branch: &str, target: &str) -> PredicateOutcome {
    if let Some((minimum, maximum)) = branch.split_once(" - ") {
        return combine_and([
            compare_token(&format!(">={}", minimum.trim()), target),
            compare_token(&format!("<={}", maximum.trim()), target),
        ]);
    }
    combine_and(
        branch
            .split(|character: char| character.is_whitespace() || character == ',')
            .filter(|token| !token.is_empty())
            .map(|token| compare_token(token, target)),
    )
}

fn combine_and(outcomes: impl IntoIterator<Item = PredicateOutcome>) -> PredicateOutcome {
    let outcomes = outcomes.into_iter().collect::<Vec<_>>();
    if outcomes
        .iter()
        .any(|outcome| *outcome == PredicateOutcome::Excludes)
    {
        PredicateOutcome::Excludes
    } else if !outcomes.is_empty()
        && outcomes
            .iter()
            .all(|outcome| *outcome == PredicateOutcome::Matches)
    {
        PredicateOutcome::Matches
    } else {
        PredicateOutcome::Unknown
    }
}

fn compare_token(token: &str, target: &str) -> PredicateOutcome {
    if token == "*" || token.eq_ignore_ascii_case("x") {
        return PredicateOutcome::Matches;
    }
    let Some(target) = normalized_version(target) else {
        return PredicateOutcome::Unknown;
    };
    let wildcard = token.trim_start_matches(['=', 'v']);
    if wildcard
        .split('.')
        .any(|part| part == "*" || part.eq_ignore_ascii_case("x"))
    {
        let expected = wildcard
            .split('.')
            .take_while(|part| *part != "*" && !part.eq_ignore_ascii_case("x"))
            .collect::<Vec<_>>();
        let actual = [
            target.major.to_string(),
            target.minor.to_string(),
            target.patch.to_string(),
        ];
        return if expected
            .iter()
            .zip(actual.iter())
            .all(|(left, right)| left == right)
        {
            PredicateOutcome::Matches
        } else {
            PredicateOutcome::Excludes
        };
    }
    let (operator, raw) = [">=", "<=", ">", "<", "=", "~", "^"]
        .iter()
        .find_map(|operator| token.strip_prefix(operator).map(|raw| (*operator, raw)))
        .unwrap_or(("=", token));
    let Some(required) = normalized_version(raw) else {
        return PredicateOutcome::Unknown;
    };
    let matches = match operator {
        ">=" => target >= required,
        "<=" => target <= required,
        ">" => target > required,
        "<" => target < required,
        "~" => {
            let upper = semver::Version::new(required.major, required.minor + 1, 0);
            target >= required && target < upper
        }
        "^" => {
            let upper = if required.major > 0 {
                semver::Version::new(required.major + 1, 0, 0)
            } else if required.minor > 0 {
                semver::Version::new(0, required.minor + 1, 0)
            } else {
                semver::Version::new(0, 0, required.patch + 1)
            };
            target >= required && target < upper
        }
        _ => target == required,
    };
    if matches {
        PredicateOutcome::Matches
    } else {
        PredicateOutcome::Excludes
    }
}

fn normalized_version(value: &str) -> Option<semver::Version> {
    let value = value.trim().trim_start_matches('v');
    if let Ok(version) = semver::Version::parse(value) {
        return Some(version);
    }
    let (base, suffix) = value
        .split_once('-')
        .map_or((value, None), |(base, suffix)| (base, Some(suffix)));
    let components = base.split('.').collect::<Vec<_>>();
    if components.is_empty()
        || components.len() > 3
        || components
            .iter()
            .any(|component| component.parse::<u64>().is_err())
    {
        return None;
    }
    let mut normalized = components.join(".");
    for _ in components.len()..3 {
        normalized.push_str(".0");
    }
    if let Some(suffix) = suffix {
        normalized.push('-');
        normalized.push_str(suffix);
    }
    semver::Version::parse(&normalized).ok()
}

fn sha1_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
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

    fn profile_with_version(
        paths: &AppPaths,
        loader: Loader,
        minecraft_version: &str,
    ) -> profiles::Profile {
        profiles::save(
            paths,
            ProfileInput {
                id: None,
                name: "Import target".into(),
                username: "Player_1".into(),
                minecraft_version: minecraft_version.into(),
                fabric_loader_version: (loader == Loader::Fabric).then(|| "0.16.14".into()),
                loader,
                preset: Preset::Vanilla,
                memory_mb: 2048,
            },
        )
        .unwrap()
    }

    fn profile(paths: &AppPaths, loader: Loader) -> profiles::Profile {
        profile_with_version(paths, loader, crate::DEFAULT_VERSION)
    }

    fn write_mod(path: &Path, metadata: &str) {
        let file = fs::File::create(path).unwrap();
        let mut archive = zip::ZipWriter::new(file);
        archive
            .start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(metadata.as_bytes()).unwrap();
        archive.finish().unwrap();
    }

    #[tokio::test]
    async fn import_preserves_source_and_worlds_are_opt_in() {
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
        .await
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
        write_mod(
            &source.join("mods/unknown.jar"),
            r#"{"id":"unknown","name":"Unknown","version":"1.0","depends":{"minecraft":"around-1.20"}}"#,
        );
        let preview = preview(&paths, &source, &profile.id).unwrap();
        let item = preview
            .items
            .iter()
            .find(|item| item.category == ImportCategory::Mods)
            .unwrap();
        assert_eq!(item.compatibility, Compatibility::Unknown);
        assert!(!item.selected_by_default);
    }

    #[test]
    fn fabric_version_predicates_cover_common_ranges() {
        for predicate in [
            "1.21.11",
            ">=1.21.11",
            ">=1.21.9 <1.22",
            "~1.21.11",
            "^1.21",
            "1.21.x",
            "1.21.10 - 1.21.12",
        ] {
            assert_eq!(
                evaluate_predicate(predicate, "1.21.11"),
                PredicateOutcome::Matches,
                "predicate {predicate}"
            );
        }
        assert_eq!(
            evaluate_predicate("<1.21.11", "1.21.11"),
            PredicateOutcome::Excludes
        );
        assert_eq!(
            evaluate_predicate("not-a-version", "1.21.11"),
            PredicateOutcome::Unknown
        );
    }

    #[test]
    fn same_version_fabric_metadata_range_is_compatible() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path().join("flint"));
        paths.ensure().unwrap();
        let profile = profile_with_version(&paths, Loader::Fabric, "1.21.11");
        let jar = temp.path().join("sodium.jar");
        write_mod(
            &jar,
            r#"{"schemaVersion":1,"id":"sodium","name":"Sodium","version":"0.8.0","environment":"client","depends":{"fabricloader":">=0.16.0","minecraft":">=1.21.9 <1.22"},"recommends":{"fabric-api":"*"},"suggests":{"reeses-sodium-options":"*"}}"#,
        );
        let item = inspect_mod(&jar, &profile).unwrap();
        assert_eq!(item.mod_id.as_deref(), Some("sodium"));
        assert_eq!(item.mod_version.as_deref(), Some("0.8.0"));
        assert_eq!(item.environment.as_deref(), Some("client"));
        assert_eq!(item.compatibility, Compatibility::Compatible);
        assert!(item.selected_by_default);
    }

    #[test]
    fn known_hash_match_becomes_resolvable_reinstall() {
        let mut item = ImportItem {
            category: ImportCategory::Mods,
            name: "local.jar".into(),
            relative_path: "mods/local.jar".into(),
            compatibility: Compatibility::Unknown,
            detail: String::new(),
            selected_by_default: false,
            mod_id: None,
            mod_version: None,
            environment: None,
            resolution: None,
            fabric_api_module: false,
            required_mod_ids: Vec::new(),
        };
        apply_hash_match(
            &mut item,
            modrinth::ImportedHashMatch {
                title: "Sodium".into(),
                compatible: Some(modrinth::ImportedModResolution {
                    project_id: "AANobbMI".into(),
                    title: "Sodium".into(),
                    version_number: "mc1.21.11-0.8.0".into(),
                }),
            },
            "1.21.11",
        );
        assert_eq!(item.compatibility, Compatibility::Resolvable);
        assert_eq!(item.resolution.unwrap().project_id, "AANobbMI");
        assert!(item.selected_by_default);
    }

    #[tokio::test]
    async fn fabric_api_modules_do_not_duplicate_the_main_package() {
        let temp = tempfile::tempdir().unwrap();
        let paths = AppPaths::at(temp.path().join("flint"));
        paths.ensure().unwrap();
        let profile = profile_with_version(&paths, Loader::Fabric, "1.21.11");
        let source = temp.path().join("minecraft");
        fs::create_dir_all(source.join("mods")).unwrap();
        write_mod(
            &source.join("mods/fabric-api.jar"),
            r#"{"id":"fabric-api","name":"Fabric API","version":"1","depends":{"minecraft":">=1.21.11"}}"#,
        );
        write_mod(
            &source.join("mods/fabric-api-base.jar"),
            r#"{"id":"fabric-api-base","name":"Fabric API Base","version":"1","depends":{"minecraft":">=1.21.11"},"custom":{"fabric-api:module-lifecycle":"stable"}}"#,
        );
        let preview = preview_resolved(&paths, &source, &profile.id)
            .await
            .unwrap();
        let mods = preview
            .items
            .iter()
            .filter(|item| item.category == ImportCategory::Mods)
            .collect::<Vec<_>>();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].mod_id.as_deref(), Some("fabric-api"));
        assert!(mods[0].detail.contains("ignored to prevent duplicates"));
    }
}
