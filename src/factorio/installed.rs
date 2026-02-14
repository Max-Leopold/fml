use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct InstalledMod {
    pub name: String,
    pub version: semver::Version,
    pub title: String,
    pub factorio_version: String,
}

#[derive(Debug, Deserialize)]
struct InfoJson {
    name: String,
    version: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    factorio_version: String,
}

pub fn read_installed_mods(mods_dir: &str) -> Result<Vec<InstalledMod>> {
    let mut installed = Vec::new();
    let dir = Path::new(mods_dir);

    if !dir.is_dir() {
        bail!("Mods directory does not exist: {}", mods_dir);
    }

    for entry in std::fs::read_dir(dir).context("Failed to read mods directory")? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let is_zip = path
            .extension()
            .map(|ext| ext == "zip")
            .unwrap_or(false);

        if !is_zip || !path.is_file() {
            continue;
        }

        match parse_mod_zip(&path) {
            Ok(m) => installed.push(m),
            Err(e) => {
                eprintln!(
                    "Warning: skipping {}: {}",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    e
                );
            }
        }
    }

    installed.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(installed)
}

fn parse_mod_zip(path: &Path) -> Result<InstalledMod> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open {}", path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("Failed to read zip: {}", path.display()))?;

    // Find info.json — it may be at the root or inside a top-level directory
    let info_name = (0..archive.len())
        .filter_map(|i| {
            let file = archive.by_index(i).ok()?;
            let name = file.name().to_string();
            if name.ends_with("info.json") {
                // Accept "info.json" or "modname_version/info.json" (one level deep)
                let parts: Vec<&str> = name.split('/').collect();
                if parts.len() <= 2 {
                    return Some(name);
                }
            }
            None
        })
        .next();

    let info_name = info_name
        .with_context(|| format!("No info.json found in {}", path.display()))?;

    let mut info_file = archive.by_name(&info_name)
        .with_context(|| format!("Failed to read info.json from {}", path.display()))?;

    let mut contents = String::new();
    info_file
        .read_to_string(&mut contents)
        .with_context(|| format!("Failed to read info.json contents from {}", path.display()))?;

    let info: InfoJson = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse info.json from {}", path.display()))?;

    let version = semver::Version::parse(&info.version)
        .with_context(|| format!("Invalid version '{}' in {}", info.version, path.display()))?;

    let title = if info.title.is_empty() {
        info.name.clone()
    } else {
        info.title
    };

    Ok(InstalledMod {
        name: info.name,
        version,
        title,
        factorio_version: info.factorio_version,
    })
}

pub fn delete_mod(mod_name: &str, version: &semver::Version, mods_dir: &str) -> Result<()> {
    let dir = Path::new(mods_dir);
    let expected_prefix = format!("{}_", mod_name);

    for entry in std::fs::read_dir(dir).context("Failed to read mods directory")? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with(&expected_prefix) && file_name.ends_with(".zip") {
            // Verify this is the exact mod by checking it matches name_version.zip
            let expected_file = format!("{}_{}.zip", mod_name, version);
            if file_name == expected_file {
                std::fs::remove_file(entry.path())
                    .with_context(|| format!("Failed to delete {}", file_name))?;
                return Ok(());
            }
        }
    }

    bail!(
        "Mod file not found: {}_{}.zip in {}",
        mod_name,
        version,
        mods_dir
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delete_mod_precise_matching() {
        // Verify that delete_mod("bob", ...) would NOT match "boblogistics_1.0.0.zip"
        // We test this by checking the prefix logic: "bob_" != "boblogistics_"
        let prefix_bob = format!("{}_", "bob");
        let prefix_boblogistics = format!("{}_", "boblogistics");

        assert!(!"boblogistics_1.0.0.zip".starts_with(&prefix_bob));
        assert!("boblogistics_1.0.0.zip".starts_with(&prefix_boblogistics));
        assert!("bob_1.0.0.zip".starts_with(&prefix_bob));
    }
}
