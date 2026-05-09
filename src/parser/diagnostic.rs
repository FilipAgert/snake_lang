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

    let line_str = if line_s == line_e {
        format!("{}:{}-{}", line_s, col_s, col_e)
    } else {
        format!("{}:{}-{}:{}", line_s, col_s, line_e, col_e)
    };
    println!("Error at {}: {:?}", line_str, error.error_t);
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
