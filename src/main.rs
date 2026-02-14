mod app;
mod config;
mod event;
mod factorio;
mod handler;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tokio::sync::mpsc;

use app::App;
use event::{spawn_event_loop, AppEvent};
use factorio::{
    installed, mod_list::ModList, types,
};

#[derive(Parser)]
#[command(name = "fml", about = "Factorio Mod Manager for headless servers")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize FML configuration
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            config::FmlConfig::init()?;
            return Ok(());
        }
        None => {}
    }

    // Load config
    let config = config::FmlConfig::load()?;

    // Read server settings
    let server_settings = types::read_server_settings(&config.server_config_path)?;

    // Detect Factorio version
    let factorio_version = types::detect_factorio_version(&config.mods_dir_path)?;
    eprintln!("Detected Factorio version: {}", factorio_version);

    // Set up panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(
        factorio_version.clone(),
        server_settings,
        config.mods_dir_path.clone(),
    );

    // Create event channel
    let (tx, mut rx) = mpsc::unbounded_channel::<AppEvent>();

    // Spawn terminal event loop
    spawn_event_loop(tx.clone());

    // Spawn initial async task: fetch mod list
    {
        let fv = factorio_version.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = factorio::api::fetch_mod_list(&fv).await;
            let _ = tx.send(AppEvent::ModListLoaded(result));
        });
    }

    // Spawn initial async task: read installed mods
    {
        let mods_dir = config.mods_dir_path.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = (|| -> anyhow::Result<(Vec<installed::InstalledMod>, ModList)> {
                let mods = installed::read_installed_mods(&mods_dir)?;
                let mod_list = ModList::load_or_create(&mods_dir)?;
                Ok((mods, mod_list))
            })();
            let _ = tx.send(AppEvent::InstalledModsLoaded(result));
        });
    }

    // Main loop
    loop {
        terminal.draw(|frame| ui::draw(&app, frame))?;

        if let Some(event) = rx.recv().await {
            handler::handle_event(event, &mut app, tx.clone());
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
