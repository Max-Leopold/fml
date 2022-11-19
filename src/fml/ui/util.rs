use std::cmp;

use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::{Span, Spans, Text};

pub fn centered_rect(size_x: u16, size_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(cmp::max(0, (r.height as i16 - size_y as i16) / 2) as u16),
                Constraint::Length(size_y),
                Constraint::Length(cmp::max(0, (r.height as i16 - size_y as i16) / 2) as u16),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(cmp::max(0, (r.width as i16 - size_x as i16) / 2) as u16),
                Constraint::Length(size_x),
                Constraint::Length(cmp::max(0, (r.width as i16 - size_x as i16) / 2) as u16),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_text(text: Text, size_x: usize, size_y: usize, wrap: Option<bool>) -> Text {
    let lines = text.lines;
    let mut line_count = 0;
    for line in &lines {
        if wrap.is_some() && wrap.unwrap() {
            line_count += (line.width() + size_x as usize - 1) / size_x as usize;
        } else {
            line_count += 1;
        }
    }
    let top_padding = (size_y - line_count) / 2;
    let bottom_padding = size_y - line_count - top_padding;
    let mut new_lines = vec![];
    for _ in 0..top_padding {
        new_lines.push(Spans::from(Span::raw("")));
    }
    for line in lines {
        new_lines.push(line);
    }
    for _ in 0..bottom_padding {
        new_lines.push(Spans::from(Span::raw("")));
    }
    Text::from(new_lines)
}
