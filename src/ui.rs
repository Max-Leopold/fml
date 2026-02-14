use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{ActiveBlock, App, Tab};

pub fn draw(app: &App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tab bar
            Constraint::Min(1),   // main content
            Constraint::Length(1), // status bar
        ])
        .split(frame.area());

    draw_tabs(app, frame, chunks[0]);

    match app.tab {
        Tab::Manage => draw_manage_tab(app, frame, chunks[1]),
        Tab::Install => draw_install_tab(app, frame, chunks[1]),
    }

    draw_status_bar(app, frame, chunks[2]);

    if app.show_quit_popup {
        draw_quit_popup(frame);
    }
}

fn draw_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let titles: Vec<Line> = vec![
        Line::from(" Manage "),
        Line::from(" Install "),
    ];

    let selected = match app.tab {
        Tab::Manage => 0,
        Tab::Install => 1,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" FML "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn draw_manage_tab(app: &App, frame: &mut Frame, area: Rect) {
    let has_pending = app.manage_mods.iter().any(|m| m.pending);
    let saved_mods: Vec<&crate::app::ManageMod> =
        app.manage_mods.iter().filter(|m| !m.pending).collect();
    let pending_mods: Vec<&crate::app::ManageMod> =
        app.manage_mods.iter().filter(|m| m.pending).collect();

    if saved_mods.is_empty() && pending_mods.is_empty() {
        let msg = Paragraph::new("No mods installed")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Installed Mods "));
        frame.render_widget(msg, area);
        return;
    }

    let is_focused = app.active_block == ActiveBlock::ManageModList;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    if !has_pending {
        // Simple case: no pending mods, render a single list
        let items: Vec<ListItem> = app
            .manage_mods
            .iter()
            .map(|m| {
                let prefix = if m.enabled { "✔ " } else { "  " };
                let text = format!(
                    "{}{} ({})",
                    prefix, m.installed_mod.title, m.installed_mod.version
                );
                ListItem::new(text)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Installed Mods ")
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");

        let mut state = ListState::default();
        state.select(app.manage_selected);
        frame.render_stateful_widget(list, area, &mut state);
    } else {
        // We have pending mods — build a combined list with a separator
        let mut items: Vec<ListItem> = Vec::new();
        let mut index_map: Vec<usize> = Vec::new(); // maps list row -> manage_mods index

        // Saved mods first
        for (i, m) in app.manage_mods.iter().enumerate() {
            if m.pending {
                continue;
            }
            let prefix = if m.enabled { "✔ " } else { "  " };
            let text = format!(
                "{}{} ({})",
                prefix, m.installed_mod.title, m.installed_mod.version
            );
            items.push(ListItem::new(text));
            index_map.push(i);
        }

        // Separator
        let sep_text = Line::from(Span::styled(
            "── Newly Installed (Ctrl+S to save) ──",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ));
        items.push(ListItem::new(sep_text));

        // Pending mods
        for (i, m) in app.manage_mods.iter().enumerate() {
            if !m.pending {
                continue;
            }
            let prefix = if m.enabled { "✔ " } else { "  " };
            let text = Line::from(Span::styled(
                format!(
                    "{}{} ({})",
                    prefix, m.installed_mod.title, m.installed_mod.version
                ),
                Style::default().fg(Color::Cyan),
            ));
            items.push(ListItem::new(text));
            index_map.push(i);
        }

        // Convert manage_selected (index into manage_mods) to display row
        let display_selected = app.manage_selected.and_then(|sel| {
            index_map.iter().position(|&idx| idx == sel)
        });

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Installed Mods ")
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");

        let mut state = ListState::default();
        state.select(display_selected);
        frame.render_stateful_widget(list, area, &mut state);
    }
}

fn draw_install_tab(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar
            Constraint::Min(1),   // mod list
        ])
        .split(area);

    // Search bar
    let is_search_focused = app.active_block == ActiveBlock::InstallSearch;
    let search_border = if is_search_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let search_text = if app.install_filter.is_empty() && !is_search_focused {
        Span::styled(
            "Type / to search...",
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw(&app.install_filter)
    };

    let search = Paragraph::new(Line::from(search_text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search ")
                .border_style(search_border),
        );
    frame.render_widget(search, chunks[0]);

    // Show cursor in search bar when focused
    if is_search_focused {
        let cursor_x = chunks[0].x + 1 + app.install_filter.len() as u16;
        let cursor_y = chunks[0].y + 1;
        frame.set_cursor_position((cursor_x, cursor_y));
    }

    // Mod list
    if app.loading {
        let loading = Paragraph::new("Loading mod list...")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Mod Portal "));
        frame.render_widget(loading, chunks[1]);
        return;
    }

    let filtered = app.filtered_install_mods();

    if filtered.is_empty() {
        let msg = if app.install_filter.is_empty() {
            "No mods available"
        } else {
            "No mods match filter"
        };
        let empty = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Mod Portal "));
        frame.render_widget(empty, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|m| {
            let prefix = if app.is_installed(&m.name) {
                "✔ "
            } else {
                "  "
            };
            let text = format!("{}{}", prefix, m.title);
            // Truncate long titles (char-aware to avoid splitting multi-byte chars)
            let max_len = (chunks[1].width as usize).saturating_sub(4);
            let display = if text.chars().count() > max_len {
                let truncated: String = text.chars().take(max_len.saturating_sub(1)).collect();
                format!("{}…", truncated)
            } else {
                text
            };
            ListItem::new(display)
        })
        .collect();

    let is_list_focused = app.active_block == ActiveBlock::InstallModList;
    let list_border = if is_list_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Mod Portal ({}) ", filtered.len()))
                .border_style(list_border),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

    let mut state = ListState::default();
    state.select(app.install_selected);
    frame.render_stateful_widget(list, chunks[1], &mut state);
}

fn draw_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    let text = if let Some((msg, _)) = &app.status_message {
        Span::styled(msg.as_str(), Style::default().fg(Color::Yellow))
    } else {
        let hints = match app.tab {
            Tab::Manage => {
                "Tab: switch tabs | ↑↓: navigate | Enter: toggle | d: delete | Ctrl+S: save | Ctrl+C: quit"
            }
            Tab::Install => {
                "Tab: switch tabs | ↑↓: navigate | Enter: install | /: search | Ctrl+S: save | Ctrl+C: quit"
            }
        };
        Span::styled(hints, Style::default().fg(Color::DarkGray))
    };

    let paragraph = Paragraph::new(Line::from(text));
    frame.render_widget(paragraph, area);
}

fn draw_quit_popup(frame: &mut Frame) {
    let area = centered_rect(40, 7, frame.area());
    frame.render_widget(Clear, area);

    let popup = Paragraph::new(vec![
        Line::from(""),
        Line::from("  Save changes before quitting?"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Save & quit  "),
            Span::styled("[n]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit  "),
            Span::styled("[Esc]", Style::default().fg(Color::DarkGray)),
            Span::raw(" Cancel"),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Quit ")
            .border_style(Style::default().fg(Color::Yellow)),
    );

    frame.render_widget(popup, area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
