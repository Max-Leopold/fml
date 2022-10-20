use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub token: String,
    pub visibility: Visibility,
    #[serde(rename = "game_password")]
    pub game_password: String,
    pub description: String,
    pub name: String,
    pub username: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Visibility {
    pub public: bool,
}

pub fn get_server_config(server_config_path: &str) -> Result<Root, Box<dyn std::error::Error>> {
    let server_config = std::fs::read_to_string(server_config_path)?;
    let server_config: Root = serde_json::from_str(&server_config)?;
    Ok(server_config)
}
