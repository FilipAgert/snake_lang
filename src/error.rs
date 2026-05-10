use crate::lexer::token::{Keyword, Operator, Span, Symbol, TokenType};
use crate::parser::semantic_analyser::ExpressionType;
#[derive(Debug)]
pub enum SemanticError {
    IncompatibleTypes {
        left: ExpressionType,
        right: ExpressionType,
    },
    IncompatibleReturnType {
        fun_sig: ExpressionType,
        attempted: ExpressionType,
    },
    UseBeforeDefinition,
    AlreadyDefinedInScope,
    TooManyArguments {
        limit: usize,
        provided: usize,
    },
    TooFewArguments {
        desired: usize,
        provided: usize,
        missing_span: Span,
    },
    InvalidArgumentType {
        arg_type: ExpressionType,
        parameter_type: ExpressionType,
        parameter_span: Span,
    },
}
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
#[derive(Debug, Clone)]
pub enum StatementError {
    ExpressionError(ExpressionError),
    UnexpectedEOF,
    UnexpectedToken(TokenType),
    ExpectedToken { expected: TokenType, got: TokenType },
    AssignmentToNonId,
    ExpectedBlockHere,
}

impl From<ExpressionError> for StatementError {
    fn from(error: ExpressionError) -> Self {
        StatementError::ExpressionError(error)
    }
}
#[derive(Debug, Clone)]
pub enum ExpressionError {
    MissingOperand,
    BinaryOperandOnLhsError(Operator),
    UnexpectedKeyword(Keyword),
    UnexpectedSymbol(Symbol),
    UnexpectedEOF,
    MissingClosingBrace(TokenType),
}
