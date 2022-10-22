mod factorio;
mod fml;

use anyhow::Result;
use fml::app::FML;
use log::error;

fn main() -> Result<()> {
    better_panic::install();

    let mods_config_path = "mod-list.json";
    let server_config_path = "server-settings.json";

    let res = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            FML::new()
                .with_mods_config(mods_config_path)
                .with_server_config(server_config_path)
                .start()
                .await
        });

    if let Err(err) = res {
        error!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}
