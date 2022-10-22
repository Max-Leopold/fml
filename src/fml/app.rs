use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{debug, info};
use tui::backend::{Backend, CrosstermBackend};
use tui::widgets::{Block, Borders};
use tui::Terminal;

use crate::factorio::{api, mods_config, server_config};

use super::event::{Event, Events};

pub struct FML {
    pub mods: Vec<api::Mod>,
    pub mods_config: mods_config::ModsConfig,
    pub server_config: server_config::ServerConfig,
    events: Events,
}

impl FML {
    pub fn new() -> Self {
        Self {
            mods: Vec::new(),
            mods_config: mods_config::ModsConfig::default(),
            server_config: server_config::ServerConfig::default(),
            events: Events::with_config(None),
        }
    }

    pub fn with_server_config(&mut self, server_config_path: &str) -> &mut Self {
        debug!("Loading server config from {}", server_config_path);
        self.server_config = server_config::get_server_config(server_config_path)
            .expect("Failed to load server config");
        self
    }

    pub fn with_mods_config(&mut self, mods_config_path: &str) -> &mut Self {
        debug!("Loading mods config from {}", mods_config_path);
        self.mods_config =
            mods_config::get_mods_config(mods_config_path).expect("Failed to load mods config");
        self
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting FML!");
        self.mods = api::get_mods(Some(api::SortBy::Downloads)).await?;

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.run(&mut terminal).await?;

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

    pub async fn run<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            if let Some(event) = self.next_event().await {
                match event {
                    Event::Input(input) => match input {
                        KeyEvent {
                            code: KeyCode::Char('q'),
                            ..
                        } => {
                            break;
                        }
                        _ => {}
                    },
                    Event::Tick => {
                        terminal.draw(|f| {
                            let size = f.size();
                            let block = Block::default().title("FML").borders(Borders::ALL);
                            f.render_widget(block, size);
                        })?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn next_event(&mut self) -> Option<Event<KeyEvent>> {
        self.events.next().await
    }
}
