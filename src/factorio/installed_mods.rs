use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::OneOrMany;

#[serde_as]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledMod {
    pub name: String,
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    // #[serde(rename = "version")]
    pub version: Vec<String>,
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
    for mod_file in std::fs::read_dir(mods_dir)? {
        let mod_file = mod_file?;
        if mod_file.file_type()?.is_dir()
            || !mod_file.file_name().to_str().unwrap().ends_with(".zip")
        {
            continue;
        }

        let mut mod_file = std::fs::File::open(mod_file.path())?;
        let installed_mod = parse_installed_mod(&mut mod_file);

        if let None = installed_mod {
            continue;
        }

        let installed_mod = installed_mod.unwrap();
        let duplicate = installed_mods
            .iter_mut()
            .find(|m| installed_mod.name == m.name);

        if duplicate.is_some() {
            duplicate
                .unwrap()
                .version
                .push(installed_mod.version.first().unwrap().to_string());
        } else {
            installed_mods.push(installed_mod);
        }
    }
    installed_mods.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(installed_mods)
}

pub fn parse_installed_mod(file: &mut File) -> Option<InstalledMod> {
    // Because we have to clone the zip archive and fs::File doesn't implement Clone, we have to
    // read the entire file into memory and then create a Cusror from it to satisfy the Read + Seek trait
    // requirements of zip::ZipArchive::new
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    let cursor = Cursor::new(buffer);
    let mut zip_archive = zip::ZipArchive::new(cursor).unwrap();
    let zip_archive_clone = zip_archive.clone();
    let info = zip_archive_clone
        .file_names()
        .find(|file_name| file_name.ends_with("info.json"));
    if info.is_none() {
        return None;
    }

    let info = zip_archive.by_name(info.unwrap()).unwrap();
    Some(serde_json::from_reader(info).unwrap())
}

pub fn delete_mod(mod_name: &str, mods_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files = std::fs::read_dir(mods_dir)?;
    for file in files {
        let file = file?;
        if file.file_name().to_str().unwrap().starts_with(mod_name) {
            std::fs::remove_file(file.path())?;
        }
    }
    Ok(())
}
