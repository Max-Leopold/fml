use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::io::Write;
use std::path::Path;

use super::types::{Dependency, Mod, ModListEntry, Release};

const BASE_URL: &str = "https://mods.factorio.com";

// --- JSON response structures ---

#[derive(Debug, Deserialize)]
struct ModListResponse {
    results: Vec<ApiModEntry>,
}

#[derive(Debug, Deserialize)]
struct ApiModEntry {
    name: String,
    title: String,
    downloads_count: u64,
    #[serde(default)]
    summary: String,
}

#[derive(Debug, Deserialize)]
struct ApiModFull {
    name: String,
    title: String,
    #[serde(default)]
    summary: String,
    downloads_count: u64,
    #[serde(default)]
    releases: Vec<ApiRelease>,
}

#[derive(Debug, Deserialize)]
struct ApiRelease {
    download_url: String,
    file_name: String,
    version: String,
    #[serde(default)]
    sha1: String,
    info_json: ApiInfoJson,
}

#[derive(Debug, Deserialize)]
struct ApiInfoJson {
    factorio_version: String,
    #[serde(default)]
    dependencies: Vec<String>,
}

// --- Public API functions ---

pub async fn fetch_mod_list(factorio_version: &str) -> Result<Vec<ModListEntry>> {
    let url = format!(
        "{}/api/mods?page_size=max&hide_deprecated=true&version={}",
        BASE_URL, factorio_version
    );

    let resp = reqwest::get(&url)
        .await
        .context("Failed to fetch mod list from Factorio mod portal")?;

    if !resp.status().is_success() {
        bail!(
            "Mod portal returned HTTP {} when fetching mod list",
            resp.status()
        );
    }

    let body: ModListResponse = resp
        .json()
        .await
        .context("Failed to parse mod list response")?;

    let mut entries: Vec<ModListEntry> = body
        .results
        .into_iter()
        .map(|e| ModListEntry {
            name: e.name,
            title: e.title,
            downloads_count: e.downloads_count,
            summary: e.summary,
        })
        .collect();

    entries.sort_by(|a, b| b.downloads_count.cmp(&a.downloads_count));
    Ok(entries)
}

pub async fn fetch_mod_details(name: &str) -> Result<Mod> {
    let url = format!("{}/api/mods/{}/full", BASE_URL, name);

    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to fetch details for mod '{}'", name))?;

    if resp.status().as_u16() == 404 {
        bail!("Mod '{}' not found on the mod portal", name);
    }

    if !resp.status().is_success() {
        bail!(
            "Mod portal returned HTTP {} when fetching mod '{}'",
            resp.status(),
            name
        );
    }

    let body: ApiModFull = resp
        .json()
        .await
        .with_context(|| format!("Failed to parse details for mod '{}'", name))?;

    let releases: Vec<Release> = body
        .releases
        .into_iter()
        .filter_map(|r| {
            let version = semver::Version::parse(&r.version).ok()?;
            let dependencies: Vec<Dependency> = r
                .info_json
                .dependencies
                .iter()
                .filter_map(|d| d.parse().ok())
                .collect();
            Some(Release {
                download_url: r.download_url,
                file_name: r.file_name,
                version,
                factorio_version: r.info_json.factorio_version,
                sha1: r.sha1,
                dependencies,
            })
        })
        .collect();

    Ok(Mod {
        name: body.name,
        title: body.title,
        summary: body.summary,
        downloads_count: body.downloads_count,
        releases,
    })
}

pub async fn download_mod(
    release: &Release,
    username: &str,
    token: &str,
    dir: &str,
) -> Result<()> {
    let url = format!(
        "{}{}?username={}&token={}",
        BASE_URL, release.download_url, username, token
    );

    let file_path = Path::new(dir).join(&release.file_name);

    let resp = reqwest::get(&url)
        .await
        .with_context(|| format!("Failed to download mod: {}", release.file_name))?;

    if !resp.status().is_success() {
        bail!(
            "Download failed for '{}' (HTTP {}): check username and token in server-settings.json",
            release.file_name,
            resp.status()
        );
    }

    let bytes = resp
        .bytes()
        .await
        .with_context(|| format!("Failed to read download body for {}", release.file_name))?;

    // Write to file
    let mut file = std::fs::File::create(&file_path)
        .with_context(|| format!("Failed to create file: {}", file_path.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("Failed to write file: {}", file_path.display()))?;
    drop(file);

    // Verify SHA1 checksum if provided
    if !release.sha1.is_empty() {
        use sha1_smol::Sha1;
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        let digest = hasher.digest().to_string();
        if digest != release.sha1 {
            let _ = std::fs::remove_file(&file_path);
            bail!(
                "SHA1 mismatch for '{}': expected {}, got {}",
                release.file_name,
                release.sha1,
                digest
            );
        }
    }

    // Verify it's a valid zip
    match zip::ZipArchive::new(std::fs::File::open(&file_path)?) {
        Ok(_) => Ok(()),
        Err(e) => {
            // Delete corrupt file
            let _ = std::fs::remove_file(&file_path);
            bail!(
                "Downloaded file '{}' is not a valid zip archive: {}",
                release.file_name,
                e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Hits the network
    async fn test_fetch_mod_list() {
        let mods = fetch_mod_list("2.0").await.unwrap();
        assert!(!mods.is_empty());
        println!("First 5 mods:");
        for m in mods.iter().take(5) {
            println!("  {} - {} ({} downloads)", m.name, m.title, m.downloads_count);
        }
    }

    #[tokio::test]
    #[ignore] // Hits the network
    async fn test_fetch_mod_details() {
        let m = fetch_mod_details("flib").await.unwrap();
        assert_eq!(m.name, "flib");
        assert!(!m.releases.is_empty());
        println!("flib has {} releases", m.releases.len());
    }
}
