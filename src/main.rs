mod factorio;
mod fml;
mod fml_config;

use std::fs::File;
use std::sync::{Arc, Mutex};
use std::{io, panic};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use fml::app::FML;
use fml_config::FmlConfig;
use log::{error, LevelFilter};
use simplelog::{Config, WriteLogger};
use tui::backend::CrosstermBackend;
use tui::Terminal;

use crate::fml::event::Event;
use crate::fml::handler::handler;

use crate::fml::ui::draw::draw;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    action: Option<Action>,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Init,
}

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

#[tokio::main]
async fn main() -> Result<()> {
    better_panic::install();
    panic::set_hook(Box::new(|panic_info| {
        panic_restore_terminal();
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("dev.log").unwrap(),
    );

    let args = Args::parse();
    match args.action {
        Some(action) => match action {
            Action::Init => {
                FmlConfig::init()?;
            }
        },
        _ => {}
    }

    let config = FmlConfig::load_config()?;

    let fml = Arc::new(Mutex::new(FML::new(config)));

    let fml_clone = fml.clone();
    let res = start_ui(fml_clone);

    if let Err(err) = res {
        error!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn start_ui(fml: Arc<Mutex<FML>>) -> Result<()> {
    log::info!("Starting FML!");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let mut fml = fml.lock().unwrap();
        if fml.should_quit {
            break;
        }

        terminal.draw(|frame| draw(&fml, frame))?;

        match fml.events.next()? {
            Event::Input(input) => {
                log::info!("Process Input: {:?}", input);
                handler::handle(input, &mut fml);
            }
            Event::Tick => {
                fml.ticks += 1;
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
