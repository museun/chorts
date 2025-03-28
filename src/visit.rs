use std::borrow::Cow;

use crate::data::{Code, Level, Message, Reason, Span, Text};

pub trait Visitor {
    fn visit_message(&mut self, message: &Message) {
        let _ = message;
    }

    fn visit_level(&mut self, level: &Level) {
        let _ = level;
    }

    fn visit_code(&mut self, code: &str) {
        let _ = code;
    }

    fn visit_span(&mut self, file: Filename<'_>, text: &[Text]) {
        let _ = file;
        let _ = text;
    }

    fn visit_text(&mut self, text: Highlight<'_>) {
        let _ = text;
    }
}

pub trait Visit {
    fn accept(&self, visitor: &mut impl Visitor);
}

impl<T: Visit> Visit for Option<T> {
    fn accept(&self, visitor: &mut impl Visitor) {
        if let Some(this) = self {
            this.accept(visitor);
        }
    }
}

impl<T: Visit> Visit for Vec<T> {
    fn accept(&self, visitor: &mut impl Visitor) {
        for value in self {
            value.accept(visitor);
        }
    }
}

impl<T: Visit> Visit for [T] {
    fn accept(&self, visitor: &mut impl Visitor) {
        for value in self {
            value.accept(visitor);
        }
    }
}

impl Visit for Reason {
    fn accept(&self, visitor: &mut impl Visitor) {
        if let Self::CompilerMessage { message } = self {
            visitor.visit_message(message);
        }
    }
}

impl Visit for Message {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_message(self);
        for child in &self.children {
            visitor.visit_message(child);
        }
    }
}

impl Visit for Code {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_code(&self.code);
    }
}

impl Visit for Level {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_level(self);
    }
}

impl Visit for Span {
    fn accept(&self, visitor: &mut impl Visitor) {
        visitor.visit_span(
            Filename {
                name: Cow::Borrowed(&self.file_name),
                row: self.line_start,
                col: self.column_start,
            },
            &self.text,
        );
    }
}

impl Visit for Text {
    fn accept(&self, visitor: &mut impl Visitor) {
        if self.highlight_end.saturating_sub(self.highlight_start) == 0 {
            return;
        }
        visitor.visit_text(Highlight {
            data: Cow::Borrowed(&self.text),
            start: self.highlight_start,
            end: self.highlight_end,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Filename<'a> {
    pub name: Cow<'a, str>,
    pub row: usize,
    pub col: usize,
}

impl Filename<'_> {
    pub fn to_owned(self) -> Filename<'static> {
        Filename {
            name: Cow::Owned(self.name.to_string()),
            row: self.row,
            col: self.col,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Highlight<'a> {
    pub data: Cow<'a, str>,
    pub start: usize,
    pub end: usize,
}

impl Highlight<'_> {
    pub fn to_owned(self) -> Highlight<'static> {
        Highlight {
            data: Cow::Owned(self.data.to_string()),
            start: self.start,
            end: self.end,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Partition<'a> {
    pub head: &'a str,
    pub middle: &'a str,
    pub tail: &'a str,
}

pub fn partition<'a, 'b>(
    spans: &'b [Highlight<'a>],
) -> impl Iterator<Item = Partition<'a>> + use<'a, 'b>
where
    'b: 'a,
{
    let mut iter = spans.iter().enumerate();
    let mut left_pad = 0;

    std::iter::from_fn(move || loop {
        let (i, span) = iter.next()?;
        if span.data.trim_start().is_empty() {
            continue;
        }

        if i == 0 {
            let s = span.data.trim_start();
            left_pad = span.data.len() - s.len();
        }

        let start = span.start.saturating_sub(left_pad + 1);
        let end = span.end.saturating_sub(left_pad + 1);

        let start = str_indices::chars::from_byte_idx(&span.data, start);
        let end = str_indices::chars::from_byte_idx(&span.data, end);

        let text = &span.data[left_pad..];
        let start = floor_char_boundary(text, start);
        let end = ceil_char_boundary(text, end);

        break Some(Partition {
            head: &text[..start],
            middle: &text[start..end],
            tail: &text[end..],
        });
    })
}

fn floor_char_boundary(str: &str, index: usize) -> usize {
    if index >= str.len() {
        return str.len();
    }

    let start = index.saturating_sub(3);
    let next = str.as_bytes()[start..=index]
        .iter()
        .rposition(|&b| (b as u8) >= -0x40)
        .unwrap();
    start + next
}

fn ceil_char_boundary(str: &str, index: usize) -> usize {
    if index > str.len() {
        return str.len();
    }

    let end = (index + 4).min(str.len());
    str.as_bytes()[index..end]
        .iter()
        .position(|&b| (b as u8) >= -0x40)
        .map_or(end, |pos| pos + index)
}

// TODO relocate the highlight
