use core::fmt;
use fs2::FileExt;
use lazy_static::lazy_static;
use regex::Regex;
use std::cmp::min;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::skip_none;

#[derive(Default)]
pub struct Registry {
    all_mod_identifiers: Option<Vec<ModIdentifier>>,
    mods: HashMap<ModIdentifier, Mod>,
}

lazy_static! {
    pub static ref REGISTRY: Mutex<Registry> = Mutex::new(Registry::default());
    // See https://wiki.factorio.com/Tutorial:Mod_structure#dependencies for the dependency string format
    static ref DEPDENDECY_REGEX: Regex = Regex::new(r"^(?P<prefix>[!?~()]*)\s*(?P<mod_name>\S[^>=<]+)\s*(?P<equality>[=<>]+)?\s*(?P<version>\S+)?\s*$").unwrap();
}

impl Registry {
    pub async fn load_mod_identifiers() -> Result<Vec<ModIdentifier>, Box<dyn std::error::Error>> {
        if let Some(all_mod_identifiers) = REGISTRY.lock().unwrap().get_mod_identifiers() {
            return Ok(all_mod_identifiers);
        }

        let url = "https://mods.factorio.com/api/mods?page_size=max";
        let mut mod_identifiers = reqwest::get(url).await?.json::<ModIdentifiers>().await?.results;
        mod_identifiers.sort_by(|a, b| b.downloads_count.cmp(&a.downloads_count));

        REGISTRY
            .lock()
            .unwrap()
            .set_mod_identifiers(mod_identifiers.clone());

        Ok(mod_identifiers)
    }

    fn set_mod_identifiers(&mut self, mod_identifiers: Vec<ModIdentifier>) {
        self.all_mod_identifiers = Some(mod_identifiers);
    }

    pub fn get_mod_identifiers(&self) -> Option<Vec<ModIdentifier>> {
        self.all_mod_identifiers.clone()
    }

    pub async fn load_mod(
        mod_identifier: &ModIdentifier,
    ) -> Result<Mod, Box<dyn std::error::Error>> {
        if let Some(mod_) = REGISTRY.lock().unwrap().mods.get(mod_identifier) {
            return Ok(mod_.clone());
        }
        let url = format!(
            "https://mods.factorio.com/api/mods/{}/full",
            mod_identifier.name
        );
        let mod_ = reqwest::get(url).await?.json::<Mod>().await?;
        REGISTRY.lock().unwrap().add_mod(mod_.clone());
        Ok(mod_)
    }

    fn add_mod(&mut self, mod_: Mod) {
        self.mods.insert(
            ModIdentifier {
                name: mod_.name.clone(),
                title: mod_.title.clone(),
                downloads_count: mod_.downloads_count,
            },
            mod_,
        );
    }

    pub fn get_mod(&self, mod_identifier: &ModIdentifier) -> Option<Mod> {
        if self.mods.contains_key(mod_identifier) {
            return Some(self.mods.get(mod_identifier).unwrap().clone());
        }

        None
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ModIdentifiers {
    results: Vec<ModIdentifier>,
}

#[derive(Debug, Default, Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct ModIdentifier {
    pub name: String,
    pub title: String,
    #[serde(rename = "downloads_count")]
    downloads_count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mods {
    results: Vec<Mod>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub description: String,
    #[serde(rename = "downloads_count")]
    pub downloads_count: i64,
    pub releases: Vec<Release>,
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
    pub equality: Option<Equality>,
    pub version: Option<Version>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Equality {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
}

#[derive(Default, Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
}

impl ModIdentifier {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

impl Dependency {
    pub fn get_max_and_min_version(&self) -> (Option<Version>, Option<Version>) {
        if self.version.is_none() {
            return (None, None);
        }

        match self.equality {
            Some(Equality::GreaterThan) => {
                let mut version = self.version.clone().unwrap();
                version.patch += 1;
                (Some(version), None)
            }
            Some(Equality::GreaterThanOrEqual) => (Some(self.version.clone().unwrap()), None),
            Some(Equality::LessThan) => {
                let mut version = self.version.clone().unwrap();
                version.patch -= 1;
                (None, Some(version))
            }
            Some(Equality::LessThanOrEqual) => (None, Some(self.version.clone().unwrap())),
            Some(Equality::Equal) => (
                Some(self.version.clone().unwrap()),
                Some(self.version.clone().unwrap()),
            ),
            None => (None, None),
        }
    }
}

impl Equality {
    pub fn from_str(s: &str) -> Option<Equality> {
        match s {
            ">" => Some(Equality::GreaterThan),
            ">=" => Some(Equality::GreaterThanOrEqual),
            "<" => Some(Equality::LessThan),
            "<=" => Some(Equality::LessThanOrEqual),
            "=" => Some(Equality::Equal),
            _ => None,
        }
    }

    pub fn to_str(e: Option<Equality>) -> &'static str {
        match e {
            Some(Equality::GreaterThan) => ">",
            Some(Equality::GreaterThanOrEqual) => ">=",
            Some(Equality::LessThan) => "<",
            Some(Equality::LessThanOrEqual) => "<=",
            Some(Equality::Equal) => "=",
            None => "",
        }
    }
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
        let captures = skip_none!(DEPDENDECY_REGEX.captures(&dependency));
        let mod_name = skip_none!(captures.name("mod_name"));
        let mod_name = mod_name.as_str().trim();

        let prefix = captures.name("prefix");
        let prefix = match prefix {
            Some(prefix) => Some(prefix.as_str().trim()),
            None => None,
        };
        let equality = captures.name("equality");
        let equality = match equality {
            Some(equality) => Equality::from_str(equality.as_str().trim()),
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
    pub fn latest_release(&mut self) -> Option<Release> {
        self.releases.sort_by(|a, b| b.version.cmp(&a.version));
        self.releases.first().cloned()
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
        self.releases.sort_by(|a, b| b.version.cmp(&a.version));

        let mut latest_version = None;
        for release in &self.releases {
            if max_version.is_some() && release.version > *max_version.unwrap() {
                continue;
            } else if min_version.is_some() && release.version < *min_version.unwrap() {
                break;
            } else {
                latest_version = Some(release);
                break;
            }
        }
        Ok(latest_version.cloned())
    }
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
