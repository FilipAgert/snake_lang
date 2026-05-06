#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u32,
    pub col: u32,
    pub len: u32,
}

impl Token {
    pub fn new(type_t: TokenType) -> Token {
        Token {
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
#[derive(Debug, PartialEq)]
pub enum Keyword {
    Int,
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Plus,  // +
    Equal, // =
}

#[derive(Debug, PartialEq)]
pub enum Symbol {}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s {
            "int" => Some(Keyword::Int),
            _ => None,
        }
    }
}
