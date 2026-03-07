#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub line: u32,
    pub col: u8,
    pub len: u8,
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
