use crate::data::{Code, Level, Message, Reason, Span};

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
    fn visit_span(&mut self, file: &str, row: usize, col: usize) {
        let _ = file;
        let _ = row;
        let _ = col;
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
        visitor.visit_span(&self.file_name, self.line_start, self.column_start);
    }
}
