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

use crate::factorio::installed_mods::InstalledMod;
use crate::factorio::{api, installed_mods, mod_list, server_settings};
use crate::fml_config::FmlConfig;

use super::event::{Event, Events, KeyCode};
use super::handler::handler;
use super::install_mod_list::{InstallModItem, InstallModList};
use super::manage_mod_list::ManageModList;
use super::mod_downloader::{ModDownloadRequest, ModDownloader};
use super::widgets::enabled_list::EnabledList;
use super::widgets::loading::Loading;
use super::{markdown, util};

#[derive(Debug, Clone, Copy)]
pub enum Tab {
    Manage,
    Install,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActiveBlock {
    InstallModList,
    InstallSearch,
    InstallModDetails,
    ManageModList,
    QuitPopup,
}

pub struct Route {
    active_block: ActiveBlock,
    tab: Tab,
}

const DEFAULT_ROUTE: Route = Route {
    active_block: ActiveBlock::ManageModList,
    tab: Tab::Manage,
};

pub struct FML {
    pub install_mod_list: Arc<Mutex<InstallModList>>,
    pub manage_mod_list: Arc<Mutex<ManageModList>>,
    pub server_settings: server_settings::ServerSettings,
    pub fml_config: FmlConfig,
    pub events: Events,
    mod_downloader: ModDownloader,
    navigation_history: Vec<Route>,
    pub ticks: u64,
    should_quit: bool,
    pub scroll_offset: u16,
}

impl FML {
    pub async fn new(fml_config: FmlConfig) -> FML {
        let server_settings =
            server_settings::get_server_settings(&fml_config.server_config_path).unwrap();

        let install_mod_list = Arc::new(Mutex::new(InstallModList::default()));
        let install_mod_list_clone = install_mod_list.clone();
        // in a seperate thread we will update the mod list
        let mods_dir_path = fml_config.mods_dir_path.clone();
        tokio::spawn(async move {
            let mod_list = Self::generate_install_mod_list(&mods_dir_path).await;
            install_mod_list_clone.lock().unwrap().set_items(mod_list);
        });

        let manage_mod_list = Arc::new(Mutex::new(ManageModList::default()));
        let manage_mod_list_clone = manage_mod_list.clone();
        // in a seperate thread we will update the mod list
        let mods_dir_path = fml_config.mods_dir_path.clone();
        tokio::spawn(async move {
            let mod_list_items = Self::generate_manage_mod_list(&mods_dir_path);
            let mod_list = mod_list::ModList::load_or_create(&mods_dir_path).unwrap();
            manage_mod_list_clone
                .lock()
                .unwrap()
                .set_items(mod_list_items, mod_list);
        });
        let events = Events::with_config(None);
        let mod_downloader = ModDownloader::new(install_mod_list.clone(), manage_mod_list.clone());
        let ticks = 0;
        let should_quit = false;
        let navigation_history = vec![DEFAULT_ROUTE];
        let scroll_offset = 0;

        FML {
            install_mod_list,
            manage_mod_list,
            server_settings,
            fml_config,
            events,
            mod_downloader,
            navigation_history,
            ticks,
            should_quit,
            scroll_offset,
        }
    }

    async fn generate_install_mod_list(mods_dir: &str) -> Vec<InstallModItem> {
        let mods = api::get_mods(None).await.ok().unwrap();
        let installed_mods = installed_mods::read_installed_mods(mods_dir).unwrap();
        let installed_mods = installed_mods
            .into_iter()
            .map(|mod_| (mod_.name.clone(), mod_))
            .collect::<std::collections::HashMap<String, installed_mods::InstalledMod>>();
        let mod_list_items = mods
            .into_iter()
            .map(|mod_| {
                let mut mod_item = InstallModItem::new(mod_);
                if installed_mods.contains_key(&mod_item.mod_.name) {
                    mod_item.download_info.downloaded = true;
                    mod_item.download_info.versions = installed_mods
                        .get(&mod_item.mod_.name)
                        .unwrap()
                        .version
                        .clone();
                }
                mod_item
            })
            .collect();
        mod_list_items
    }

    fn generate_manage_mod_list(mods_dir: &str) -> Vec<InstalledMod> {
        installed_mods::read_installed_mods(mods_dir).unwrap()
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
                handler::handle(event, self);
            }
        }

        Ok(())
    }

    pub fn quit_gracefully(&mut self) {
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
        match self.current_tab() {
            Tab::Manage => self.draw_manage_tab(frame, chunks[1]),
            Tab::Install => self.draw_install_tab(frame, chunks[1]),
        }

        if self.active_block() == ActiveBlock::QuitPopup {
            let block = Block::default()
                .title("Save Changes?")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));
            let area = util::centered_rect(30, 6, frame.size());
            let text = util::centered_text(
                Text::raw("Save changes to mod-list.json? (y/n)"),
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
            .select(self.current_tab() as usize)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs, rect);
    }

    fn draw_manage_tab(&mut self, frame: &mut Frame<impl Backend>, rect: Rect) {
        self.draw_manage_list(frame, rect);
    }

    fn draw_manage_list(&mut self, frame: &mut Frame<impl Backend>, rect: Rect) {
        let items = self.manage_mod_list.lock().unwrap().items();

        let list = EnabledList::with_items(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Mods")
                    .border_style(self.block_style(ActiveBlock::ManageModList)),
            )
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol(">> ")
            .installed_symbol("✔  ");

        frame.render_stateful_widget(list, rect, &mut self.manage_mod_list.lock().unwrap().state);
    }

    fn draw_install_tab(&mut self, frame: &mut Frame<impl Backend>, rect: Rect) {
        if !(self.install_mod_list.lock().unwrap().is_ready()) {
            let loading = Loading::new()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Mods")
                        .border_style(self.block_style(ActiveBlock::InstallModList)),
                )
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
        self.draw_install_list(frame, chunks[1]);
    }

    fn draw_install_list(&mut self, frame: &mut Frame<impl Backend>, layout: Rect) {
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
        let items = self.install_mod_list.lock().unwrap().items();

        let list = EnabledList::with_items(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Mods")
                    .border_style(self.block_style(ActiveBlock::InstallModList)),
            )
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_symbol(">> ")
            .installed_symbol("✔  ");

        frame.render_stateful_widget(
            list,
            chunks[0],
            &mut self.install_mod_list.lock().unwrap().state,
        );

        self.draw_mod_details(frame, chunks[1]);
    }

    fn draw_mod_details(&mut self, frame: &mut Frame<impl Backend>, layout: Rect) {
        let selected_mod = self.install_mod_list.lock().unwrap().selected_mod();
        if let Some(selected_mod) = selected_mod {
            if !(selected_mod.lock().unwrap().loading) {
                selected_mod.lock().unwrap().loading = true;
                let selected_mod = selected_mod.clone();
                let install_mod_list = self.install_mod_list.clone();
                tokio::spawn(async move {
                    let name = selected_mod.lock().unwrap().mod_.name.clone();
                    // Small debounce so we don't spam the api
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    let new_selected_mod = install_mod_list.lock().unwrap().selected_mod();
                    if let Some(new_selected_mod) = new_selected_mod {
                        if new_selected_mod.lock().unwrap().mod_.name == name {
                            // Load full mod information from api
                            match api::get_mod(&name).await {
                                Ok(mod_) => {
                                    selected_mod.lock().unwrap().mod_ = mod_;
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

            if selected_mod.lock().unwrap().mod_.full == Some(true) {
                let mod_ = selected_mod.lock().unwrap().mod_.clone();
                let mut text = vec![
                    Spans::from(format!("Name: {}", mod_.title)),
                    Spans::from(format!("Downloads: {}", mod_.downloads_count)),
                    Spans::from("".to_string()),
                ];
                let dependencies = mod_.latest_release().info_json.dependencies.unwrap();
                let required_dependencies = dependencies.required.iter().map(|d| {
                    Spans::from(format!(
                        "- {} {} {}",
                        d.name,
                        d.equality.as_ref().unwrap_or(&String::new()),
                        d.version.as_ref().unwrap_or(&String::new())
                    ))
                });
                if required_dependencies.len() > 0 {
                    text.push(Spans::from("Required Dependencies:"));
                    text.extend(required_dependencies);
                    text.push(Spans::from("".to_string()));
                }

                let optional_dependencies = dependencies.optional.iter().map(|d| {
                    Spans::from(format!(
                        "- {} {} {}",
                        d.name,
                        d.equality.as_ref().unwrap_or(&String::new()),
                        d.version.as_ref().unwrap_or(&String::new())
                    ))
                });
                if optional_dependencies.len() > 0 {
                    text.push(Spans::from("Optional Dependencies:"));
                    text.extend(optional_dependencies);
                    text.push(Spans::from("".to_string()));
                }

                let incompatible_dependencies = dependencies.incompatible.iter().map(|d| {
                    Spans::from(format!(
                        "- {} {} {}",
                        d.name,
                        d.equality.as_ref().unwrap_or(&String::new()),
                        d.version.as_ref().unwrap_or(&String::new())
                    ))
                });
                if incompatible_dependencies.len() > 0 {
                    text.push(Spans::from("Incompatible Dependencies:"));
                    text.extend(incompatible_dependencies);
                    text.push(Spans::from("".to_string()));
                }

                let description = mod_.description.unwrap_or("".to_string());
                let mut desc = markdown::Parser::new(&description).to_spans();
                text.append(&mut desc);
                let text = Paragraph::new(text)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Mod Info")
                            .border_style(self.block_style(ActiveBlock::InstallModDetails)),
                    )
                    .scroll((self.scroll_offset, 0))
                    .wrap(Wrap { trim: true });
                frame.render_widget(text, chunks[0]);
            } else {
                let loading = Loading::new()
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Mod Info")
                            .border_style(self.block_style(ActiveBlock::InstallModDetails)),
                    )
                    .ticks(self.ticks)
                    .loading_symbols(vec!["Loading", "Loading.", "Loading..", "Loading..."]);
                frame.render_widget(loading, chunks[0]);
            }

            let download_gauge = self
                .mod_downloader
                .generate_gauge()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Download Progress"),
                )
                .gauge_style(Style::default().fg(Color::Green));
            frame.render_widget(download_gauge, chunks[1]);
        }
    }

    fn draw_search_bar(&mut self, frame: &mut Frame<impl Backend>, layout: Rect) {
        let mut search_string = self.install_mod_list.lock().unwrap().filter.clone();
        if self.active_block() == ActiveBlock::InstallSearch {
            search_string += "█";
        }
        let search_bar = tui::widgets::Paragraph::new(search_string).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Search")
                .border_style(self.block_style(ActiveBlock::InstallSearch)),
        );

        frame.render_widget(search_bar, layout);
    }

    async fn next_event(&mut self) -> Option<Event<KeyCode>> {
        self.events.next().await
    }

    fn block_style(&self, block: ActiveBlock) -> Style {
        if self.active_block() == block {
            default_active_block_style()
        } else {
            default_block_style()
        }
    }

    pub fn current_tab(&self) -> Tab {
        self.navigation_history.last().unwrap_or(&DEFAULT_ROUTE).tab
    }

    pub fn active_block(&self) -> ActiveBlock {
        self.navigation_history
            .last()
            .unwrap_or(&DEFAULT_ROUTE)
            .active_block
    }

    fn navigate(&mut self, route: Route) {
        self.navigation_history.push(route);
    }

    pub fn navigate_tab(&mut self, tab: Tab) {
        let active_block = match tab {
            Tab::Manage => ActiveBlock::ManageModList,
            Tab::Install => ActiveBlock::InstallModList,
        };
        self.navigate(Route { tab, active_block });
    }

    pub fn navigate_block(&mut self, active_block: ActiveBlock) {
        let tab = match active_block {
            ActiveBlock::ManageModList => Tab::Manage,
            ActiveBlock::InstallModList
            | ActiveBlock::InstallSearch
            | ActiveBlock::InstallModDetails => Tab::Install,
            ActiveBlock::QuitPopup => self.current_tab(),
        };

        self.navigate(Route { tab, active_block });
    }

    pub fn undo_navigation(&mut self) {
        self.navigation_history.pop();
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn save(&self) {
        self.manage_mod_list
            .lock()
            .unwrap()
            .generate_mod_list()
            .save(&self.fml_config.mods_dir_path)
            .unwrap();
        // todo!("Save installed mods to mod-list.json")
    }

    pub fn delete_mod(&self, mod_name: &str) {
        installed_mods::delete_mod(mod_name, &self.fml_config.mods_dir_path).unwrap();

        self.manage_mod_list.lock().unwrap().remove_mod(mod_name);
        self.install_mod_list.lock().unwrap().disable_mod(mod_name);
    }

    pub fn install_mod(&self, mod_: Arc<Mutex<InstallModItem>>) {
        let mod_ = mod_.lock().unwrap().clone();

        self.mod_downloader
            .tx
            .send(ModDownloadRequest {
                mod_name: mod_.mod_.name.clone(),
                username: self.server_settings.username.clone(),
                token: self.server_settings.token.clone(),
                mod_dir: self.fml_config.mods_dir_path.clone(),
            })
            .unwrap();
    }
}

fn default_active_block_style() -> Style {
    Style::default().fg(Color::Yellow)
}

fn default_block_style() -> Style {
    Style::reset()
}
