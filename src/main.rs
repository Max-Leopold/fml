mod config;
mod factorio;
mod fml;

use std::{io, panic};

use anyhow::Result;
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
use crossterm::{cursor, execute};
use fml::app::FML;
use log::error;

pub fn panic_restore_terminal() {
    disable_raw_mode().unwrap();
    let mut terminal = io::stdout();
    execute!(
        terminal,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show
    )
    .unwrap();
}

fn main() -> Result<()> {
    better_panic::install();

    let mut config = config::Config::default();
    config.load_config()?;

    let res = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            panic::set_hook(Box::new(|panic_info| {
                panic_restore_terminal();
                better_panic::Settings::auto().create_panic_handler()(panic_info);
            }));

            FML::new()
                .with_mods_config(&config.mods_dir_path)
                .with_server_config(&config.server_config_path)
                .start()
                .await
        });

    if let Err(err) = res {
        error!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}
