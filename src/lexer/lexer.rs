use crate::lexer::token::{Keyword, Operator, Symbol, Token, TokenType};

pub fn scan(s: &str) -> Vec<Token> {
    let tokens = Vec::new();

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::{Operator, Symbol, Token};

    #[test]
    fn test_scan_operators() {
        let input = "+ =";
        let result = scan(input);
        assert_eq!(result[0].token_type, TokenType::Op(Operator::Plus));
        assert_eq!(result[1].token_type, TokenType::Op(Operator::Equal));
    }
}
