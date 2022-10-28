use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct ModList {
    mods: HashMap<String, ModEntry>,
    file_path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InternalModsConfig {
    pub mods: Vec<ModEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModEntry {
    pub name: String,
    pub enabled: bool,
}

impl ModList {
    pub fn load_or_create(mods_dir_path: &str) -> Result<ModList, Box<dyn std::error::Error>> {
        let file_path = Path::new(mods_dir_path).join("mod-list.json");
        if file_path.exists() {
            let mods = std::fs::read_to_string(&file_path)?;
            let internal_mod_config: InternalModsConfig = match serde_json::from_str(&mods) {
                Ok(mods) => mods,
                Err(err) => return Err(Box::new(err)),
            };
            let map = internal_mod_config
                .mods
                .into_iter()
                .map(|mod_entry| (mod_entry.name.clone(), mod_entry))
                .collect();
            let config = ModList {
                file_path: file_path.to_str().unwrap().to_string(),
                mods: map,
            };
            Ok(config)
        } else {
            let mut map = HashMap::new();
            // Insert the base mod, which is always enabled and present
            map.insert(
                "base".to_string(),
                ModEntry {
                    name: "base".to_string(),
                    enabled: true,
                },
            );
            let config = ModList {
                file_path: file_path.to_str().unwrap().to_string(),
                mods: HashMap::new(),
            };
            config.save()?;
            Ok(config)
        }
    }

    pub fn mod_is_enabled(&mut self, name: &str) -> bool {
        if let Some(mod_config) = self.mods.get(name) {
            mod_config.enabled
        } else {
            false
        }
    }

    pub fn set_mod_enabled(&mut self, name: &str, enabled: bool) {
        if let Some(mod_config) = self.mods.get_mut(name) {
            mod_config.enabled = enabled;
        } else {
            self.mods.insert(
                name.to_string(),
                ModEntry {
                    name: name.to_string(),
                    enabled,
                },
            );
        }
        self.save().unwrap();
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let internal_mod_config = InternalModsConfig {
            mods: self
                .mods
                .values()
                .map(|mod_config| ModEntry {
                    name: mod_config.name.clone(),
                    enabled: mod_config.enabled,
                })
                .collect(),
        };
        let json = serde_json::to_string_pretty(&internal_mod_config)?;
        std::fs::write(&self.file_path, json)?;
        Ok(())
    }
}
