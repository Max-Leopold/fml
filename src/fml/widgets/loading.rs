use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::text::Span;
use tui::widgets::{Block, Widget};

#[derive(Debug, Clone)]
pub struct Loading<'a> {
    block: Option<Block<'a>>,
    style: Style,
    ticks: u64,
    loading_symbols: Vec<&'a str>,
}

impl<'a> Loading<'a> {
    pub fn new() -> Loading<'a> {
        Loading {
            block: None,
            style: Style::default(),
            ticks: 0,
            loading_symbols: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Loading<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Loading<'a> {
        self.style = style;
        self
    }

    pub fn ticks(mut self, ticks: u64) -> Loading<'a> {
        self.ticks = ticks;
        self
    }

    pub fn loading_symbols(mut self, loading_symbols: Vec<&'a str>) -> Loading<'a> {
        self.loading_symbols = loading_symbols;
        self
    }
}

impl<'a> Widget for Loading<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);
        let loading_area = match self.block.take() {
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };
        if loading_area.height < 1 {
            return;
        }
        let max_loading_symbol_width = self.loading_symbols.iter().map(|s| s.len()).max().unwrap_or(0) as u16;
        let max_loading_symbol_height = self.loading_symbols.iter().map(|s| s.lines().count()).max().unwrap_or(0) as u16;
        let loading_x = loading_area.x + (loading_area.width / 2) - (max_loading_symbol_width / 2);
        let loading_y = loading_area.y + (loading_area.height / 2) - (max_loading_symbol_height / 2);

        let loading_symbol = self.loading_symbols[self.ticks as usize % self.loading_symbols.len()];
        let text = Span::styled(loading_symbol, self.style);
        buf.set_span(loading_x, loading_y, &text, loading_area.width);
      }
  }
