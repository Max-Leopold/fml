use std::collections::HashMap;

pub fn find_installed_mods(mods_dir: &str) -> Result<HashMap<String, Vec<String>>, Box<dyn std::error::Error>> {
    let mut installed_mods: HashMap<String, Vec<String>> = HashMap::new();
    for mod_file in std::fs::read_dir(mods_dir)? {
        let mod_file = mod_file?;
        let mut mod_file_name = mod_file.file_name().into_string().unwrap();
        if !mod_file_name.ends_with(".zip") {
            continue;
        }
        mod_file_name = mod_file_name.replace(".zip", "");
        let mod_name = mod_file_name.split("_").take(mod_file_name.split("_").count() - 1).collect::<Vec<&str>>().join("_");
        let mod_version = mod_file_name.split("_").last().unwrap().to_string();
        if installed_mods.contains_key(&mod_name) {
            installed_mods.get_mut(&mod_name).unwrap().push(mod_version);
        } else {
            installed_mods.insert(mod_name, vec![mod_version]);
        }
    }
    Ok(installed_mods)
}
