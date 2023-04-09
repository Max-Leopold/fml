use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;

use super::api;

#[derive(Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub downloads_count: i64,
    pub releases: BTreeMap<semver::Version, Release>,
}

#[derive(Debug, Clone)]
pub struct Release {
    pub download_url: String,
    pub file_name: String,
    pub version: semver::Version,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_req: semver::VersionReq,
    pub dep_type: DependencyType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
}

impl Mod {
    pub fn release(&self, version: &semver::Version) -> Option<Release> {
        self.releases.get(version).cloned()
    }

    pub fn latest_release(&mut self) -> Option<Release> {
        self.releases.values().rev().cloned().next()
    }

    pub async fn download_version<F: Fn(u16)>(
        &mut self,
        ver_req: semver::VersionReq,
        username: &str,
        token: &str,
        dir: &str,
        f: Option<F>,
    ) -> Result<File, Box<dyn Error>> {
        let release = self.find_matching_release(&ver_req);
        if release.is_none() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No matching releases found",
            )));
        }

        let release = release.unwrap();
        let file = api::download_release(&release, username, token, dir, f).await?;

        Ok(file)
    }

    pub fn find_matching_release(&self, ver_req: &semver::VersionReq) -> Option<Release> {
        let entry = self
            .releases
            .iter()
            .rev()
            .find(|(version, release)| ver_req.matches(version));

        if let Some((version, release)) = entry {
            Some(release.clone())
        } else {
            None
        }
    }
}

impl Release {
    pub fn required_dependencies(&self) -> Vec<Dependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.dep_type == DependencyType::Required)
            .cloned()
            .collect()
    }

    pub fn optional_dependencies(&self) -> Vec<Dependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.dep_type == DependencyType::Optional)
            .cloned()
            .collect()
    }

    pub fn incompatible_dependencies(&self) -> Vec<Dependency> {
        self.dependencies
            .iter()
            .filter(|dep| dep.dep_type == DependencyType::Incompatible)
            .cloned()
            .collect()
    }
}
