use log::info;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModsConfig {
    pub mods: Vec<Mod>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub name: String,
    pub enabled: bool,
}

pub fn get_mods_config(mods_config_path: &str) -> Result<ModsConfig, Box<dyn std::error::Error>> {
    let mods_config = std::fs::read_to_string(mods_config_path)?;
    let mods_config: ModsConfig = serde_json::from_str(&mods_config)?;
    Ok(mods_config)
}
