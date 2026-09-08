use crate::error::{AppError, Result};
use regex::Regex;
use serde::Serialize;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInfo {
    pub path: PathBuf,
    pub major_version: u32,
    pub description: String,
}

pub fn list() -> Vec<JavaInfo> {
    let mut candidates = Vec::new();
    if let Some(home) = std::env::var_os("JAVA_HOME") {
        candidates.push(PathBuf::from(home).join("bin/java.exe"));
    }
    if let Ok(output) = Command::new("where.exe").arg("java.exe").output() {
        candidates.extend(
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(PathBuf::from),
        );
    }
    for root in [
        r"C:\Program Files\Eclipse Adoptium",
        r"C:\Program Files\Java",
        r"C:\Program Files\Microsoft",
        r"C:\Program Files\Amazon Corretto",
    ] {
        candidates.extend(java_children(Path::new(root)));
    }
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        candidates.extend(java_children(
            &PathBuf::from(local_app_data).join(r"Programs\Eclipse Adoptium"),
        ));
    }
    let mut seen = HashSet::new();
    let mut detected = Vec::new();
    for candidate in candidates {
        if candidate.is_file() && seen.insert(candidate.clone()) {
            if let Some(info) = inspect(&candidate) {
                detected.push(info);
            }
        }
    }
    detected.sort_by_key(|info| std::cmp::Reverse(info.major_version));
    detected
}

pub fn detect(required_major: u32, manual_path: Option<&Path>) -> Result<JavaInfo> {
    let selected = if let Some(path) = manual_path {
        inspect(path).filter(|info| info.major_version == required_major)
    } else {
        list()
            .into_iter()
            .find(|info| info.major_version == required_major)
    }
    .ok_or_else(|| {
        AppError::new(
            "java_not_found",
            format!(
                "This Minecraft version requires a 64-bit Java {required_major} runtime. Install Temurin or Microsoft OpenJDK {required_major}, or choose a compatible executable in Settings."
            ),
        )
    })?;
    tracing::info!(
        path = %selected.path.display(),
        major_version = selected.major_version,
        "detected compatible Java runtime"
    );
    Ok(selected)
}

fn java_children(root: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path().join("bin/java.exe"))
        .collect()
}

pub fn inspect(path: &Path) -> Option<JavaInfo> {
    let output = Command::new(path)
        .args(["-XshowSettings:properties", "-version"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let regex = Regex::new(r#"version "(\d+)(?:\.(\d+))?"#).ok()?;
    let captures = regex.captures(&text)?;
    let first: u32 = captures.get(1)?.as_str().parse().ok()?;
    let major = if first == 1 {
        captures.get(2)?.as_str().parse().ok()?
    } else {
        first
    };
    let is_64_bit = text.lines().any(|line| {
        let line = line.trim().to_ascii_lowercase();
        line.starts_with("os.arch =")
            && (line.contains("amd64") || line.contains("x86_64") || line.contains("aarch64"))
    });
    if !is_64_bit {
        return None;
    }
    let description = text
        .lines()
        .next()
        .unwrap_or("Java runtime")
        .trim()
        .to_string();
    Some(JavaInfo {
        path: path.to_path_buf(),
        major_version: major,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::detect;

    #[test]
    fn java_version_regex_handles_modern_versions() {
        let regex = regex::Regex::new(r#"version "(\d+)(?:\.(\d+))?"#).unwrap();
        let captures = regex
            .captures(r#"openjdk version "21.0.8" 2025-07-15"#)
            .unwrap();
        assert_eq!(captures.get(1).unwrap().as_str(), "21");
    }

    #[test]
    fn detects_installed_runtime_when_requested() {
        let Ok(expected) = std::env::var("FLINT_TEST_JAVA_MAJOR") else {
            return;
        };
        let expected: u32 = expected
            .parse()
            .expect("FLINT_TEST_JAVA_MAJOR must be numeric");
        let detected =
            detect(expected, None).expect("requested installed Java runtime was not detected");
        assert_eq!(detected.major_version, expected);
        assert!(detected.path.is_file());
    }
}
