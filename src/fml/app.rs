use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{debug, info};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders};
use tui::{Frame, Terminal};

use crate::factorio::{api, mods_config, server_config};

use super::event::{Event, Events, KeyCode};
use super::mods::StatefulModList;
use super::widgets::mod_list::{ModList, ModListItem};

pub struct FML {
    mod_list: StatefulModList,
    mods_config: mods_config::ModsConfig,
    server_config: server_config::ServerConfig,
    events: Events,
    filter: String,
}

impl FML {
    pub fn new() -> Self {
        Self {
            mod_list: StatefulModList::default(),
            mods_config: mods_config::ModsConfig::default(),
            server_config: server_config::ServerConfig::default(),
            events: Events::with_config(None),
            filter: String::new(),
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
        self.mod_list = self
            .generate_mod_list()
            .await
            .expect("Failed to generate mod list");

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

    async fn generate_mod_list(&self) -> Option<StatefulModList> {
        let mods = api::get_mods(None).await.ok()?;
        let mod_list_items = mods
            .into_iter()
            .map(|mod_| ModListItem::new(mod_, false))
            .collect();
        let mod_list = StatefulModList::with_items(mod_list_items);
        Some(mod_list)
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
                        KeyCode::Ctrl('c') => break,
                        KeyCode::Up => self.mod_list.previous(),
                        KeyCode::Down => self.mod_list.next(),
                        KeyCode::Enter => self.mod_list.toggle_install(None),
                        KeyCode::Char(c) => {
                            self.mod_list.reset_selected();
                            self.filter.push(c);
                        }
                        KeyCode::Backspace => {
                            self.mod_list.reset_selected();
                            self.filter.pop();
                        }
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
        let chunks = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints(
                [
                    tui::layout::Constraint::Length(3),
                    tui::layout::Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(rect);

        self.draw_search_bar(frame, chunks[0]);
        self.draw_list(frame, chunks[1]);
    }

    fn draw_list(&mut self, frame: &mut Frame<impl Backend>, layout: Rect) {
        let items = self.mod_list.items(&self.filter);

        let list = ModList::with_items(items)
            .block(Block::default().borders(Borders::ALL).title("Mods"))
            .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ")
            .installed_symbol("✔  ");

        frame.render_stateful_widget(list, layout, &mut self.mod_list.state);
    }

    fn draw_search_bar(&mut self, frame: &mut Frame<impl Backend>, layout: Rect) {
        let search_bar = tui::widgets::Paragraph::new(self.filter.as_str())
            .block(Block::default().borders(Borders::ALL).title("Search"));

        frame.render_widget(search_bar, layout);
    }

    async fn next_event(&mut self) -> Option<Event<KeyCode>> {
        self.events.next().await
    }
}
