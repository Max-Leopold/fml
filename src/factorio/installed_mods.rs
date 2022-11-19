use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use serde::{Deserialize, Serialize};

use crate::skip_err;
use crate::skip_none;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub name: String,
    pub version: String,
    #[serde(rename = "factorio_version")]
    pub factorio_version: String,
    pub title: String,
    pub dependencies: Option<Vec<String>>,
    pub description: String,
}

pub fn read_installed_mods(
    mods_dir: &str,
) -> Result<Vec<InstalledMod>, Box<dyn std::error::Error>> {
    let mut installed_mods: Vec<InstalledMod> = Vec::new();
    for potential_mod_file in std::fs::read_dir(mods_dir)? {
        if potential_mod_file.is_err() {
            log::error!(
                "Error reading directory entry: {}",
                potential_mod_file.err().unwrap()
            );
            continue;
        }

        let potential_mod_file = potential_mod_file.unwrap();

        let is_dir = potential_mod_file
            .file_type()
            .and_then(|ft| Ok(ft.is_dir()))
            .unwrap_or(true);
        let is_zip_file = potential_mod_file
            .file_name()
            .to_str()
            .and_then(|file_name| Some(file_name.ends_with(".zip")))
            .unwrap_or(false);

        if is_dir || !is_zip_file {
            log::info!(
                "Skipping directory entry: {}",
                potential_mod_file.file_name().to_str().unwrap()
            );
            continue;
        }

        if let Ok(mut mod_file) = std::fs::File::open(potential_mod_file.path()) {
            let installed_mod = skip_err!(parse_installed_mod(&mut mod_file));

            installed_mods.push(installed_mod);
        }
    }
    installed_mods.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(installed_mods)
}

pub fn parse_installed_mod(file: &mut File) -> Result<InstalledMod, Box<dyn std::error::Error>> {
    // Because we have to clone the zip archive and fs::File doesn't implement Clone, we have to
    // read the entire file into memory and then create a Cusror from it to satisfy the Read + Seek trait
    // requirements of zip::ZipArchive::new
    file.seek(SeekFrom::Start(0))?;

    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let cursor = Cursor::new(buffer);
    let mut zip_archive = zip::ZipArchive::new(cursor)?;
    let zip_archive_clone = zip_archive.clone();
    let info = zip_archive_clone
        .file_names()
        .find(|file_name| file_name.ends_with("info.json"));

    match info {
        Some(info) => {
            let info = zip_archive.by_name(info)?;
            Ok(serde_json::from_reader(info)?)
        }
        None => Err("No info.json file found in mod".into()),
    }
}

pub fn delete_mod(mod_name: &str, mods_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files = std::fs::read_dir(mods_dir)?;
    for file in files {
        let file = file?;
        let file_name = file.file_name();
        let file_name = skip_none!(file_name.to_str());
        if file_name.starts_with(mod_name) {
            std::fs::remove_file(file.path())?;
        }
    }
    Ok(())
}
