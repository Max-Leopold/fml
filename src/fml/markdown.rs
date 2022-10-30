use pulldown_cmark::{Event, Tag};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};

/// Iterator that parse a markdown text and outputs styled spans.
pub struct Parser<'a> {
    first: bool,
    parser: pulldown_cmark::Parser<'a, 'a>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser with the given input text.
    pub fn new(input: &'a str) -> Self {
        Parser {
            first: true,
            parser: pulldown_cmark::Parser::new(input),
        }
    }

    pub fn to_spans(&mut self) -> Vec<Spans<'a>> {
        let mut res: Vec<Spans> = Vec::new();
        let mut spans: Vec<Span> = vec![];
        let mut style = Style::default();

        while let Some(event) = &self.parser.next() {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Emphasis => {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    Tag::Heading(level, ..) => {
                        res.push(Spans::default());
                        spans.push(self.literal(format!("{} ", heading(*level as usize)), &style));
                    }
                    Tag::BlockQuote => spans.push(self.literal("> ", &style)),
                    Tag::Link(_, _, _) => spans.push(self.literal("[", &style)),
                    Tag::CodeBlock(_) => spans.push(self.literal("```", &style)),
                    Tag::Strong => {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    Tag::Paragraph => {
                        if !self.first {
                            res.push(Spans::from(spans));

                            spans = vec![];
                        }
                    }
                    _ => (),
                },
                Event::End(tag) => match tag {
                    Tag::Paragraph => {
                        if self.first {
                            self.first = false
                        }
                    }
                    Tag::Heading(..) => {
                        res.push(Spans::from(spans));

                        spans = vec![];
                    }
                    Tag::Link(_, link, _) => {
                        spans.push(self.literal(format!("]({})", link), &style))
                    }
                    Tag::CodeBlock(_) => spans.push(self.literal("```", &style)),
                    Tag::Emphasis => {
                        style = style.remove_modifier(Modifier::ITALIC);
                    }
                    Tag::Strong => {
                        style = style.remove_modifier(Modifier::BOLD);
                    }
                    _ => (),
                },
                Event::Rule => spans.push(self.literal("---", &style)),
                Event::SoftBreak => {
                    res.push(Spans::from(spans));
                    spans = vec![];
                }
                Event::HardBreak => {
                    res.push(Spans::from(spans));
                    spans = vec![];
                }
                // Treat all text the same
                Event::FootnoteReference(text)
                | Event::Html(text)
                | Event::Text(text)
                | Event::Code(text) => {
                    // Split text into lines. If the text spans mutliple lines we need to add a
                    // we have to create a span for each line.
                    let lines = text.split('\n');
                    for (i, line) in lines.enumerate() {
                        if i > 0 {
                            res.push(Spans::from(spans));
                            spans = vec![];
                        }
                        spans.push(self.literal(line, &style));
                    }
                }
                Event::TaskListMarker(checked) => {
                    let mark = if *checked { "[x]" } else { "[ ]" };
                    spans.push(self.literal(mark, &style));
                }
            }
        }
        // Add the last line
        res.push(Spans::from(spans));

        res
    }

    /// Creates a new span with the given value
    fn literal<S>(&self, text: S, style: &Style) -> Span<'a>
    where
        S: Into<String>,
    {
        let text = text.into();
        Span::styled(text, style.clone())
    }
}

fn heading(level: usize) -> &'static str {
    &"##########"[..level]
}
