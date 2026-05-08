use crate::lexer::token::*;
use crate::parser::expression::*;
use std::iter::Peekable;
use std::slice::Iter;

enum Statement {
    Root {
        statements: Vec<Statement>,
    },
    Declaration(Declaration),
    FunctionDeclaration {
        identifier: String,
        arguments: Vec<Declaration>,
        body: Vec<Statement>,
    },
    Assignment {
        identifier: String,
        value: Expression,
    },
    Block {
        statements: Vec<Statement>,
    },
    ExpressionStatement(Expression),
}

struct Declaration {
    identifier: String,
    keyword: DeclarationKeyword,
}

enum StatementError {
    ExpressionError(ExpressionError),
    UnimplementedError,
    UnexpectedEOF,
    DeclarationError(DeclarationError),
}

enum DeclarationError {
    MissingDeclarationKeyword,
    MissingIdentifier,
    MissingSemicolon,
    UnexpectedToken,
}

fn parse_declaration(tokens: &mut Peekable<Iter<Token>>) -> Result<Declaration, DeclarationError> {
    let decl = match tokens
        .next()
        .ok_or(DeclarationError::MissingDeclarationKeyword)?
        .token_type
    {
        TokenType::Keyword(Keyword::Declaration(decl)) => Ok(decl),
        _ => Err(DeclarationError::MissingDeclarationKeyword),
    }?;
    let id = match &tokens
        .next()
        .ok_or(DeclarationError::MissingIdentifier)?
        .token_type
    {
        TokenType::Identifier(id) => Ok(id.clone()),
        _ => Err(DeclarationError::MissingIdentifier),
    }?;

    match tokens
        .next()
        .ok_or(DeclarationError::MissingSemicolon)?
        .token_type
    {
        TokenType::Symbol(Symbol::Semicolon) => Ok(()),
        _ => Err(DeclarationError::MissingSemicolon),
    }?;

    Ok(Declaration {
        identifier: id,
        keyword: decl,
    })
}
fn parse_fn_declaration(tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, StatementError> {
    Err(StatementError::UnimplementedError)
}

pub fn generate_ast(tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, StatementError> {
    let mut root_statements = Vec::<Statement>::new();

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Keyword(Keyword::Declaration(_)) => {
                let decl = parse_declaration(tokens)?;
                root_statements.push(Statement::Declaration(decl));
            }
            TokenType::Keyword(Keyword::FunctionDeclaration) => {}

            TokenType::Symbol(Symbol::Semicolon) => {
                tokens.next().expect("Should be unreachable");
            }
        }
    }

    Ok(Statement::Root {
        statements: root_statements,
    })
}
