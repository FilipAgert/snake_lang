#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u32,
    pub col: u32,
    pub len: u32,
}

impl Token {
    pub fn new(type_t: TokenType) -> Self {
        Self {
            token_type: type_t,
            col: 0,
            line: 0,
            len: 0,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum TokenType {
    Keyword(Keyword),
    Identifier(String),
    Op(Operator),
    Integer(i64),
    Symbol(Symbol),
    EOF,
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
        if let Ok(int) = str.parse::<i64>() {
            return TokenType::Integer(int);
        }
        TokenType::Identifier(str.to_string())
    }
}
#[derive(Debug, PartialEq)]
pub enum Keyword {
    Int,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,  // +
    Equal, // =
}

impl Operator {
    pub fn from_ch(c: char) -> Option<Operator> {
        match c {
            '=' => Some(Operator::Equal),
            '+' => Some(Operator::Plus),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
    Bracket(Bracket),
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
        match s {
            "int" => Some(Keyword::Int),
            _ => None,
        }
    }
}
