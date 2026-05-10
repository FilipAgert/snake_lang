use std::panic::PanicHookInfo;

use crate::error::*;
use crate::lexer::token::Span;

#[derive(Debug)]
pub struct Diagnostic {
    errors: Vec<Error>,
}

fn get_line_col(source: &str, index: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, c) in source.char_indices() {
        if i > index {
            break;
        }

        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub enum HighlightColor {
    Red,
    Yellow,
}

impl HighlightColor {
    fn code(&self) -> &str {
        match self {
            HighlightColor::Red => "31",
            HighlightColor::Yellow => "33",
        }
    }
}

fn format_highlight(
    source: &str,
    start: usize,
    end: usize,
    line_num: usize,
    color: HighlightColor,
    label: Option<&str>,
) -> String {
    let color_code = color.code();

    // 1. Find line boundaries
    let line_start = source[..start].rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    let line_end = source[start..]
        .find('\n')
        .map(|idx| idx + start)
        .unwrap_or(source.len());

    // 2. Extract segments
    let before = &source[line_start..start];
    let highlight = &source[start..end];
    let after = &source[end.min(line_end)..line_end];

    // 3. Build the string
    let gutter_width = 4;
    let mut output = String::new();

    // Line 1: Empty gutter
    output.push_str(&format!("{:width$} |\n", "", width = gutter_width));

    // Line 2: The source code line with color
    output.push_str(&format!(
        "{:>width$} | {}\x1b[1;{}m{}\x1b[0m{}\n",
        line_num,
        before,
        color_code,
        highlight,
        after,
        width = gutter_width
    ));

    // Line 3: The caret and optional label
    let padding = " ".repeat(before.chars().count());
    let carets = "^".repeat(highlight.chars().count());
    let label_text = label.unwrap_or("");

    output.push_str(&format!(
        "{:width$} | {}\x1b[1;{}m{} {}\x1b[0m",
        "",
        padding,
        color_code,
        carets,
        label_text,
        width = gutter_width
    ));

    output
}
pub fn print_error(source: &str, error: &Error) {
    let (line_s, col_s) = get_line_col(source, error.span.start);

    println!("\x1b[1;31mError:\x1b[0m {:?}", error.error_t);
    println!("  --> line {}:{}", line_s, col_s);

    let snippet = format_highlight(
        source,
        error.span.start,
        error.span.end,
        line_s,
        HighlightColor::Red,
        Some("expected a semicolon"), // Or None
    );

    println!("{}", snippet);
}

impl Diagnostic {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn push<T>(&mut self, span: Span, error_t: T)
    where
        T: Into<ErrorT>,
    {
        let converted_error: ErrorT = error_t.into();
        self.errors.push(Error {
            error_t: converted_error,
            span,
        });
    }

    pub fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }

    pub fn get_errors(&self) -> &[Error] {
        &self.errors.as_slice()
    }

    pub fn print_errors(&self, source: &str) {
        for error in &self.errors {
            print_error(source, error);
        }
    }
}
