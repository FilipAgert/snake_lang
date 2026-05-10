use super::super::lexer::token::*;
use crate::parser::diagnostic::*;
use std::collections::VecDeque;

pub struct ParseState<'a> {
    tokens: VecDeque<Token>,
    counter: Counter,
    diag: &'a mut Diagnostic,
    anon_fun: Counter,
    anon_var: Counter,
}
static EOF_TOKEN: Token = Token {
    token_type: TokenType::EOF,
    span: Span::min_info(),
};
impl<'a> ParseState<'a> {
    pub fn peek(&self) -> &Token {
        self.tokens.front().unwrap_or(&EOF_TOKEN)
    }

    pub fn peek_at(&self, offset: usize) -> &Token {
        self.tokens.get(offset).unwrap_or(&EOF_TOKEN)
    }

    pub fn next(&mut self) -> Token {
        let token = self.tokens.pop_front().unwrap_or(EOF_TOKEN.clone());
        token
    }

    pub fn next_id(&mut self) -> usize {
        return self.counter.next_id();
    }

    pub fn next_anon_fun(&mut self) -> Box<str> {
        let next_ctr = self.anon_fun.next_id();
        let str = format!("fun_{}", next_ctr);
        str.into_boxed_str()
    }
    pub fn next_anon_var(&mut self) -> Box<str> {
        let next_ctr = self.anon_var.next_id();
        let str = format!("var_{}", next_ctr);
        str.into_boxed_str()
    }

    pub fn new(tokens: Vec<Token>, diag: &'a mut Diagnostic) -> Self {
        Self {
            tokens: tokens.into(),
            counter: Counter::new(),
            diag: diag,
            anon_fun: Counter::new(),
            anon_var: Counter::new(),
        }
    }

    pub fn report<T>(&mut self, span: Span, error_t: T)
    where
        T: Into<ErrorT>,
    {
        self.diag.push(span, error_t);
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
pub struct Counter {
    next_id: usize,
}

impl Counter {
    pub fn next_id(self: &mut Self) -> usize {
        let this = self.next_id;
        self.next_id += 1;
        this
    }

    pub fn num_allocated(&self) -> usize {
        self.next_id
    }

    pub fn new() -> Self {
        Counter { next_id: 0 }
    }
}
