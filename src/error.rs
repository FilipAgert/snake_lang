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
        missing_types: Vec<ExpressionType>,
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
                "Expected a code block '{}...{}' starting here.",
                Bracket::CurlyBrace(crate::lexer::token::Side::Left),
                Bracket::CurlyBrace(crate::lexer::token::Side::Right)
            ),
            StatementError::ExpectedToken { expected, got } => {
                write!(f, "Expected '{}' but got '{}'.", expected, got)
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
                write!(
                    f,
                    "The identifier '{}' is already defined in this scope.",
                    id
                )
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
                write!(f, "Use of '{}' before its definition.", id)
            }
            SemanticError::InvalidArgumentType {
                arg_type,
                parameter_type,
                ..
            } => {
                write!(
                    f,
                    "Attempting to call function with type {} when signature requires {}.",
                    arg_type, parameter_type
                )
            }
        }
    }
}

impl SemanticError {
    pub fn detailed_text(&self) -> Option<String> {
        match self {
            SemanticError::AlreadyDefinedInScope(..) => Some(format!("already defined here")),
            SemanticError::IncompatibleReturnType { fun_sig, attempted } => {
                Some(format!("should be type {} not {}", fun_sig, attempted))
            }
            SemanticError::IncompatibleTypes { left, right } => {
                Some(format!("{} and {}", left, right))
            }
            SemanticError::InvalidArgumentType { arg_type, .. } => {
                Some(format!("arg of type {}.", arg_type))
            }
            SemanticError::TooFewArguments {
                desired, provided, ..
            } => Some(format!("missing {} argument(s)", desired - provided)),
            SemanticError::TooManyArguments { limit, provided } => {
                Some(format!("{} excess argument(s)", provided - limit))
            }
            SemanticError::UseBeforeDefinition(id) => Some(format!("{} used here", id)),
        }
    }
    pub fn secondary_text(&self) -> Option<String> {
        match self {
            SemanticError::AlreadyDefinedInScope(id) => {
                Some(format!("original definition of '{}' is here", id))
            }
            SemanticError::InvalidArgumentType { parameter_type, .. } => {
                Some(format!("parameter of type {}", parameter_type))
            }
            SemanticError::IncompatibleReturnType { fun_sig, .. } => Some(format!(
                "function signature specifies {} return type",
                fun_sig
            )),
            SemanticError::TooFewArguments { missing_types, .. } => {
                let arglist = missing_types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Some(format!("missing arguments: [{}]", arglist))
            }
            SemanticError::TooManyArguments { limit, .. } => {
                Some(format!("the {} legal arguments", limit))
            }
            _ => None,
        }
    }
    pub fn secondary_span(&self) -> Option<&Span> {
        match self {
            SemanticError::AlreadyDefinedInScope(..) => {
                todo!("Don't have original definition span.")
            }
            SemanticError::IncompatibleReturnType { .. } => {
                todo!("Don't have span of function signature.")
            }
            SemanticError::InvalidArgumentType { parameter_span, .. } => Some(parameter_span),
            SemanticError::TooFewArguments { missing_span, .. } => Some(missing_span),
            SemanticError::UseBeforeDefinition { .. } => {
                todo!("Maybe should check where defined later")
            }
            _ => None,
        }
    }
}
impl ExpressionError {
    pub fn detailed_text(&self) -> Option<String> {
        match self {
            ExpressionError::BinaryOperandOnLhsError(o) => {
                Some(format!("Operator '{}' requires two operands", o))
            }
            ExpressionError::MissingClosingBrace(b) => Some(format!("Unclosed '{}' bracket", b)),
            ExpressionError::UnexpectedEOF => Some("File ended unexpectedly".to_string()),
            ExpressionError::UnexpectedKeyword(k) => Some(format!("'{}' is not valid here", k)),
            ExpressionError::UnexpectedSymbol(s) => Some(format!("Symbol '{}' is out of place", s)),
        }
    }

    pub fn secondary_text(&self) -> Option<String> {
        match self {
            ExpressionError::MissingClosingBrace(b) => {
                Some(format!("This '{}' remains unclosed", b))
            }
            // Usually, Unexpected tokens don't have a secondary span
            // unless you track the start of the current expression.
            _ => None,
        }
    }
    pub fn secondary_span(&self) -> Option<&Span> {
        None
    }
}

impl StatementError {
    pub fn detailed_text(&self) -> Option<String> {
        match self {
            StatementError::ExpectedBlockHere => {
                Some("Body of statement must start with '{'".to_string())
            }
            StatementError::ExpectedToken { expected, .. } => {
                Some(format!("Expected '{}' here", expected))
            }
            StatementError::ExpressionError(expr) => expr.detailed_text(),
            StatementError::UnexpectedToken(t) => Some(format!("Did not expect '{}' here", t)),
        }
    }
    pub fn secondary_text(&self) -> Option<String> {
        match self {
            StatementError::ExpressionError(expr) => expr.secondary_text(),
            _ => None,
        }
    }
    pub fn secondary_span(&self) -> Option<&Span> {
        None
    }
}

impl ErrorT {
    pub fn detailed_text(&self) -> Option<String> {
        match self {
            ErrorT::ExpressionError(e) => e.detailed_text(),
            ErrorT::StatementError(e) => e.detailed_text(),
            ErrorT::SemanticError(e) => e.detailed_text(),
        }
    }

    pub fn secondary_text(&self) -> Option<String> {
        match self {
            ErrorT::ExpressionError(e) => e.secondary_text(),
            ErrorT::StatementError(e) => e.secondary_text(),
            ErrorT::SemanticError(e) => e.secondary_text(),
        }
    }

    pub fn secondary_span(&self) -> Option<&Span> {
        match self {
            ErrorT::ExpressionError(e) => e.secondary_span(),
            ErrorT::StatementError(e) => e.secondary_span(),
            ErrorT::SemanticError(e) => e.secondary_span(),
        }
    }
}
