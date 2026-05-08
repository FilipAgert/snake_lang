use crate::lexer::token::{Token, TokenType};
use std::iter::Peekable;
use std::mem;
use std::string::String;

#[derive(Debug)]
struct TokenStr {
    string: String,
    line: u32,
    col: u32,
}
impl TokenStr {
    fn new(string: String, line: u32, col: u32) -> Self {
        TokenStr { string, line, col }
    }
}

const SPECIAL_SYMBOLS: &'static str = "()[]{};=+-/*";

pub fn scan(str: &str) -> Vec<Token> {
    let token_str = seperate_string(str);
    let mut tokens = Vec::new();
    for t_str in token_str {
        let t_type = TokenType::from_str(&t_str.string);
        let token = Token {
            token_type: t_type,
            line: t_str.line,
            col: t_str.col,
            len: t_str
                .string
                .len()
                .try_into()
                .expect("Should work to convert usize into u32"),
        };
        tokens.push(token);
    }
    tokens
}

fn seperate_string(str: &str) -> Vec<TokenStr> {
    let mut cursor = str.chars().peekable();
    let mut strings: Vec<TokenStr> = Vec::new();
    let mut curr_str: String = String::new();
    let mut line_ctr = 0;
    let mut col_ctr = 0;
    let mut start_col = 0;

    while let Some(c) = cursor.next() {
        col_ctr += 1;
        if c == '\n' {
            line_ctr += 1;
            col_ctr = 0;
            continue;
        }

        if (c == ' ' || c == '\n') && !curr_str.is_empty() {
            let str_token = TokenStr::new(mem::take(&mut curr_str), line_ctr, start_col);
            strings.push(str_token);
            start_col = col_ctr;
        } else if SPECIAL_SYMBOLS.contains(c) {
            if !curr_str.is_empty() {
                let str_token = TokenStr::new(mem::take(&mut curr_str), line_ctr, start_col);
                strings.push(str_token);
            }
            strings.push(TokenStr::new(c.to_string(), line_ctr, col_ctr));
            start_col = col_ctr;
        } else if c != ' ' {
            curr_str.push(c);
        }
    }
    if !curr_str.is_empty() {
        strings.push(TokenStr::new(curr_str, line_ctr, start_col));
    }

    strings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::{Bracket, Keyword, Literal, Operator, Side, Symbol};

    #[test]
    fn test_sep_strings() {
        let input = "int val = 4;\nint x = f(4);";
        let result = seperate_string(input);
        assert_eq!(result[0].string, "int");
        assert_eq!(result[1].string, "val");
        assert_eq!(result[2].string, "=");
        assert_eq!(result[3].string, "4");
        assert_eq!(result[4].string, ";");
        assert_eq!(result[5].string, "int");
        assert_eq!(result[6].string, "x");
        assert_eq!(result[7].string, "=");
        assert_eq!(result[8].string, "f");
        assert_eq!(result[9].string, "(");
        assert_eq!(result[10].string, "4");
        assert_eq!(result[11].string, ")");
        assert_eq!(result[12].string, ";");
        for res in result {
            println!("{:?}", res)
        }
    }

    #[test]
    fn test_scan() {
        let input = "int val = 4;\nint x = f(4);";
        let result = scan(input);
        assert_eq!(result[0].token_type, TokenType::Keyword(Keyword::Int));
        assert_eq!(
            result[1].token_type,
            TokenType::Identifier("val".to_string())
        );
        assert_eq!(result[2].token_type, TokenType::Op(Operator::Equal));
        assert_eq!(
            result[3].token_type,
            TokenType::Literal(Literal::Integer(4))
        );
        assert_eq!(result[4].token_type, TokenType::Symbol(Symbol::Semicolon));
        assert_eq!(result[5].token_type, TokenType::Keyword(Keyword::Int));
        assert_eq!(result[6].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(result[7].token_type, TokenType::Op(Operator::Equal));
        assert_eq!(result[8].token_type, TokenType::Identifier("f".to_string()));
        assert_eq!(
            result[9].token_type,
            TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Left)))
        );
        assert_eq!(
            result[10].token_type,
            TokenType::Literal(Literal::Integer(4))
        );
        assert_eq!(
            result[11].token_type,
            TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Right)))
        );
        assert_eq!(result[12].token_type, TokenType::Symbol(Symbol::Semicolon));
        for res in result {
            println!("{:?}", res)
        }
    }
}
