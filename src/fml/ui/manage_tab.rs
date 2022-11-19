use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::style::{Color, Style};
use tui::text::Text;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

use crate::fml::app::{ActiveBlock, FML};
use crate::fml::widgets::enabled_list::EnabledList;

use super::util;

pub fn draw_manage_tab(fml: &FML, frame: &mut Frame<impl Backend>, rect: Rect) {
    draw_manage_list(fml, frame, rect);
}

fn draw_manage_list(fml: &FML, frame: &mut Frame<impl Backend>, rect: Rect) {
    let items = fml.manage_mod_list.lock().unwrap().items();
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Mods")
        .border_style(fml.block_style(ActiveBlock::ManageModList));

    if items.is_empty() {
        let text = util::centered_text(
            Text::raw("No mods installed"),
            block.inner(rect).width.into(),
            block.inner(rect).height.into(),
            Some(true),
        );
        let text = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(text, rect);
        return;
    }

    let list = EnabledList::with_items(items)
        .block(block)
        .highlight_style(Style::default().fg(Color::Yellow))
        .highlight_symbol(">> ")
        .installed_symbol("✔  ");

    frame.render_stateful_widget(list, rect, &mut fml.manage_mod_list.lock().unwrap().state);
}
