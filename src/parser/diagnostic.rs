use std::panic::PanicHookInfo;

use crate::lexer::token::Span;
use crate::parser::expression::ExpressionError;
use crate::parser::semantic_analyser::SemanticError;
use crate::parser::statement::StatementError;
#[derive(Debug)]
pub enum ErrorT {
    SemanticError(SemanticError),
    StatementError(StatementError),
    ExpressionError(ExpressionError),
}
impl From<SemanticError> for ErrorT {
    fn from(value: SemanticError) -> Self {
        Self::SemanticError(value)
    }
}
impl From<StatementError> for ErrorT {
    fn from(value: StatementError) -> Self {
        Self::StatementError(value)
    }
}
impl From<ExpressionError> for ErrorT {
    fn from(value: ExpressionError) -> Self {
        Self::ExpressionError(value)
    }
}
#[derive(Debug)]
pub struct Error {
    pub error_t: ErrorT,
    pub span: Span,
}

#[derive(Debug)]
pub struct Diagnostic {
    errors: Vec<Error>,
}

fn get_line_col(source: &str, index: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, c) in source.char_indices() {
        if i >= index {
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

pub fn print_error(source: &str, error: &Error) {
    let (line_s, col_s) = get_line_col(source, error.span.start);
    let (line_e, col_e) = get_line_col(source, error.span.end);

    // 1. Find the start and end of the line containing the error
    let line_start = source[..error.span.start]
        .rfind('\n')
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let line_end = source[error.span.start..]
        .find('\n')
        .map(|idx| idx + error.span.start)
        .unwrap_or(source.len());

    let full_line = &source[line_start..line_end];

    // 2. Split the line into three parts: before, error, after
    let before = &source[line_start..error.span.start];
    let highlight = &source[error.span.start..error.span.end];
    let after = &source[error.span.end..line_end];

    // 3. Print the error header
    println!("\x1b[1;31mError:\x1b[0m {:?}", error.error_t);
    println!("  --> line {}:{}", line_s, col_s);

    // 4. Print the highlighted line
    // Use | as a margin character
    let gutter_width = 4;
    println!("{:width$} |", "", width = gutter_width);
    println!(
        "{:>width$} | {}{}\x1b[1;31m{}\x1b[0m{}",
        line_s,
        before,
        "",
        highlight,
        after,
        width = gutter_width
    );

    // 5. Print a "caret" (^) under the error
    let padding = " ".repeat(before.chars().count());
    let carets = "^".repeat(highlight.chars().count());
    println!(
        "{:width$} | {}\x1b[1;31m{}\x1b[0m",
        "",
        padding,
        carets,
        width = gutter_width
    );
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
