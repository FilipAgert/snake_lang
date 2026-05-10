#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Span {
    pub start: usize, // start (inclusive)
    pub end: usize,   // (exclusive)
}
impl Span {
    pub fn merge(s1: &Self, s2: &Self) -> Self {
        let start = s1.start.min(s2.start);
        let end = s1.end.max(s2.end);
        Self { start, end }
    }
    pub const fn min_info() -> Self {
        Self {
            start: usize::MAX,
            end: usize::MIN,
        }
    }
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
        if let Some(op) = Operator::from_str(str) {
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
    Declaration(DeclarationKeyword),
    FunctionDeclaration,
    Return,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DeclarationKeyword {
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

impl Operator {
    pub fn from_str(str: &str) -> Option<Operator> {
        match str {
            "=" => Some(Operator::Equal),
            "+" => Some(Operator::Plus),
            "/" => Some(Operator::Divide),
            "-" => Some(Operator::Minus),
            "*" => Some(Operator::Times),
            "!" => Some(Operator::Not),
            "<" => Some(Operator::Lt),
            "<=" => Some(Operator::Le),
            ">" => Some(Operator::Gt),
            ">=" => Some(Operator::Ge),
            "==" => Some(Operator::EqualEqual),
            "!=" => Some(Operator::NotEqual),
            "&&" => Some(Operator::And),
            "||" => Some(Operator::Or),
            "^" => Some(Operator::Xor),
            _ => None,
        }
    }

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
        if let Some(decl_key) = DeclarationKeyword::from_str(s) {
            return Some(Keyword::Declaration(decl_key));
        }
        match s {
            "fn" => Some(Keyword::FunctionDeclaration),
            "return" => Some(Keyword::Return),
            _ => None,
        }
    }
}

impl DeclarationKeyword {
    pub fn from_str(s: &str) -> Option<DeclarationKeyword> {
        match s {
            "int" => Some(DeclarationKeyword::Int),
            "void" => Some(DeclarationKeyword::Void),
            "bool" => Some(DeclarationKeyword::Bool),
            _ => None,
        }
    }
}

impl From<Literal> for DeclarationKeyword {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Integer(..) => DeclarationKeyword::Int,
            Literal::Bool(..) => DeclarationKeyword::Bool,
        }
    }
}
