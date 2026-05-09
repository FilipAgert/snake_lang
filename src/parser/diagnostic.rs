use crate::lexer::token::Span;
use crate::parser::expression::ExpressionError;
use crate::parser::semantic_analyser::SemanticError;
use crate::parser::statement::StatementError;
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
pub struct Error {
    pub error_t: ErrorT,
    pub span: Span,
}
pub struct Diagnostic {
    errors: Vec<Error>,
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
}
