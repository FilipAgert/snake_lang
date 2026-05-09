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
