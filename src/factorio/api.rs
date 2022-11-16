use core::fmt;
use fs2::FileExt;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::{min, Ordering};
use std::error::Error;
use std::fs::File;
use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::skip_none;

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
    #[serde(deserialize_with = "deserialize_version")]
    pub version: Version,
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

#[derive(Default, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
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

fn deserialize_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = String::deserialize(deserializer)?;
    Ok(Version::from_str(&version))
}

fn deserialize_dependencies<'de, D>(deserializer: D) -> Result<Option<Dependencies>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let dependencies: Option<Vec<String>> = Option::deserialize(deserializer)?;
    if dependencies.is_none() {
        return Ok(None);
    }
    let dependencies = dependencies.unwrap_or(Vec::new());

    let mut required_dependencies = Vec::new();
    let mut optional_dependencies = Vec::new();
    let mut incompatible_dependencies = Vec::new();
    for dependency in dependencies {
        // See https://wiki.factorio.com/Tutorial:Mod_structure#dependencies for the dependency string format
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^(?P<prefix>[!?~()]*)\s*(?P<mod_name>\S[^>=<]+)\s*(?P<equality>[=<>]+)?\s*(?P<version>\S+)?\s*$").unwrap();
        }
        let captures = skip_none!(RE.captures(&dependency));
        let mod_name = skip_none!(captures.name("mod_name"));
        let mod_name = mod_name.as_str().trim();

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
    pub fn latest_release(&self) -> Option<Release> {
        match self.latest_release {
            Some(ref latest_release) => Some(latest_release.clone()),
            None => match self.releases {
                Some(ref releases) => releases.first().cloned(),
                None => None,
            },
        }
    }

    pub async fn download_version<F: Fn(u16)>(
        &mut self,
        min_version: Option<&Version>,
        max_version: Option<&Version>,
        username: &str,
        token: &str,
        dir: &str,
        f: Option<F>,
    ) -> Result<File, Box<dyn Error>> {
        self.load_fully().await?;

        let release = self.find_release(min_version, max_version).await?;
        if release.is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No matching releases found",
            )));
        }

        let release = release.unwrap();
        let file = download_release(&release, username, token, dir, f).await?;

        Ok(file)
    }

    pub async fn find_release(
        &mut self,
        min_version: Option<&Version>,
        max_version: Option<&Version>,
    ) -> Result<Option<Release>, Box<dyn Error>> {
        self.load_fully().await?;

        if self.releases.is_none() {
            return Ok(None);
        }

        let mut releases = self.releases.as_ref().unwrap_or(&Vec::new()).clone();
        releases.sort_by(|a, b| b.version.cmp(&a.version));

        let mut latest_version = None;
        for release in releases {
            if max_version.is_some() && release.version > *max_version.unwrap() {
                continue;
            } else if min_version.is_some() && release.version < *min_version.unwrap() {
                break;
            } else {
                latest_version = Some(release);
                break;
            }
        }
        Ok(latest_version)
    }

    async fn load_fully(&mut self) -> Result<(), Box<dyn Error>> {
        if self.full.unwrap_or(false) {
            return Ok(());
        }
        _ = std::mem::replace(self, get_mod(&self.name).await?);
        Ok(())
    }
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

pub async fn download_release<F: Fn(u16)>(
    release: &Release,
    username: &str,
    token: &str,
    dir: &str,
    f: Option<F>,
) -> Result<File, Box<dyn Error>> {
    let url = format!(
        "https://mods.factorio.com{}?username={}&token={}",
        release.download_url, username, token
    );
    let client = reqwest::Client::new();
    let mut response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(1);
    let mut downloaded: usize = 0;
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(format!("{}/{}", dir, release.file_name))?;

    file.lock_exclusive()?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
        downloaded = min(downloaded + (chunk.len() as usize), total_size as usize);
        let downloaded_percent = ((downloaded as f64 / total_size as f64) * 100.0) as u16;
        if let Some(ref f) = f {
            f(downloaded_percent);
        }
    }

    file.unlock()?;

    Ok(file)
}

pub async fn download_mod<F: Fn(u16)>(
    mod_: &Mod,
    username: &str,
    token: &str,
    dir: &str,
    f: Option<F>,
) -> Result<File, Box<dyn Error>> {
    let latest_release = mod_.latest_release();
    if latest_release.is_none() {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No releases found",
        )));
    }

    let latest_release = latest_release.unwrap();
    download_release(&latest_release, username, token, dir, f).await
}
