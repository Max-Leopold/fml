use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct ModList {
    pub mods: HashMap<String, ModEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InternalModList {
    pub mods: Vec<ModEntry>,
}

#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModEntry {
    pub name: String,
    pub enabled: bool,
    pub version: Option<String>,
}

impl ModList {
    pub fn new() -> ModList {
        let mut mods = HashMap::new();
        mods.insert(
            "base".to_string(),
            ModEntry {
                name: "base".to_string(),
                enabled: true,
                version: None,
            },
        );

        ModList { mods }
    }

    pub fn load_or_create(mods_dir_path: &str) -> Result<ModList, Box<dyn std::error::Error>> {
        let file_path = Path::new(mods_dir_path).join("mod-list.json");
        if file_path.exists() {
            let mods = std::fs::read_to_string(&file_path)?;
            let internal_mod_config: InternalModList = serde_json::from_str(&mods)?;
            let map = internal_mod_config
                .mods
                .into_iter()
                .map(|mod_entry| (mod_entry.name.clone(), mod_entry))
                .collect();
            let config = ModList {
                mods: map,
            };
            return Ok(config);
        } else {
            let config = ModList::new();
            return Ok(config);
        }
    }

    pub fn save(&self, mods_dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let internal_mod_config = InternalModList {
            mods: self
                .mods
                .values()
                .map(|mod_config| ModEntry {
                    name: mod_config.name.clone(),
                    enabled: mod_config.enabled,
                    version: mod_config.version.clone(),
                })
                .collect(),
        };
        let json = serde_json::to_string_pretty(&internal_mod_config)?;
        let path = Path::new(mods_dir_path).join("mod-list.json");
        std::fs::write(path, json)?;
        Ok(())
    }
}
