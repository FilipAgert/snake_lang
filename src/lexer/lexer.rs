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
    let mut strings: Vec<TokenStr> = Vec::new();
    let mut curr_str: String = String::new();
    let mut curr_start = 0;

    while let Some((idx, c)) = cursor.next() {
        if c == '\n' {
            curr_start = idx + 1;
            continue;
        }

        if (c == ' ' || c == '\n') && !curr_str.is_empty() {
            let span = Span {
                start: curr_start,
                end: idx,
            };
            let str_token = TokenStr::new(mem::take(&mut curr_str), span);
            strings.push(str_token);
            curr_start = idx + 1;
        } else if SPECIAL_SYMBOLS.contains(c) {
            if !curr_str.is_empty() {
                let span = Span {
                    start: curr_start,
                    end: idx,
                };
                let str_token = TokenStr::new(mem::take(&mut curr_str), span);
                strings.push(str_token);
            }
            let spec_char_span = Span {
                start: idx,
                end: idx + 1,
            };
            strings.push(TokenStr::new(c.to_string(), spec_char_span));
            curr_start = idx + 1;
        } else if OPERATORS.contains(c) {
            if !curr_str.is_empty() {
                let span = Span {
                    start: curr_start,
                    end: idx,
                };
                let str_token = TokenStr::new(mem::take(&mut curr_str), span);
                strings.push(str_token);
            }
            curr_str.push(c);
            let mut end = idx + 1;
            while let Some((idx, c)) = cursor.peek().copied()
                && OPERATORS.contains(c)
            {
                cursor.next();
                curr_str.push(c);
                end = idx + 1;
            }
            let spec_char_span = Span {
                start: idx,
                end: end,
            };
            strings.push(TokenStr::new(mem::take(&mut curr_str), spec_char_span));
            curr_start = idx + 1;
        } else if c != ' ' {
            curr_str.push(c);
        } else if c == ' ' {
            curr_start = idx + 1;
        }
    }
    if !curr_str.is_empty() {
        let span = Span {
            start: curr_start,
            end: str.len(),
        };
        strings.push(TokenStr::new(curr_str, span));
    }

    strings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::{Bracket, BuiltInType, Keyword, Literal, Operator, Side, Symbol};

    #[test]
    fn test_sep_strings() {
        let input = "fn f(num: int);";
        let result = seperate_string(input);
        assert_eq!(result[0].string, "fn");
        assert_eq!(result[0].span.start, 0);
        assert_eq!(result[0].span.end, 2);
        assert_eq!(result[1].string, "f");
        assert_eq!(result[1].span.start, 3);
        assert_eq!(result[1].span.end, 4);
        assert_eq!(result[2].string, "(");
        assert_eq!(result[2].span.start, 4);
        assert_eq!(result[2].span.end, 5);
        assert_eq!(result[3].string, "num");
        assert_eq!(result[3].span.start, 5);
        assert_eq!(result[3].span.end, 8);
        assert_eq!(result[4].string, ":");
        assert_eq!(result[4].span.start, 8);
        assert_eq!(result[4].span.end, 9);
        assert_eq!(result[5].string, "int");
        assert_eq!(result[5].span.start, 10);
        assert_eq!(result[5].span.end, 13);
        assert_eq!(result[6].string, ")");
        assert_eq!(result[6].span.start, 13);
        assert_eq!(result[6].span.end, 14);
        assert_eq!(result[7].string, ";");
        assert_eq!(result[7].span.start, 14);
        assert_eq!(result[7].span.end, 15);
        for res in result {
            println!("{:?}", res)
        }
    }

    #[test]
    fn test_sep_strings_with_indentation() {
        let input = "int val = 4;\n    int x = f(4);";
        let result = seperate_string(input);
        assert_eq!(result[5].string, "int");
        assert_eq!(result[5].span.start, 17);
        assert_eq!(result[5].span.end, 20);
        for res in result {
            println!("{:?}", res)
        }
    }

    #[test]
    fn test_sep_strings_without_indentation() {
        let input = "int val = 4;\nint x = f(4);";
        let result = seperate_string(input);
        assert_eq!(result[5].string, "int");
        assert_eq!(result[5].span.start, 13);
        assert_eq!(result[5].span.end, 16);
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
