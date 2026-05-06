use crate::lexer::token::{Keyword, Operator, Symbol, Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    cursor: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Lexer {
        Lexer {
            cursor: s.chars().peekable(),
        }
    }

    pub fn scan(self: Lexer<'a>) -> Vec<Token> {
        let tokens = Vec::new();

        tokens
    }

    fn advance_token(mut self: Lexer<'a>) -> Token {
        while let Some(c) = self.advance_char() {
            match c {}
        }

        Token::new(TokenType::EOF)
    }

    fn advance_char(mut self: Lexer<'a>) -> Option<char> {
        self.cursor.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::{Operator, Symbol, Token};

    #[test]
    fn test_scan_operators() {
        let input = "+ =";
        let lexer = Lexer::new(input);
        let result = lexer.scan();
        assert_eq!(result[0].token_type, TokenType::Op(Operator::Plus));
        assert_eq!(result[1].token_type, TokenType::Op(Operator::Equal));
    }
}
