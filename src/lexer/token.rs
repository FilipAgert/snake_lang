use crate::diagnostics::span::Span;
use std::fmt;
use std::str::FromStr;
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    Keyword(Keyword),
    Identifier(Box<str>),
    Op(Operator),
    Literal(Literal),
    Symbol(Symbol),
    EOF,
}

impl From<Symbol> for TokenType {
    fn from(value: Symbol) -> Self {
        TokenType::Symbol(value)
    }
}
impl From<Bracket> for TokenType {
    fn from(value: Bracket) -> Self {
        TokenType::Symbol(Symbol::Bracket(value))
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(i32),
    Bool(bool),
}

impl Literal {
    pub fn from_str(str: &str) -> Option<Self> {
        if let Ok(int) = str.parse::<i32>() {
            return Some(Literal::Integer(int));
        } else if str.eq("true") {
            return Some(Literal::Bool(true));
        } else if str.eq("false") {
            return Some(Literal::Bool(false));
        }
        None
    }
}

impl TokenType {
    pub fn from_str(str: &str) -> Self {
        if let Ok(op) = Operator::from_str(str) {
            return TokenType::Op(op);
        }
        if str.len() == 1 {
            let ch = str.chars().next().expect("Already checked length");
            if let Some(symbol) = Symbol::from_ch(ch) {
                return TokenType::Symbol(symbol);
            }
        }
        if let Some(keyword) = Keyword::from_str(str) {
            return TokenType::Keyword(keyword);
        }
        if let Some(literal) = Literal::from_str(str) {
            return TokenType::Literal(literal);
        }
        TokenType::Identifier(Box::from(str))
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Declaration(BuiltInType),
    FunctionDeclaration,
    Return,
    If,
    Else,
    Struct,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BuiltInType {
    Int,
    Void,
    Error,
    Bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Plus,       // +
    Equal,      // =
    Times,      // *
    Divide,     // /
    Minus,      // -
    Lt,         // <
    Le,         // <=
    Gt,         // >
    Ge,         // >=
    EqualEqual, // ==
    Not,        // !
    And,        // &&
    Or,         // ||
    NotEqual,   // !=
    Xor,        // ^
}

impl FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Operator::Plus),
            "=" => Ok(Operator::Equal),
            "*" => Ok(Operator::Times),
            "/" => Ok(Operator::Divide),
            "-" => Ok(Operator::Minus),
            "<" => Ok(Operator::Lt),
            "<=" => Ok(Operator::Le),
            ">" => Ok(Operator::Gt),
            ">=" => Ok(Operator::Ge),
            "==" => Ok(Operator::EqualEqual),
            "!" => Ok(Operator::Not),
            "&&" => Ok(Operator::And),
            "||" => Ok(Operator::Or),
            "!=" => Ok(Operator::NotEqual),
            "^" => Ok(Operator::Xor),
            _ => Err(format!("Invalid operator: {}", s)),
        }
    }
}
impl Operator {
    pub fn precedence_value(self: &Self) -> i32 {
        match &self {
            Operator::Times => 20,
            Operator::Divide => 20,
            Operator::Minus => 10,
            Operator::Plus => 10,
            Operator::Equal => 0,
            Operator::Not => 30,
            Operator::And => 20,
            Operator::Or | Operator::Xor => 10,
            Operator::EqualEqual | Operator::NotEqual => 0,
            Operator::Ge | Operator::Gt | Operator::Le | Operator::Lt => 10,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Symbol {
    Bracket(Bracket),
    Comma,
    Colon,
    Semicolon,
}
impl From<Bracket> for Symbol {
    fn from(value: Bracket) -> Self {
        Symbol::Bracket(value)
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Bracket {
    Parenthesis(Side),
    Square(Side),
    CurlyBrace(Side),
}
impl Symbol {
    pub fn from_ch(c: char) -> Option<Symbol> {
        if let Some(b) = Bracket::from_ch(c) {
            return Some(Symbol::Bracket(b));
        }

        match c {
            ';' => Some(Symbol::Semicolon),
            ',' => Some(Symbol::Comma),
            ':' => Some(Symbol::Colon),
            _ => None,
        }
    }
}

impl Bracket {
    pub fn from_ch(c: char) -> Option<Bracket> {
        match c {
            '(' => Some(Bracket::Parenthesis(Side::Left)),
            ')' => Some(Bracket::Parenthesis(Side::Right)),
            '[' => Some(Bracket::Square(Side::Left)),
            ']' => Some(Bracket::Square(Side::Right)),
            '{' => Some(Bracket::CurlyBrace(Side::Left)),
            '}' => Some(Bracket::CurlyBrace(Side::Right)),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Side {
    Left,
    Right,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        if let Some(decl_key) = BuiltInType::from_str(s) {
            return Some(Keyword::Declaration(decl_key));
        }
        match s {
            "fn" => Some(Keyword::FunctionDeclaration),
            "return" => Some(Keyword::Return),
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "struct" => Some(Keyword::Struct),
            _ => None,
        }
    }
}

impl BuiltInType {
    pub fn from_str(s: &str) -> Option<BuiltInType> {
        match s {
            "int" => Some(BuiltInType::Int),
            "void" => Some(BuiltInType::Void),
            "bool" => Some(BuiltInType::Bool),
            _ => None,
        }
    }
}

impl From<Literal> for BuiltInType {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Integer(..) => BuiltInType::Int,
            Literal::Bool(..) => BuiltInType::Bool,
        }
    }
}

impl fmt::Display for BuiltInType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            BuiltInType::Int => "int",
            BuiltInType::Void => "void",
            BuiltInType::Bool => "bool",
            BuiltInType::Error => "err",
        };
        write!(f, "{}", s)
    }
}
impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Keyword::Declaration(t) => write!(f, "{}", t),
            Keyword::FunctionDeclaration => write!(f, "fn"),
            Keyword::Return => write!(f, "return"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::Struct => write!(f, "struct"),
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::Bracket(b) => write!(f, "{}", b),
            Symbol::Comma => write!(f, ","),
            Symbol::Colon => write!(f, ":"),
            Symbol::Semicolon => write!(f, ";"),
        }
    }
}
impl fmt::Display for Bracket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let c = match self {
            Bracket::Parenthesis(Side::Left) => '(',
            Bracket::Parenthesis(Side::Right) => ')',
            Bracket::Square(Side::Left) => '[',
            Bracket::Square(Side::Right) => ']',
            Bracket::CurlyBrace(Side::Left) => '{',
            Bracket::CurlyBrace(Side::Right) => '}',
        };
        write!(f, "{}", c)
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let symbol = match self {
            Operator::Plus => "+",
            Operator::Equal => "=",
            Operator::Times => "*",
            Operator::Divide => "/",
            Operator::Minus => "-",
            Operator::Lt => "<",
            Operator::Le => "<=",
            Operator::Gt => ">",
            Operator::Ge => ">=",
            Operator::EqualEqual => "==",
            Operator::Not => "!",
            Operator::And => "&&",
            Operator::Or => "||",
            Operator::NotEqual => "!=",
            Operator::Xor => "^",
        };
        write!(f, "{}", symbol)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Integer(val) => write!(f, "{}", val),
            Literal::Bool(val) => write!(f, "{}", val),
        }
    }
}
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TokenType::Keyword(k) => write!(f, "{}", k),
            TokenType::Identifier(id) => write!(f, "{}", id),
            TokenType::Op(op) => write!(f, "{}", op),
            TokenType::Literal(lit) => write!(f, "{}", lit),
            TokenType::Symbol(sym) => write!(f, "{}", sym),
            TokenType::EOF => write!(f, "EOF"),
        }
    }
}
