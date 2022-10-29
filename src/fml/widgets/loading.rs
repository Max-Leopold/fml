use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::text::Text;
use tui::widgets::{Block, Widget};

#[derive(Debug, Clone, Default)]
pub struct Loading<'a> {
    block: Option<Block<'a>>,
    style: Style,
}

impl<'a> Loading<'a> {
    pub fn block(mut self, block: Block<'a>) -> Loading<'a> {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Loading<'a> {
        self.style = style;
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
            },
            None => area
        };
        if loading_area.height < 1 {
            return;
        }

        let loading_text = Text::from("Loading...");
        let loading_width = loading_text.width() as u16;
        let loading_height = loading_text.height() as u16;
        let loading_x = loading_area.x + (loading_area.width / 2) - (loading_width / 2);
        let loading_y = loading_area.y + (loading_area.height / 2) - (loading_height / 2);
        buf.set_string(loading_x, loading_y, "Loading...", self.style);
    }
}
