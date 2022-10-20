mod factorio;
mod fml;

use fml::app::FML;

fn main() {
    let mods_config_path = "mod-list.json";
    let server_config_path = "server-settings.json";
    let result = FML::default()
        .with_mods_config(mods_config_path)
        .with_server_config(server_config_path)
        .start();

    println!("{}", result);
}
