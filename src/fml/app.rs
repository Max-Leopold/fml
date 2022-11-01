use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use log::info;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Spans, Text};
use tui::widgets::{Block, Borders, Gauge, Paragraph, Wrap};
use tui::{Frame, Terminal};

use crate::factorio::{api, mod_list, server_settings};
use crate::fml_config::FmlConfig;

use super::event::{Event, Events, KeyCode};
use super::handler::handler::EventHandler;
use super::mods::StatefulModList;
use super::widgets::loading::Loading;
use super::widgets::mod_list::{ModList, ModListItem};
use super::{markdown, util};

#[derive(Debug, Clone, Copy)]
pub enum Tabs {
    Manage,
    Install,
}

pub enum ActiveBlock {
    InstallModList,
    InstallSearch,
    ManageModList,
}

#[derive(Debug, Clone)]
pub struct ModItem {
    pub mod_: api::Mod,
    pub download_info: DownloadInfo,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadInfo {
    pub downloaded: bool,
    pub downloading: bool,
    pub download_perc: u16,
    pub versions: Vec<String>,
}

pub struct FML {
    pub stateful_mod_list: Arc<Mutex<StatefulModList>>,
    pub mod_list: mod_list::ModList,
    pub server_settings: server_settings::ServerSettings,
    pub fml_config: FmlConfig,
    events: Events,
    pub filter: String,
    pub current_tab: Tabs,
    pub show_quit_popup: bool,
    pub active_block: ActiveBlock,
    pub ticks: u64,
    should_quit: bool,
}

impl ModItem {
    pub fn new(mod_: api::Mod) -> ModItem {
        ModItem {
            mod_,
            download_info: DownloadInfo::default(),
        }
    }
}

impl FML {
    pub async fn new(fml_config: FmlConfig) -> FML {
        let mod_list = mod_list::ModList::load_or_create(&fml_config.mods_dir_path).unwrap();
        let server_settings =
            server_settings::get_server_settings(&fml_config.server_config_path).unwrap();

        let stateful_mod_list = Arc::new(Mutex::new(StatefulModList::default()));
        let stateful_mod_list_clone = stateful_mod_list.clone();
        // in a seperate thread we will update the mod list
        let mods_dir_path = fml_config.mods_dir_path.clone();
        tokio::spawn(async move {
            let mod_list = Self::generate_mod_list(&mods_dir_path).await;
            stateful_mod_list_clone.lock().unwrap().set_items(mod_list);
        });
        let events = Events::with_config(None);
        let filter = String::new();
        let current_tab = Tabs::Manage;
        let ticks = 0;
        let show_quit_popup = false;
        let active_block = ActiveBlock::ManageModList;
        let should_quit = false;

        FML {
            stateful_mod_list,
            mod_list,
            server_settings,
            fml_config,
            events,
            filter,
            current_tab,
            show_quit_popup,
            active_block,
            ticks,
            should_quit,
        }
    }

    async fn generate_mod_list(mods_dir: &str) -> Vec<ModListItem> {
        let mods = api::get_mods(None).await.ok().unwrap();
        let installed_mods = util::find_installed_mods(mods_dir).unwrap();
        let mod_list_items = mods
            .into_iter()
            .map(|mod_| {
                let mut mod_item = ModItem::new(mod_);
                if installed_mods.contains_key(&mod_item.mod_.name) {
                    mod_item.download_info.downloaded = true;
                    mod_item.download_info.versions = installed_mods.get(&mod_item.mod_.name).unwrap().to_vec();
                }
                ModListItem::new(mod_item)
            })
            .collect();
        mod_list_items
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
            if self.should_quit {
                break;
            }

            terminal.draw(|frame| self.draw(frame))?;
            if let Some(event) = self.next_event().await {
                super::handler::handler::Handler::handle(event, self);
            }
        }

        Ok(())
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
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

        if self.show_quit_popup {
            let block = Block::default()
                .title("Save Changes?")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));
            let area = util::centered_rect(30, 6, frame.size());
            let text = util::centered_text(
                Text::raw("Save changes to mod-list.json? (y/n/c)"),
                block.inner(area).width.into(),
                block.inner(area).height.into(),
                Some(true),
            );
            let popup = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            frame.render_widget(tui::widgets::Clear, area);
            frame.render_widget(popup, area);
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
        if !(self.stateful_mod_list.lock().unwrap().is_ready()) {
            let loading = Loading::new()
                .block(Block::default().borders(Borders::ALL).title("Mods"))
                .ticks(self.ticks)
                .loading_symbols(vec!["Loading", "Loading.", "Loading..", "Loading..."]);
            frame.render_widget(loading, rect);
            return;
        }

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
        let items = self.stateful_mod_list.lock().unwrap().items(&self.filter);

        let list = ModList::with_items(items)
            .block(Block::default().borders(Borders::ALL).title("Mods"))
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol(">> ")
            .installed_symbol("✔  ");

        frame.render_stateful_widget(
            list,
            chunks[0],
            &mut self.stateful_mod_list.lock().unwrap().state,
        );

        self.draw_mod_details(frame, chunks[1]);
    }

    fn draw_mod_details(&mut self, frame: &mut Frame<impl Backend>, layout: Rect) {
        let selected_mod = self.stateful_mod_list.lock().unwrap().selected_mod();
        if let Some(selected_mod) = selected_mod {
            if !(selected_mod.lock().unwrap().loading) {
                selected_mod.lock().unwrap().loading = true;
                let selected_mod = selected_mod.clone();
                let stateful_mod_list = self.stateful_mod_list.clone();
                tokio::spawn(async move {
                    let name = selected_mod.lock().unwrap().mod_item.mod_.name.clone();
                    // Small debounce so we don't spam the api
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    let new_selected_mod = stateful_mod_list.lock().unwrap().selected_mod();
                    if let Some(new_selected_mod) = new_selected_mod {
                        if new_selected_mod.lock().unwrap().mod_item.mod_.name == name {
                            // Load full mod information from api
                            match api::get_mod(&name).await {
                                Ok(mod_) => {
                                    selected_mod.lock().unwrap().mod_item.mod_ = mod_;
                                }
                                Err(err) => {
                                    selected_mod.lock().unwrap().loading = false;
                                    panic!("{}", err);
                                }
                            }
                        } else {
                            selected_mod.lock().unwrap().loading = false;
                        }
                    } else {
                        selected_mod.lock().unwrap().loading = false;
                    }
                });
            }

            let chunks = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints(
                    [
                        tui::layout::Constraint::Min(3),
                        tui::layout::Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(layout);

            if selected_mod.lock().unwrap().mod_item.mod_.full == Some(true) {
                let mod_ = selected_mod.lock().unwrap().mod_item.mod_.clone();
                let mut text = vec![
                    Spans::from(format!("Name:      {}", mod_.title)),
                    Spans::from(format!("Downloads: {}", mod_.downloads_count)),
                    Spans::from("".to_string()),
                ];
                let description = mod_.description.unwrap_or("".to_string());
                let mut desc = markdown::Parser::new(&description).to_spans();
                text.append(&mut desc);
                let text = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL).title("Mod Info"))
                    .wrap(Wrap { trim: true });
                frame.render_widget(text, chunks[0]);
            } else {
                let loading = Loading::new()
                    .block(Block::default().borders(Borders::ALL).title("Mod Info"))
                    .ticks(self.ticks)
                    .loading_symbols(vec!["Loading", "Loading.", "Loading..", "Loading..."]);
                frame.render_widget(loading, chunks[0]);
            }

            let progress = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Download Progress"),
                )
                .gauge_style(Style::default().fg(Color::Green))
                .percent(
                    selected_mod
                        .lock()
                        .unwrap()
                        .mod_item
                        .download_info
                        .download_perc,
                );
            frame.render_widget(progress, chunks[1]);
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
