use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{debug, info};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::Rect;
use tui::style::{Color, Style, Modifier};
use tui::text::Spans;
use tui::widgets::{Block, Borders};
use tui::{Frame, Terminal};

use crate::factorio::{api, mods_config, server_config};

use super::event::{Event, Events, KeyCode};
use super::mods::ModList;
use super::widgets::list::{List, ListItem};

pub struct FML {
    pub mod_list: ModList,
    pub mods_config: mods_config::ModsConfig,
    pub server_config: server_config::ServerConfig,
    events: Events,
}

impl FML {
    pub fn new() -> Self {
        Self {
            mod_list: ModList::default(),
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
        self.mod_list = ModList::with_items(
            api::get_mods(Some(api::SortBy::Downloads)).await.unwrap()
        );

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
            terminal.draw(|frame| self.draw(frame))?;
            if let Some(event) = self.next_event().await {
                match event {
                    Event::Input(input) => match input {
                        KeyCode::Ctrl('c') => { break },
                        KeyCode::Up => { self.mod_list.previous() }
                        KeyCode::Down => { self.mod_list.next() }
                        _ => {}
                    },
                    Event::Tick => {
                        // If we ever need to do some recurring task every tick we can call it here
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame<impl Backend>) {
        let rect = frame.size();

        self.draw_list(frame, rect);
    }

    fn draw_list(&mut self, frame: &mut Frame<impl Backend>, layout: Rect) {
        let items: Vec<ListItem> = self.mod_list.items.iter().map(|m| {
            let lines = vec![Spans::from(&*m.title)];
            ListItem::new(lines).style(Style::default().fg(Color::Gray))
        }).collect();

        let items = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Mods"))
            .highlight_style(
                Style::default().bg(Color::Red).add_modifier(Modifier::BOLD)
            ).highlight_symbol(">> ");

        frame.render_stateful_widget(items, layout, &mut self.mod_list.state);
    }

    async fn next_event(&mut self) -> Option<Event<KeyCode>> {
        self.events.next().await
    }
}
