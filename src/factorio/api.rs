use std::cmp::min;
use std::error::Error;
use std::fs::File;
use std::io::Write;

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
    #[serde(rename = "latest_release")]
    pub latest_release: Option<Release>,
    pub releases: Option<Vec<Release>>,
    pub full: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    #[serde(rename = "download_url")]
    pub download_url: String,
    #[serde(rename = "file_name")]
    pub file_name: String,
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
    response.latest_release = Some(response.releases.as_ref().unwrap().last().unwrap().clone());
    Ok(response)
}

pub async fn download_mod<F: Fn(u16)>(
    name: &str,
    username: &str,
    token: &str,
    dir: &str,
    f: Option<F>,
) -> Result<File, Box<dyn Error>> {
    let mod_ = get_mod(name).await?;
    let url = format!(
        "https://mods.factorio.com{}?username={}&token={}",
        mod_.latest_release.as_ref().unwrap().download_url,
        username,
        token
    );
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap();
    let mut downloaded: usize = 0;
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(format!(
            "{}/{}",
            dir,
            mod_.latest_release.as_ref().unwrap().file_name
        ))?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
        downloaded = min(downloaded + (chunk.len() as usize), total_size as usize);
        let downloaded_percent = ((downloaded as f64 / total_size as f64) * 100.0) as u16;
        if let Some(ref f) = f {
            f(downloaded_percent);
        }
    }

    Ok(file)
}
