use core::fmt;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::min;
use std::error::Error;
use std::fmt::Display;
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
    #[serde(rename = "info_json")]
    pub info_json: InfoJson,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoJson {
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_dependencies")]
    pub dependencies: Option<Dependencies>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependencies {
    pub required: Vec<Dependency>,
    pub optional: Vec<Dependency>,
    pub incompatible: Vec<Dependency>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub equality: Option<String>,
    pub version: Option<Version>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    pub fn from_str(version: &str) -> Version {
        let mut version_parts = version.split('.');
        let major = version_parts
            .next()
            .and_then(|part| part.parse::<i64>().ok())
            .unwrap_or(0);
        let minor = version_parts
            .next()
            .and_then(|part| part.parse::<i64>().ok())
            .unwrap_or(0);
        let patch = version_parts
            .next()
            .and_then(|part| part.parse::<i64>().ok())
            .unwrap_or(0);

        Version {
            major,
            minor,
            patch,
        }
    }
}

fn deserialize_dependencies<'de, D>(deserializer: D) -> Result<Option<Dependencies>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let dependencies: Option<Vec<String>> = Option::deserialize(deserializer)?;
    if dependencies.is_none() {
        return Ok(None);
    }
    let dependencies = dependencies.unwrap();

    let mut required_dependencies = Vec::new();
    let mut optional_dependencies = Vec::new();
    let mut incompatible_dependencies = Vec::new();
    for dependency in dependencies {
        // See https://wiki.factorio.com/Tutorial:Mod_structure#dependencies for the dependency string format
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(?P<prefix>[!?~()]*)\s*(?P<mod_name>\S[^>=<]+)\s*(?P<equality>[=<>]+)?\s*(?P<version>\S+)?\s*$").unwrap();
        }
        let captures = RE.captures(&dependency).unwrap();
        let mod_name = captures.name("mod_name").unwrap().as_str().trim();

        let prefix = captures.name("prefix");
        let prefix = match prefix {
            Some(prefix) => Some(prefix.as_str().trim()),
            None => None,
        };
        let equality = captures.name("equality");
        let equality = match equality {
            Some(equality) => Some(equality.as_str().trim().to_string()),
            None => None,
        };
        let version = captures.name("version");
        let version = match version {
            Some(version) => Some(Version::from_str(version.as_str().trim())),
            None => None,
        };

        let dependency = Dependency {
            name: mod_name.to_string(),
            equality,
            version,
        };

        match prefix {
            Some("!") => {
                incompatible_dependencies.push(dependency);
            }
            Some("?") | Some("(?)") => {
                optional_dependencies.push(dependency);
            }
            _ => {
                required_dependencies.push(dependency);
            }
        }
    }

    Ok(Some(Dependencies {
        required: required_dependencies,
        optional: optional_dependencies,
        incompatible: incompatible_dependencies,
    }))
}

impl Mod {
    pub fn latest_release(&self) -> Release {
        if let Some(true) = self.full {
            return self.releases.as_ref().unwrap().last().unwrap().clone();
        }

        self.latest_release.as_ref().unwrap().clone()
    }
}

pub async fn get_mods(sort_by: Option<SortBy>) -> Result<Vec<Mod>, Box<dyn std::error::Error>> {
    let url = "https://mods.factorio.com/api/mods?page_size=max";
    let mut mods = reqwest::get(url).await?.json::<Mods>().await.unwrap();
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
    mod_: &Mod,
    username: &str,
    token: &str,
    dir: &str,
    f: Option<F>,
) -> Result<File, Box<dyn Error>> {
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
