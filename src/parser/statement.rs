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
    UnexpectedToken,
    DeclarationError(DeclarationError),
    AssignmentToNonId,
}

impl From<DeclarationError> for StatementError {
    fn from(error: DeclarationError) -> Self {
        StatementError::DeclarationError(error)
    }
}

impl From<ExpressionError> for StatementError {
    fn from(error: ExpressionError) -> Self {
        StatementError::ExpressionError(error)
    }
}

impl From<Expression> for Statement {
    fn from(expr: Expression) -> Self {
        Statement::ExpressionStatement(expr)
    }
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
    todo!();
}

pub fn generate_ast(tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, StatementError> {
    let mut root_statements = Vec::<Statement>::new();

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Keyword(Keyword::Declaration(_)) => {
                let decl = parse_declaration(tokens)?;
                root_statements.push(Statement::Declaration(decl));
            }
            TokenType::Keyword(Keyword::FunctionDeclaration) => {
                todo!();
            }
            TokenType::Identifier(_) => {
                // This must be an expression. If it is a binop expression with operator =, turn it into an assignment.
                let expr = parse_expression(tokens, -1)?;
                if let Expression::BinOp(binop) = expr.clone()
                    && binop.op == Operator::Equal
                {
                    if let Expression::ValueExpression(ValueExpression::Identifier(id)) =
                        *binop.left
                    {
                        root_statements.push(Statement::Assignment {
                            identifier: id,
                            value: *binop.right.clone(),
                        });
                    } else {
                        return Err(StatementError::AssignmentToNonId);
                    }
                } else {
                    root_statements.push(expr.into());
                }
            }
            TokenType::Op(Operator::Minus) | TokenType::Literal(_) => {
                root_statements.push(parse_expression(tokens, 0)?.into());
            }
            TokenType::Symbol(Symbol::Semicolon) => {
                tokens.next().expect("Should be unreachable");
            }
            TokenType::EOF => break,
            _ => {
                return Err(StatementError::UnexpectedToken);
            }
        }
    }

    Ok(Statement::Root {
        statements: root_statements,
    })
}
