use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use std::str::FromStr;
use std::sync::LazyLock;

// --- Mod Portal types ---

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub downloads_count: u64,
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub download_url: String,
    pub file_name: String,
    pub version: semver::Version,
    pub factorio_version: String,
    pub sha1: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_req: semver::VersionReq,
    pub dep_type: DependencyType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
}

static DEP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*(\(\?\)|[!?~])?\s*([a-zA-Z0-9_-][a-zA-Z0-9_ -]*[a-zA-Z0-9_-]|[a-zA-Z0-9_-])\s*(?:([<>=!]+)\s*(\d+\.\d+\.\d+))?\s*$")
        .unwrap()
});

impl FromStr for Dependency {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let caps = DEP_REGEX
            .captures(s)
            .with_context(|| format!("Invalid dependency string: '{}'", s))?;

        let prefix = caps.get(1).map(|m| m.as_str());
        let name = caps[2].trim().to_string();

        let dep_type = match prefix {
            None => DependencyType::Required,
            Some("~") => DependencyType::Required,
            Some("?") => DependencyType::Optional,
            Some("(?)") => DependencyType::Optional,
            Some("!") => DependencyType::Incompatible,
            Some(other) => bail!("Unknown dependency prefix: '{}'", other),
        };

        let version_req = match (caps.get(3), caps.get(4)) {
            (Some(op), Some(ver)) => {
                let op_str = op.as_str();
                let ver_str = ver.as_str();
                let req_str = if op_str == "=" {
                    format!("={}", ver_str)
                } else {
                    format!("{}{}", op_str, ver_str)
                };
                semver::VersionReq::parse(&req_str)
                    .with_context(|| format!("Invalid version requirement: '{}'", req_str))?
            }
            _ => semver::VersionReq::STAR,
        };

        Ok(Dependency {
            name,
            version_req,
            dep_type,
        })
    }
}

// --- Mod list entry (from GET /api/mods) ---

#[derive(Debug, Clone)]
pub struct ModListEntry {
    pub name: String,
    pub title: String,
    pub downloads_count: u64,
    pub summary: String,
}

// --- Server settings ---

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub token: String,
}

pub fn read_server_settings(path: &str) -> Result<ServerSettings> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read server settings from: {}", path))?;
    let settings: ServerSettings = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse server settings: {}", path))?;

    if settings.username.is_empty() || settings.token.is_empty() {
        bail!(
            "Valid Factorio credentials (username and token) are required in {}. \
             Both 'username' and 'token' fields must be non-empty.",
            path
        );
    }

    Ok(settings)
}

// --- Factorio version detection ---

#[derive(Debug, Deserialize)]
struct BaseInfoJson {
    version: String,
}

pub fn detect_factorio_version(mods_dir: &str) -> Result<String> {
    let info_path = Path::new(mods_dir)
        .join("..")
        .join("data")
        .join("base")
        .join("info.json");

    let contents = std::fs::read_to_string(&info_path).with_context(|| {
        format!(
            "Failed to read Factorio base info.json at: {}. \
             Expected standard Factorio server layout with data/base/info.json \
             adjacent to the mods directory.",
            info_path.display()
        )
    })?;

    let info: BaseInfoJson = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse {}", info_path.display()))?;

    let parts: Vec<&str> = info.version.split('.').collect();
    if parts.len() < 2 {
        bail!(
            "Invalid version format in {}: '{}'",
            info_path.display(),
            info.version
        );
    }

    Ok(format!("{}.{}", parts[0], parts[1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_required_no_version() {
        let dep: Dependency = "base".parse().unwrap();
        assert_eq!(dep.name, "base");
        assert_eq!(dep.dep_type, DependencyType::Required);
        assert_eq!(dep.version_req, semver::VersionReq::STAR);
    }

    #[test]
    fn parse_required_with_version() {
        let dep: Dependency = "base >= 2.0.0".parse().unwrap();
        assert_eq!(dep.name, "base");
        assert_eq!(dep.dep_type, DependencyType::Required);
        assert!(dep.version_req.matches(&semver::Version::new(2, 0, 0)));
        assert!(!dep.version_req.matches(&semver::Version::new(1, 9, 0)));
    }

    #[test]
    fn parse_incompatible() {
        let dep: Dependency = "! incompatible-mod".parse().unwrap();
        assert_eq!(dep.name, "incompatible-mod");
        assert_eq!(dep.dep_type, DependencyType::Incompatible);
    }

    #[test]
    fn parse_optional() {
        let dep: Dependency = "? quality".parse().unwrap();
        assert_eq!(dep.name, "quality");
        assert_eq!(dep.dep_type, DependencyType::Optional);
    }

    #[test]
    fn parse_hidden_optional() {
        let dep: Dependency = "(?) hidden-lib".parse().unwrap();
        assert_eq!(dep.name, "hidden-lib");
        assert_eq!(dep.dep_type, DependencyType::Optional);
    }

    #[test]
    fn parse_no_load_order() {
        let dep: Dependency = "~ some-mod >= 1.0.0".parse().unwrap();
        assert_eq!(dep.name, "some-mod");
        assert_eq!(dep.dep_type, DependencyType::Required);
        assert!(dep.version_req.matches(&semver::Version::new(1, 0, 0)));
        assert!(dep.version_req.matches(&semver::Version::new(2, 0, 0)));
        assert!(!dep.version_req.matches(&semver::Version::new(0, 9, 0)));
    }

    #[test]
    fn parse_exact_version() {
        let dep: Dependency = "some-mod = 1.2.3".parse().unwrap();
        assert_eq!(dep.name, "some-mod");
        assert_eq!(dep.dep_type, DependencyType::Required);
        assert!(dep.version_req.matches(&semver::Version::new(1, 2, 3)));
        assert!(!dep.version_req.matches(&semver::Version::new(1, 2, 4)));
    }

    #[test]
    fn parse_less_than() {
        let dep: Dependency = "some-mod < 2.0.0".parse().unwrap();
        assert_eq!(dep.name, "some-mod");
        assert!(dep.version_req.matches(&semver::Version::new(1, 9, 9)));
        assert!(!dep.version_req.matches(&semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn parse_mod_name_with_spaces() {
        let dep: Dependency = "? Krastorio 2".parse().unwrap();
        assert_eq!(dep.name, "Krastorio 2");
        assert_eq!(dep.dep_type, DependencyType::Optional);
    }
}
