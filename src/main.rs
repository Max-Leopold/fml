mod factorio;
mod fml;
mod fml_config;

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

    let mut config = fml_config::FmlConfig::default();
    config.load_config()?;

    let res = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async {
            panic::set_hook(Box::new(|panic_info| {
                panic_restore_terminal();
                better_panic::Settings::auto().create_panic_handler()(panic_info);
            }));

            FML::new(config).await.start().await
        });

    if let Err(err) = res {
        error!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}
