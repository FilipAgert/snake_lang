use crate::lexer::token::{Keyword, Operator, Symbol, Token};

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
        assert_eq!(result[0], Token::Op(Operator::Plus));
        assert_eq!(result[1], Token::Op(Operator::Equal));
    }
}
