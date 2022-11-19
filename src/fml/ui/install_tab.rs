use std::time::Duration;
use tui::backend::Backend;
use tui::layout::{Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

use crate::factorio::api::{self, Dependency, Equality};
use crate::fml::app::{ActiveBlock, FML};
use crate::fml::markdown;
use crate::fml::widgets::enabled_list::EnabledList;
use crate::fml::widgets::loading::Loading;

pub fn draw_install_tab(fml: &FML, frame: &mut Frame<impl Backend>, rect: Rect) {
    if !(fml.install_mod_list.lock().unwrap().is_ready()) {
        let loading = Loading::new()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Mods")
                    .border_style(fml.block_style(ActiveBlock::InstallModList)),
            )
            .ticks(fml.ticks)
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

    draw_search_bar(fml, frame, chunks[0]);
    draw_install_list(fml, frame, chunks[1]);
}

fn draw_install_list(fml: &FML, frame: &mut Frame<impl Backend>, layout: Rect) {
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
    let items = fml.install_mod_list.lock().unwrap().items();

    let list = EnabledList::with_items(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Mods")
                .border_style(fml.block_style(ActiveBlock::InstallModList)),
        )
        .highlight_style(Style::default().fg(Color::Yellow))
        .highlight_symbol(">> ")
        .installed_symbol("✔  ");

    frame.render_stateful_widget(
        list,
        chunks[0],
        &mut fml.install_mod_list.lock().unwrap().state,
    );

    draw_mod_details(fml, frame, chunks[1]);
}

fn draw_mod_details(fml: &FML, frame: &mut Frame<impl Backend>, layout: Rect) {
    let selected_mod = fml.install_mod_list.lock().unwrap().selected_mod();
    if let Some(selected_mod) = selected_mod {
        if !(selected_mod.lock().unwrap().loading) {
            selected_mod.lock().unwrap().loading = true;
            let selected_mod = selected_mod.clone();
            let install_mod_list = fml.install_mod_list.clone();
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

        if selected_mod.lock().unwrap().mod_.full == Some(true) {
            let mod_ = selected_mod.lock().unwrap().mod_.clone();
            let mut text = vec![
                Spans::from(format!("Name: {}", mod_.title)),
                Spans::from(format!("Downloads: {}", mod_.downloads_count)),
                Spans::from("".to_string()),
            ];
            let latest_release = mod_.latest_release();
            if let Some(latest_release) = latest_release {
                if let Some(dependencies) = latest_release.info_json.dependencies.as_ref() {
                    let map_dependencies = |dependencies: &Vec<Dependency>| {
                        dependencies
                            .iter()
                            .map(|d| {
                                Spans::from(format!(
                                    "- {} {} {}",
                                    d.name,
                                    Equality::to_str(d.equality.clone()),
                                    d.version
                                        .as_ref()
                                        .and_then(|v| Some(v.to_string()))
                                        .unwrap_or("".to_string())
                                ))
                            })
                            .collect::<Vec<_>>()
                    };
                    let required_dependencies = map_dependencies(&dependencies.required);
                    if required_dependencies.len() > 0 {
                        text.push(Spans::from("Required Dependencies:"));
                        text.extend(required_dependencies);
                        text.push(Spans::from("".to_string()));
                    }

                    let optional_dependencies = map_dependencies(&dependencies.optional);
                    if optional_dependencies.len() > 0 {
                        text.push(Spans::from("Optional Dependencies:"));
                        text.extend(optional_dependencies);
                        text.push(Spans::from("".to_string()));
                    }

                    let incompatible_dependencies = map_dependencies(&dependencies.incompatible);
                    if incompatible_dependencies.len() > 0 {
                        text.push(Spans::from("Incompatible Dependencies:"));
                        text.extend(incompatible_dependencies);
                        text.push(Spans::from("".to_string()));
                    }
                }
            }

            let description = mod_.description.unwrap_or("".to_string());
            let mut desc = markdown::Parser::new(&description).to_spans();
            text.append(&mut desc);
            let text = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Mod Info")
                        .border_style(fml.block_style(ActiveBlock::InstallModDetails)),
                )
                .scroll((fml.scroll_offset, 0))
                .wrap(Wrap { trim: true });
            frame.render_widget(text, layout);
        } else {
            let loading = Loading::new()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Mod Info")
                        .border_style(fml.block_style(ActiveBlock::InstallModDetails)),
                )
                .ticks(fml.ticks)
                .loading_symbols(vec!["Loading", "Loading.", "Loading..", "Loading..."]);
            frame.render_widget(loading, layout);
        }
    }
}

fn draw_search_bar(fml: &FML, frame: &mut Frame<impl Backend>, layout: Rect) {
    let mut search_string = fml.install_mod_list.lock().unwrap().filter.clone();
    if fml.active_block() == ActiveBlock::InstallSearch {
        search_string += "█";
    }
    let search_bar = tui::widgets::Paragraph::new(search_string).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Search")
            .border_style(fml.block_style(ActiveBlock::InstallSearch)),
    );

    frame.render_widget(search_bar, layout);
}
