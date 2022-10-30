use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mods {
    pub results: Vec<Mod>,
}

pub enum SortBy {
    Downloads,
}

impl Mods {
    pub fn sort(&mut self, sort_by: SortBy) {
        match sort_by {
            SortBy::Downloads => self
                .results
                .sort_by(|a, b| b.downloads_count.cmp(&a.downloads_count)),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub description: Option<String>,
    #[serde(rename = "downloads_count")]
    pub downloads_count: i64,
    pub category: Option<String>,
    pub full: Option<bool>,
}

pub async fn get_mods(sort_by: Option<SortBy>) -> Result<Vec<Mod>, Box<dyn std::error::Error>> {
    let url = "https://mods.factorio.com/api/mods?page_size=max";
    let mut mods = reqwest::get(url).await?.json::<Mods>().await?;
    let sort_by = sort_by.unwrap_or(SortBy::Downloads);
    mods.sort(sort_by);
    Ok(mods.results)
}

pub async fn get_mod(name: &str) -> Result<Mod, reqwest::Error> {
    let url = format!("https://mods.factorio.com/api/mods/{}/full", name);
    let mut response = reqwest::get(url).await?.json::<Mod>().await?;
    response.full = Some(true);
    Ok(response)
}
