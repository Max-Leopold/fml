use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct ModEntry {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct ModList {
    pub mods: HashMap<String, ModEntry>,
}

// On-disk format matching Factorio's native mod-list.json
#[derive(Debug, Serialize, Deserialize)]
struct InternalModList {
    mods: Vec<InternalModEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct InternalModEntry {
    name: String,
    enabled: bool,
}

impl ModList {
    pub fn new() -> Self {
        let mut mods = HashMap::new();
        mods.insert(
            "base".to_string(),
            ModEntry {
                name: "base".to_string(),
                enabled: true,
            },
        );
        ModList { mods }
    }

    pub fn load_or_create(mods_dir: &str) -> Result<Self> {
        let file_path = Path::new(mods_dir).join("mod-list.json");

        if file_path.exists() {
            let contents = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;
            let internal: InternalModList = serde_json::from_str(&contents)
                .with_context(|| format!("Failed to parse {}", file_path.display()))?;

            let mods = internal
                .mods
                .into_iter()
                .map(|e| {
                    (
                        e.name.clone(),
                        ModEntry {
                            name: e.name,
                            enabled: e.enabled,
                        },
                    )
                })
                .collect();

            Ok(ModList { mods })
        } else {
            Ok(ModList::new())
        }
    }

    pub fn save(&self, mods_dir: &str) -> Result<()> {
        let file_path = Path::new(mods_dir).join("mod-list.json");

        // Always include base as enabled, then all others
        let mut entries: Vec<InternalModEntry> = Vec::new();
        entries.push(InternalModEntry {
            name: "base".to_string(),
            enabled: true,
        });

        let mut others: Vec<&ModEntry> = self
            .mods
            .values()
            .filter(|e| e.name != "base")
            .collect();
        others.sort_by(|a, b| a.name.cmp(&b.name));

        for entry in others {
            entries.push(InternalModEntry {
                name: entry.name.clone(),
                enabled: entry.enabled,
            });
        }

        let internal = InternalModList { mods: entries };
        let json = serde_json::to_string_pretty(&internal)?;
        std::fs::write(&file_path, json)
            .with_context(|| format!("Failed to write {}", file_path.display()))?;

        Ok(())
    }

    pub fn is_enabled(&self, name: &str) -> bool {
        self.mods.get(name).map(|e| e.enabled).unwrap_or(false)
    }

    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(entry) = self.mods.get_mut(name) {
            entry.enabled = enabled;
        } else {
            self.mods.insert(
                name.to_string(),
                ModEntry {
                    name: name.to_string(),
                    enabled,
                },
            );
        }
    }

    pub fn remove(&mut self, name: &str) {
        self.mods.remove(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let mut list = ModList::new();
        list.set_enabled("some-mod", true);
        list.set_enabled("other-mod", false);

        list.save(dir_path).unwrap();

        let loaded = ModList::load_or_create(dir_path).unwrap();

        assert!(loaded.is_enabled("base"));
        assert!(loaded.is_enabled("some-mod"));
        assert!(!loaded.is_enabled("other-mod"));
        assert!(!loaded.is_enabled("nonexistent"));
    }

    #[test]
    fn load_nonexistent_creates_default() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let list = ModList::load_or_create(dir_path).unwrap();
        assert!(list.is_enabled("base"));
        assert_eq!(list.mods.len(), 1);
    }

    #[test]
    fn save_format_matches_factorio() {
        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_str().unwrap();

        let mut list = ModList::new();
        list.set_enabled("test-mod", true);
        list.save(dir_path).unwrap();

        let contents = fs::read_to_string(dir.path().join("mod-list.json")).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();

        assert!(parsed["mods"].is_array());
        let mods = parsed["mods"].as_array().unwrap();
        assert_eq!(mods[0]["name"], "base");
        assert_eq!(mods[0]["enabled"], true);
    }
}
