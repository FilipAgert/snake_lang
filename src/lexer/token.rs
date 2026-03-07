#[derive(Debug, PartialEq)]
pub enum Token {
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
