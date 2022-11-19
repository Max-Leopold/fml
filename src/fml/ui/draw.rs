use tui::backend::Backend;
use tui::layout::{Alignment, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Spans, Text};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

use crate::fml::app::{ActiveBlock, Tab, FML};

use super::install_tab::draw_install_tab;
use super::manage_tab::draw_manage_tab;
use super::util;

pub fn draw(fml: &FML, frame: &mut Frame<impl Backend>) {
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

    draw_tabs(fml, frame, chunks[0]);
    match fml.current_tab() {
        Tab::Manage => draw_manage_tab(fml, frame, chunks[1]),
        Tab::Install => draw_install_tab(fml, frame, chunks[1]),
    }

    if fml.active_block() == ActiveBlock::QuitPopup {
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

fn draw_tabs(fml: &FML, frame: &mut Frame<impl Backend>, rect: Rect) {
    let tabs = vec!["Manage", "Install"];
    let tabs = tabs
        .iter()
        .enumerate()
        .map(|(_, t)| Spans::from(*t))
        .collect();

    let tabs = tui::widgets::Tabs::new(tabs)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(fml.current_tab() as usize)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));

    let download_gauge = fml.mod_downloader.generate_gauge();
    let rect = match download_gauge {
        Some(_) => {
            let chunks = Layout::default()
                .direction(tui::layout::Direction::Horizontal)
                .constraints(
                    [
                        tui::layout::Constraint::Percentage(50),
                        tui::layout::Constraint::Percentage(50),
                    ]
                    .as_ref(),
                )
                .split(rect);

            let download_gauge = download_gauge
                .unwrap()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Download Progress"),
                )
                .gauge_style(Style::default().fg(Color::Green));

            frame.render_widget(download_gauge, chunks[1]);
            chunks[0]
        }
        None => rect,
    };

    frame.render_widget(tabs, rect);
}
