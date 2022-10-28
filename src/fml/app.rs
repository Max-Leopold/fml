use std::borrow::Borrow;
use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::{debug, info};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph};
use tui::{Frame, Terminal};

use crate::factorio::{api, mod_list, server_settings};
use crate::fml_config::{self, FmlConfig};

use super::event::{Event, Events, KeyCode};
use super::mods::StatefulModList;
use super::widgets::mod_list::{ModList, ModListItem};

#[derive(Debug, Clone, Copy)]
enum Tabs {
    Manage,
    Install,
}

pub struct FML {
    stateful_mod_list: StatefulModList,
    mod_list: mod_list::ModList,
    server_settings: server_settings::ServerSettings,
    events: Events,
    filter: String,
    current_tab: Tabs,
}

impl FML {
    pub async fn new(fml_config: FmlConfig) -> Self {
        let mod_list = mod_list::ModList::load_or_create(&fml_config.mods_dir_path).unwrap();
        let server_settings =
            server_settings::get_server_settings(&fml_config.server_config_path).unwrap();
        let stateful_mod_list = Self::generate_mod_list().await.unwrap();
        let events = Events::with_config(None);
        let filter = String::new();
        let current_tab = Tabs::Manage;

        FML {
            stateful_mod_list,
            mod_list,
            server_settings,
            events,
            filter,
            current_tab,
        }
    }

    async fn generate_mod_list() -> Option<StatefulModList> {
        let mods = api::get_mods(None).await.ok()?;
        let mod_list_items = mods
            .into_iter()
            .map(|mod_| {
                let mod_name = mod_.name.clone();
                ModListItem::new(mod_, false)
            })
            .collect();
        let mod_list = StatefulModList::with_items(mod_list_items);
        Some(mod_list)
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting FML!");

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
                        KeyCode::Ctrl('c') => break,
                        KeyCode::Up => self.stateful_mod_list.previous(),
                        KeyCode::Down => self.stateful_mod_list.next(),
                        KeyCode::Enter => {
                            let enabled = self.stateful_mod_list.toggle_install(None);
                            let mod_ = self.stateful_mod_list.selected_mod();
                            if let Some(mod_) = mod_ {
                                let factorio_mod = mod_.factorio_mod;
                                self.mod_list
                                    .set_mod_enabled(&factorio_mod.name, enabled.unwrap());
                            }
                        }
                        KeyCode::Char(c) => {
                            self.stateful_mod_list.reset_selected();
                            self.filter.push(c);
                        }
                        KeyCode::Backspace => {
                            self.stateful_mod_list.reset_selected();
                            self.filter.pop();
                        }
                        KeyCode::Tab => {
                            self.stateful_mod_list.reset_selected();
                            self.filter.clear();
                            match self.current_tab {
                                Tabs::Manage => self.current_tab = Tabs::Install,
                                Tabs::Install => self.current_tab = Tabs::Manage,
                            }
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

        self.draw_tabs(frame, chunks[0]);
        match self.current_tab {
            Tabs::Manage => self.draw_manage_tab(frame, chunks[1]),
            Tabs::Install => self.draw_install_tab(frame, chunks[1]),
        }
    }

    fn draw_tabs(&mut self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let tabs = vec!["Manage", "Install"];
        let tabs = tabs
            .iter()
            .enumerate()
            .map(|(_, t)| Spans::from(*t))
            .collect();

        let tabs = tui::widgets::Tabs::new(tabs)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.current_tab as usize)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs, rect);
    }

    fn draw_manage_tab(&mut self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let block = Block::default().borders(Borders::ALL).title("Manage");
        frame.render_widget(block, rect);
    }

    fn draw_install_tab(&mut self, frame: &mut Frame<impl Backend>, rect: Rect) {
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
        let chunks = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints(
                [
                    tui::layout::Constraint::Percentage(50),
                    tui::layout::Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(layout);
        let items = self.stateful_mod_list.items(&self.filter);

        let list = ModList::with_items(items)
            .block(Block::default().borders(Borders::ALL).title("Mods"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol(">> ")
            .installed_symbol("✔  ");

        frame.render_stateful_widget(list, chunks[0], &mut self.stateful_mod_list.state);

        let selected_mod = self.stateful_mod_list.selected_mod();
        if let Some(selected_mod) = selected_mod {
            let factorio_mod = selected_mod.factorio_mod;
            let mut text = vec![
                Spans::from(factorio_mod.title.clone()),
                Spans::from(factorio_mod.name.clone()),
            ];
            let wrapped = textwrap::wrap(&factorio_mod.summary, chunks[1].width as usize - 2);
            let description = wrapped
                .iter()
                .map(|s| Spans::from(s.borrow()))
                .collect::<Vec<Spans>>();
            text.extend(description);
            let block = Block::default().borders(Borders::ALL).title("Mod Details");
            let paragraph = Paragraph::new(text).block(block);
            frame.render_widget(paragraph, chunks[1]);
        }
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
