
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mods {
    pub results: Vec<Mod>,
}

pub enum SortBy {
    Downloads,
}

impl Mods {
    pub fn sort(&mut self, sort_by: SortBy) {
        match sort_by {
            SortBy::Downloads => self.results.sort_by(|a, b| b.downloads_count.cmp(&a.downloads_count)),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub name: String,
    pub title: String,
    pub owner: String,
    pub summary: String,
    #[serde(rename = "downloads_count")]
    pub downloads_count: i64,
    pub category: Option<String>,
    pub score: f64,
    #[serde(rename = "latest_release")]
    pub latest_release: Option<LatestRelease>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestRelease {
    #[serde(rename = "download_url")]
    pub download_url: String,
    #[serde(rename = "file_name")]
    pub file_name: String,
    #[serde(rename = "info_json")]
    pub info_json: InfoJson,
    #[serde(rename = "released_at")]
    pub released_at: String,
    pub version: String,
    pub sha1: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoJson {
    #[serde(rename = "factorio_version")]
    pub factorio_version: String,
}

pub fn get_mods() -> Result<Mods, Box<dyn std::error::Error>> {
    let url = "https://mods.factorio.com/api/mods?page_size=max";
    let body = reqwest::blocking::get(url)?.text()?;
    let mods: Mods = serde_json::from_str(&body)?;
    Ok(mods)
}


