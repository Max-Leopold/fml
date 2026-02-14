use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::future::Future;

use super::types::{DependencyType, Mod, Release};

#[derive(Debug)]
pub struct ResolveResult {
    pub to_download: Vec<(String, Release)>,
}

/// Resolve all dependencies for a mod before downloading anything.
///
/// The `fetch_fn` parameter makes this testable without hitting the network.
/// It takes a mod name and returns the full Mod details.
pub async fn resolve<F, Fut>(
    mod_name: &str,
    factorio_version: &str,
    installed: &HashMap<String, semver::Version>,
    fetch_fn: &F,
) -> Result<ResolveResult>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<Mod>>,
{
    let mut to_download: Vec<(String, Release)> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    resolve_recursive(
        mod_name,
        &semver::VersionReq::STAR,
        factorio_version,
        installed,
        fetch_fn,
        &mut to_download,
        &mut visited,
        mod_name, // root mod for error messages
    )
    .await?;

    Ok(ResolveResult { to_download })
}

#[allow(clippy::too_many_arguments)]
fn resolve_recursive<'a, F, Fut>(
    mod_name: &'a str,
    version_req: &'a semver::VersionReq,
    factorio_version: &'a str,
    installed: &'a HashMap<String, semver::Version>,
    fetch_fn: &'a F,
    to_download: &'a mut Vec<(String, Release)>,
    visited: &'a mut HashSet<String>,
    root_mod: &'a str,
) -> std::pin::Pin<Box<dyn Future<Output = Result<()>> + 'a>>
where
    F: Fn(String) -> Fut,
    Fut: Future<Output = Result<Mod>> + 'a,
{
    Box::pin(async move {
    // Skip base — it's the game itself
    if mod_name == "base" {
        return Ok(());
    }

    // Skip if already visited (cycle/duplicate detection)
    if visited.contains(mod_name) {
        return Ok(());
    }
    visited.insert(mod_name.to_string());

    // Check if already installed
    if let Some(installed_version) = installed.get(mod_name) {
        if version_req.matches(installed_version) {
            return Ok(());
        } else {
            bail!(
                "Installed version {} of mod '{}' does not satisfy required {}. \
                 Remove it first and retry.",
                installed_version,
                mod_name,
                version_req
            );
        }
    }

    // Fetch mod details
    let mod_details = fetch_fn(mod_name.to_string()).await.map_err(|e| {
        anyhow::anyhow!(
            "Failed to fetch dependency '{}' (needed by '{}'): {}",
            mod_name,
            root_mod,
            e
        )
    })?;

    // Find best release: latest that matches factorio_version and version constraint.
    // Releases are iterated newest-first (reverse order).
    let release = mod_details
        .releases
        .iter()
        .rev()
        .find(|r| {
            r.factorio_version == factorio_version && version_req.matches(&r.version)
        })
        .cloned();

    let release = match release {
        Some(r) => r,
        None => {
            bail!(
                "No compatible release found for mod '{}' \
                 (need Factorio version {}, version {})",
                mod_name,
                factorio_version,
                version_req
            );
        }
    };

    // Process dependencies of this release before adding it to the download list
    // (dependency-first order)
    for dep in &release.dependencies {
        match dep.dep_type {
            DependencyType::Optional => {
                // Skip optional dependencies
                continue;
            }
            DependencyType::Incompatible => {
                // Check if the incompatible mod is installed or queued
                if installed.contains_key(&dep.name) {
                    bail!(
                        "Cannot install '{}': it is incompatible with installed mod '{}'",
                        mod_name,
                        dep.name
                    );
                }
                // Also check if it's in the to_download list
                if to_download.iter().any(|(name, _)| name == &dep.name) {
                    bail!(
                        "Cannot install '{}': it is incompatible with mod '{}' \
                         (which is also being installed)",
                        mod_name,
                        dep.name
                    );
                }
            }
            DependencyType::Required => {
                resolve_recursive(
                    &dep.name,
                    &dep.version_req,
                    factorio_version,
                    installed,
                    fetch_fn,
                    to_download,
                    visited,
                    root_mod,
                )
                .await?;
            }
        }
    }

    // Add this mod to the download list (after its dependencies)
    to_download.push((mod_name.to_string(), release));

    Ok(())
    }) // Box::pin
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factorio::types::{Dependency, Mod, Release};
    use std::collections::HashMap;

    fn make_mod(name: &str, deps: Vec<&str>, version: &str, factorio_ver: &str) -> Mod {
        let dependencies: Vec<Dependency> = deps.iter().filter_map(|d| d.parse().ok()).collect();
        Mod {
            name: name.to_string(),
            title: name.to_string(),
            summary: String::new(),
            downloads_count: 0,
            releases: vec![Release {
                download_url: format!("/download/{}", name),
                file_name: format!("{}_{}.zip", name, version),
                version: semver::Version::parse(version).unwrap(),
                factorio_version: factorio_ver.to_string(),
                sha1: String::new(),
                dependencies,
            }],
        }
    }

    fn make_registry(mods: Vec<Mod>) -> HashMap<String, Mod> {
        mods.into_iter().map(|m| (m.name.clone(), m)).collect()
    }

    async fn run_resolve(
        mod_name: &str,
        registry: &HashMap<String, Mod>,
        installed: &HashMap<String, semver::Version>,
    ) -> Result<ResolveResult> {
        let fetch = |name: String| {
            let registry = registry.clone();
            async move {
                registry
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Mod '{}' not found", name))
            }
        };
        resolve(mod_name, "1.1", installed, &fetch).await
    }

    #[tokio::test]
    async fn base_is_skipped() {
        let registry = make_registry(vec![]);
        let installed = HashMap::new();
        let result = run_resolve("base", &registry, &installed).await.unwrap();
        assert!(result.to_download.is_empty());
    }

    #[tokio::test]
    async fn cycle_detection() {
        let registry = make_registry(vec![
            make_mod("mod-a", vec!["mod-b"], "1.0.0", "1.1"),
            make_mod("mod-b", vec!["mod-a"], "1.0.0", "1.1"),
        ]);
        let installed = HashMap::new();
        // Should not infinite loop — cycle is detected via visited set
        let result = run_resolve("mod-a", &registry, &installed).await.unwrap();
        assert_eq!(result.to_download.len(), 2);
        assert_eq!(result.to_download[0].0, "mod-b");
        assert_eq!(result.to_download[1].0, "mod-a");
    }

    #[tokio::test]
    async fn incompatible_blocks_install() {
        let registry = make_registry(vec![make_mod(
            "mod-a",
            vec!["! bad-mod"],
            "1.0.0",
            "1.1",
        )]);
        let mut installed = HashMap::new();
        installed.insert("bad-mod".to_string(), semver::Version::new(1, 0, 0));

        let result = run_resolve("mod-a", &registry, &installed).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("incompatible"));
        assert!(err.contains("bad-mod"));
    }

    #[tokio::test]
    async fn already_installed_is_skipped() {
        let registry = make_registry(vec![make_mod(
            "mod-a",
            vec!["mod-b >= 1.0.0"],
            "1.0.0",
            "1.1",
        )]);
        let mut installed = HashMap::new();
        installed.insert("mod-b".to_string(), semver::Version::new(1, 2, 0));

        let result = run_resolve("mod-a", &registry, &installed).await.unwrap();
        assert_eq!(result.to_download.len(), 1);
        assert_eq!(result.to_download[0].0, "mod-a");
    }

    #[tokio::test]
    async fn installed_version_mismatch_errors() {
        let registry = make_registry(vec![make_mod(
            "mod-a",
            vec!["mod-b >= 2.0.0"],
            "1.0.0",
            "1.1",
        )]);
        let mut installed = HashMap::new();
        installed.insert("mod-b".to_string(), semver::Version::new(1, 0, 0));

        let result = run_resolve("mod-a", &registry, &installed).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not satisfy"));
    }

    #[tokio::test]
    async fn version_constraints_respected() {
        // mod-a depends on mod-b >= 2.0.0, but mod-b only has 1.0.0 for factorio 1.1
        let mut mod_b = make_mod("mod-b", vec![], "1.0.0", "1.1");
        // Add a 2.0.0 release for a different factorio version
        mod_b.releases.push(Release {
            download_url: "/download/mod-b".to_string(),
            file_name: "mod-b_2.0.0.zip".to_string(),
            version: semver::Version::new(2, 0, 0),
            factorio_version: "2.0".to_string(),
            sha1: String::new(),
            dependencies: vec![],
        });

        let registry = make_registry(vec![
            make_mod("mod-a", vec!["mod-b >= 2.0.0"], "1.0.0", "1.1"),
            mod_b,
        ]);
        let installed = HashMap::new();

        let result = run_resolve("mod-a", &registry, &installed).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No compatible release"));
    }

    #[tokio::test]
    async fn optional_deps_not_downloaded() {
        let registry = make_registry(vec![
            make_mod(
                "mod-a",
                vec!["? optional-mod", "(?) hidden-mod"],
                "1.0.0",
                "1.1",
            ),
        ]);
        let installed = HashMap::new();

        let result = run_resolve("mod-a", &registry, &installed).await.unwrap();
        assert_eq!(result.to_download.len(), 1);
        assert_eq!(result.to_download[0].0, "mod-a");
    }

    #[tokio::test]
    async fn dependency_first_order() {
        let registry = make_registry(vec![
            make_mod("mod-a", vec!["mod-b", "mod-c"], "1.0.0", "1.1"),
            make_mod("mod-b", vec!["mod-c"], "1.0.0", "1.1"),
            make_mod("mod-c", vec![], "1.0.0", "1.1"),
        ]);
        let installed = HashMap::new();

        let result = run_resolve("mod-a", &registry, &installed).await.unwrap();
        let names: Vec<&str> = result.to_download.iter().map(|(n, _)| n.as_str()).collect();
        // mod-c should come first (deepest dependency), then mod-b, then mod-a
        assert_eq!(names, vec!["mod-c", "mod-b", "mod-a"]);
    }
}
