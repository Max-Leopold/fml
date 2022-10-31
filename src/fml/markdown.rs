use pulldown_cmark::{Event, Tag};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};

/// Iterator that parse a markdown text and outputs styled spans.
pub struct Parser<'a> {
    first: bool,
    input: &'a str,
}

impl<'a> Parser<'a> {
    /// Creates a new parser with the given input text.
    pub fn new(input: &'a str) -> Self {
        Parser {
            first: true,
            input,
        }
    }

    pub fn to_spans(&mut self) -> Vec<Spans<'a>> {
        let mut res: Vec<Spans> = Vec::new();
        let mut style = Style::default();

        self.input.lines().for_each(|line| {
            let mut spans = vec![];
            let mut parser = pulldown_cmark::Parser::new(line);

            while let Some(event) = parser.next() {
                match event {
                    Event::Start(tag) => match tag {
                        Tag::Emphasis => {
                            style = style.add_modifier(Modifier::ITALIC);
                        }
                        Tag::Heading(level, ..) => {
                            res.push(Spans::default());
                            spans.push(
                                self.literal(format!("{} ", heading(level as usize)), &style),
                            );
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
                        Tag::Heading(..) => {}
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
                    Event::SoftBreak => {}
                    Event::HardBreak => {}
                    // Treat all text the same
                    Event::FootnoteReference(text)
                    | Event::Html(text)
                    | Event::Text(text)
                    | Event::Code(text) => {
                        spans.push(self.literal(text.to_string(), &style));
                    }
                    Event::TaskListMarker(checked) => {
                        let mark = if checked { "[x]" } else { "[ ]" };
                        spans.push(self.literal(mark, &style));
                    }
                }
            }
            res.push(Spans::from(spans));
        });

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
