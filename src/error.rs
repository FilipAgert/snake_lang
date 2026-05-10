use crate::lexer::token::{Bracket, Keyword, Operator, Span, Symbol, TokenType};
use crate::parser::semantic_analyser::ExpressionType;
use std::fmt;
#[derive(Debug)]
pub enum ErrorT {
    SemanticError(SemanticError),
    StatementError(StatementError),
    ExpressionError(ExpressionError),
}
#[derive(Debug)]
pub struct Error {
    pub error_t: ErrorT,
    pub span: Span,
}
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
    UseBeforeDefinition(Box<str>),
    AlreadyDefinedInScope(Box<str>),
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
#[derive(Debug, Clone)]
pub enum StatementError {
    ExpressionError(ExpressionError),
    UnexpectedToken(TokenType),
    ExpectedToken { expected: TokenType, got: TokenType },
    ExpectedBlockHere,
}
#[derive(Debug, Clone)]
pub enum ExpressionError {
    BinaryOperandOnLhsError(Operator),
    UnexpectedKeyword(Keyword),
    UnexpectedSymbol(Symbol),
    UnexpectedEOF,
    MissingClosingBrace(Bracket),
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
impl From<ExpressionError> for StatementError {
    fn from(error: ExpressionError) -> Self {
        StatementError::ExpressionError(error)
    }
}

impl std::fmt::Display for ErrorT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorT::ExpressionError(e) => write!(f, "{}", e),
            ErrorT::SemanticError(e) => write!(f, "{}", e),
            ErrorT::StatementError(e) => write!(f, "{}", e),
        }
    }
}

impl std::fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionError::BinaryOperandOnLhsError(o) => {
                write!(f, "The operator {} cannot be applied on one operand.", o)
            }
            ExpressionError::MissingClosingBrace(b) => write!(f, "The brace {} was expected.", b),
            ExpressionError::UnexpectedEOF => write!(f, "Did not expect to reach EOF here."),
            ExpressionError::UnexpectedKeyword(k) => write!(f, "Did not expect keyword {}.", k),
            ExpressionError::UnexpectedSymbol(s) => write!(f, "Did not expect symbol {}", s),
        }
    }
}
impl fmt::Display for ExpressionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExpressionType::Standard(t) => write!(f, "{}", t),
            ExpressionType::Pointer(inner) => write!(f, "*{}", inner),
            ExpressionType::Custom(name) => write!(f, "{}", name),
            ExpressionType::Error => write!(f, "unknown"),
        }
    }
}
impl std::fmt::Display for StatementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementError::ExpectedBlockHere => write!(
                f,
                "Expected a block {} starting here.",
                Bracket::CurlyBrace(crate::lexer::token::Side::Left)
            ),
            StatementError::ExpectedToken { expected, got } => {
                write!(f, "Expected {} but got {}.", expected, got)
            }
            StatementError::ExpressionError(expr) => write!(f, "{}", expr),
            StatementError::UnexpectedToken(t) => write!(f, "Unexpectedly received {}.", t),
        }
    }
}
impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::AlreadyDefinedInScope(id) => {
                write!(f, "The identifier {} is already defined in this scope.", id)
            }
            SemanticError::IncompatibleReturnType { fun_sig, attempted } => {
                write!(
                    f,
                    "Function has return type {} while return statement returns type {}.",
                    fun_sig, attempted
                )
            }
            SemanticError::IncompatibleTypes { left, right } => {
                write!(
                    f,
                    "Lhs has type: {} which is incompatible with {}.",
                    left, right
                )
            }
            SemanticError::TooFewArguments {
                desired, provided, ..
            }
            | SemanticError::TooManyArguments {
                limit: desired,
                provided,
            } => {
                write!(
                    f,
                    "{} arguments provided to a function requiring {}.",
                    provided, desired
                )
            }
            SemanticError::UseBeforeDefinition(id) => {
                write!(f, "Use of {} before its definition", id)
            }
            SemanticError::InvalidArgumentType {
                arg_type,
                parameter_type,
                ..
            } => {
                write!(
                    f,
                    "Attempting to call function with type {} when signature requires {}",
                    arg_type, parameter_type
                )
            }
        }
    }
}
