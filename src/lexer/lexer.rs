use crate::diagnostics::span::Span;
use crate::lexer::token::{Token, TokenType};
use std::mem;
use std::string::String;

#[derive(Debug)]
struct TokenStr {
    string: String,
    span: Span,
}
impl TokenStr {
    fn new(string: String, span: Span) -> Self {
        TokenStr { string, span }
    }
}

const SPECIAL_SYMBOLS: &'static str = "()[]{};:,";
const OPERATORS: &'static str = "=+-/*&|<>!^";

pub fn scan(str: &str) -> Vec<Token> {
    let token_str = seperate_string(str);
    let mut tokens = Vec::new();
    for t_str in token_str {
        let t_type = TokenType::from_str(&t_str.string);
        let token = Token {
            token_type: t_type,
            span: t_str.span,
        };
        tokens.push(token);
    }
    tokens
}

fn seperate_string(str: &str) -> Vec<TokenStr> {
    let mut cursor = str.char_indices().peekable();
    let mut tokens: Vec<TokenStr> = Vec::new();
    let mut curr_str: String = String::new();
    let mut curr_start = 0;

    let mut tokenize = |s: String, start: usize, end: usize| {
        if !s.is_empty() {
            let span = Span { start, end };
            let token = TokenStr::new(s, span);
            tokens.push(token);
        }
    };

    while let Some((idx, c)) = cursor.next() {
        if c == '\n' {
            continue;
        }

        if c == ' ' {
            tokenize(std::mem::take(&mut curr_str), curr_start, idx);
            curr_start = idx + 1;
        } else if SPECIAL_SYMBOLS.contains(c) {
            tokenize(std::mem::take(&mut curr_str), curr_start, idx);
            tokenize(c.to_string(), idx, idx + 1);
            curr_start = idx + 1;
        } else if OPERATORS.contains(c) {
            tokenize(std::mem::take(&mut curr_str), curr_start, idx);
            curr_str.push(c);
            let mut end = idx + 1;
            while let Some((idx, c)) = cursor.peek().copied()
                && OPERATORS.contains(c)
            {
                cursor.next();
                curr_str.push(c);
                end = idx + 1;
            }
            tokenize(std::mem::take(&mut curr_str), curr_start, end);
            curr_start = idx + 1;
        } else {
            curr_str.push(c);
        }
    }
    tokenize(curr_str, curr_start, str.len());

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::{Bracket, BuiltInType, Keyword, Literal, Operator, Side, Symbol};

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
        assert_eq!(
            result[0].token_type,
            TokenType::Keyword(Keyword::Declaration(BuiltInType::Int))
        );
        assert_eq!(
            result[1].token_type,
            TokenType::Identifier(Box::from("val"))
        );
        assert_eq!(result[2].token_type, TokenType::Op(Operator::Equal));
        assert_eq!(
            result[3].token_type,
            TokenType::Literal(Literal::Integer(4))
        );
        assert_eq!(result[4].token_type, TokenType::Symbol(Symbol::Semicolon));
        assert_eq!(
            result[5].token_type,
            TokenType::Keyword(Keyword::Declaration(BuiltInType::Int))
        );
        assert_eq!(result[6].token_type, TokenType::Identifier(Box::from("x")));
        assert_eq!(result[7].token_type, TokenType::Op(Operator::Equal));
        assert_eq!(result[8].token_type, TokenType::Identifier(Box::from("f")));
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
