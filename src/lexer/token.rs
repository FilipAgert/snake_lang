#[derive(Debug, PartialEq)]
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
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Keyword(Keyword),
    Identifier(String),
    Op(Operator),
    Literal(Literal),
    Symbol(Symbol),
    EOF,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(i32),
}

impl Literal {
    pub fn from_str(str: &str) -> Option<Self> {
        if let Ok(int) = str.parse::<i32>() {
            return Some(Literal::Integer(int));
        }
        None
    }
}

impl TokenType {
    pub fn from_str(str: &str) -> Self {
        if str.len() == 1 {
            let ch = str.chars().next().expect("Already checked length");
            if let Some(op) = Operator::from_ch(ch) {
                return TokenType::Op(op);
            }
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
        TokenType::Identifier(str.to_string())
    }
}
#[derive(Debug, PartialEq)]
pub enum Keyword {
    Declaration(DeclarationKeyword),
    FunctionDeclaration,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DeclarationKeyword {
    Int,
    Void,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Plus,   // +
    Equal,  // =
    Times,  // *
    Divide, // /
    Minus,  // -
}

impl Operator {
    pub fn from_ch(c: char) -> Option<Operator> {
        match c {
            '=' => Some(Operator::Equal),
            '+' => Some(Operator::Plus),
            '/' => Some(Operator::Divide),
            '-' => Some(Operator::Minus),
            '*' => Some(Operator::Times),
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
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Bracket(Bracket),
    Comma,
    Colon,
    Semicolon,
}
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
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
            _ => None,
        }
    }
}

impl DeclarationKeyword {
    pub fn from_str(s: &str) -> Option<DeclarationKeyword> {
        match s {
            "int" => Some(DeclarationKeyword::Int),
            "void" => Some(DeclarationKeyword::Void),
            _ => None,
        }
    }
}
