use super::super::lexer::token::TokenType::EOF;
use super::super::lexer::token::*;
use std::collections::VecDeque;

pub struct ParseState {
    tokens: VecDeque<Token>,
    counter: NodeCounter,
}

impl ParseState {
    pub fn peek(&self) -> &Token {
        self.tokens.front().unwrap_or(&Token {
            token_type: TokenType::EOF,
            span: Span { start: 0, end: 1 },
        })
    }

    pub fn peek_at(&self, offset: usize) -> &Token {
        self.tokens.get(offset).unwrap_or(&Token {
            token_type: TokenType::EOF,
            span: Span { start: 0, end: 1 },
        })
    }

    pub fn next(&mut self) -> Token {
        let token = self.tokens.pop_front().unwrap_or(Token {
            token_type: TokenType::EOF,
            span: Span { start: 0, end: 1 },
        });
        token
    }

    pub fn next_id(&mut self) -> usize {
        return self.counter.next_id();
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens: tokens.into(),
            counter: NodeCounter::new(),
        }
    }

    // If we have an error, this iterates the tokens until we hit (but do not consume) a recovery token.
    // This could e.g. be a semicolon or closing brace.
    pub fn synchronize_to(&mut self, recovery_tokens: &[TokenType]) {
        while let token = self.peek()
            && token.token_type != TokenType::EOF
        {
            if recovery_tokens.contains(&token.token_type) {
                return;
            }
            self.next();
        }
    }
}

struct NodeCounter {
    next_id: usize,
}

impl NodeCounter {
    fn next_id(self: &mut Self) -> usize {
        let this = self.next_id;
        self.next_id += 1;
        this
    }

    fn new() -> Self {
        NodeCounter { next_id: 0 }
    }
}
