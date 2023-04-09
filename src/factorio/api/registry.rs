use crate::factorio::modification::{Dependency, DependencyType, Mod, Release};
use lazy_static::lazy_static;
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use std::sync::Mutex;

#[derive(Debug, Default, Eq, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub struct ModIdentifier {
    pub name: String,
    pub title: String,
    downloads_count: i64,
}

#[derive(Default)]
pub struct Registry {
    all_mod_identifiers: Option<Vec<ModIdentifier>>,
    mods: HashMap<String, Mod>,
}

lazy_static! {
  static ref REGISTRY: Mutex<Registry> = Mutex::new(Registry::default());
  // See https://wiki.factorio.com/Tutorial:Mod_structure#dependencies for the dependency string format
  static ref DEPDENDECY_REGEX: Regex = Regex::new(r"^(?P<prefix>[!?~(]*)(?P<name>[A-Za-z0-9_-]+)(?P<remaining>.*)$").unwrap();
}

impl Registry {
    pub async fn mod_identifiers() -> Result<Vec<ModIdentifier>, Box<dyn std::error::Error>> {
        if let Some(all_mod_identifiers) = REGISTRY.lock().unwrap().get_mod_identifiers() {
            return Ok(all_mod_identifiers);
        }

        let url = "https://mods.factorio.com/api/mods?page_size=max";
        let mut mod_identifiers = reqwest::get(url)
            .await?
            .json::<JsonModIdentifiers>()
            .await?
            .results;
        mod_identifiers.sort_by(|a, b| b.downloads_count.cmp(&a.downloads_count));

        REGISTRY
            .lock()
            .unwrap()
            .set_mod_identifiers(mod_identifiers.clone());

        Ok(mod_identifiers)
    }

    pub fn get_mod(name: &str) -> Option<Mod> {
        REGISTRY.lock().unwrap().mods.get(name).cloned()
    }

    pub async fn load_mod(name: &str) -> Result<Mod, Box<dyn std::error::Error>> {
        if let Some(mod_) = Self::get_mod(name) {
            return Ok(mod_.clone());
        }
        let url = format!("https://mods.factorio.com/api/mods/{}/full", name);
        let json_mod = reqwest::get(url).await?.json::<JsonMod>().await?;
        let mod_ = Mod::from(json_mod);
        REGISTRY.lock().unwrap().add_mod(mod_.clone());
        Ok(mod_)
    }

    fn set_mod_identifiers(&mut self, mod_identifiers: Vec<ModIdentifier>) {
        self.all_mod_identifiers = Some(mod_identifiers);
    }

    fn get_mod_identifiers(&self) -> Option<Vec<ModIdentifier>> {
        self.all_mod_identifiers.clone()
    }

    fn add_mod(&mut self, mod_: Mod) {
        self.mods.insert(mod_.name.clone(), mod_);
    }
}

// Private

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JsonModIdentifiers {
    results: Vec<ModIdentifier>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JsonMod {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub description: Option<String>,
    pub downloads_count: i64,
    pub releases: Vec<JsonRelease>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JsonRelease {
    pub download_url: String,
    pub file_name: String,
    pub info_json: JsonInfo,
    pub version: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JsonInfo {
    pub dependencies: Option<Vec<String>>,
}

impl From<JsonMod> for Mod {
  fn from(json_mod: JsonMod) -> Self {
      let releases = json_mod
          .releases
          .into_iter()
          .filter_map(|json_release| {
              let version = Version::parse(&json_release.version).ok()?;
              let release = Release::from(json_release);
              Some((version, release))
          })
          .collect();

      Self {
          name: json_mod.name,
          title: json_mod.title,
          summary: json_mod.summary,
          description: json_mod.description.unwrap_or_default(),
          downloads_count: json_mod.downloads_count,
          releases,
      }
  }
}

impl From<JsonRelease> for Release {
  fn from(json_release: JsonRelease) -> Self {
      let version = Version::parse(&json_release.version).unwrap();
      let dependencies = json_release
          .info_json
          .dependencies
          .unwrap_or_default()
          .iter()
          .filter_map(|dep_string| Dependency::from_str(dep_string).ok())
          .collect();

      Self {
          download_url: json_release.download_url,
          file_name: json_release.file_name,
          version,
          dependencies,
      }
  }
}

impl FromStr for Dependency {
  type Err = String;

  fn from_str(dep_string: &str) -> Result<Self, Self::Err> {
      let (dep_type, name, remaining) = parse_dep_type_and_name(dep_string)?;

      let version_req = if remaining.is_empty() || dep_type == DependencyType::Incompatible {
          VersionReq::parse("*").map_err(|e| e.to_string())?
      } else {
          VersionReq::parse(remaining).map_err(|e| e.to_string())?
      };

      Ok(Self {
          name,
          version_req,
          dep_type,
      })
  }
}

fn parse_dep_type_and_name(input: &str) -> Result<(DependencyType, String, &str), String> {
  let caps = DEPDENDECY_REGEX.captures(input).ok_or_else(|| format!("Invalid dependency format: {}", input))?;

  let prefix = caps.name("prefix").unwrap().as_str();
  let name = caps.name("name").unwrap().as_str().to_string();
  let remaining = caps.name("remaining").unwrap().as_str();

  let dep_type = match prefix {
      "!" => DependencyType::Incompatible,
      "?" => DependencyType::Optional,
      "(?)" => DependencyType::Optional, // Hidden optional dependency is treated as a regular optional dependency
      "~" => DependencyType::Required, // Dependencies that do not affect load order are treated as required dependencies
      _ => DependencyType::Required,
  };

  Ok((dep_type, name, remaining))
}
