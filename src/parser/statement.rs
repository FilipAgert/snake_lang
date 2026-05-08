use crate::lexer::token::*;
use crate::parser::expression::*;
use std::iter::Peekable;
use std::slice::Iter;
#[derive(Debug, Clone)]

pub enum Statement {
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

#[derive(Debug, Clone)]
pub struct Declaration {
    identifier: String,
    keyword: DeclarationKeyword,
}

#[derive(Debug, Clone)]
enum StatementError {
    ExpressionError(ExpressionError),
    UnimplementedError,
    UnexpectedEOF,
    UnexpectedToken,
    DeclarationError(DeclarationError),
    ExpectedClosingBrace,
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
#[derive(Debug, Clone)]
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
    // expect the identifier to be the first token.
    // then we expect an argument list made of several declarations
    // then we expect a return type by -> type
    // then a function body
    todo!();
}
pub fn generate_ast(tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, StatementError> {
    let root_statements = generate_ast_block(tokens)?;
    if !matches!(tokens.next(), None) {
        return Err(StatementError::UnexpectedToken);
    }
    Ok(Statement::Root {
        statements: root_statements,
    })
}
fn generate_ast_block(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Vec<Statement>, StatementError> {
    let mut block_statements = Vec::<Statement>::new();

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Keyword(Keyword::Declaration(_)) => {
                let decl = parse_declaration(tokens)?;
                block_statements.push(Statement::Declaration(decl));
            }
            TokenType::Keyword(Keyword::FunctionDeclaration) => {
                // consume function declaration keyword
                tokens.next();
                block_statements.push(parse_fn_declaration(tokens)?);
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
                        block_statements.push(Statement::Assignment {
                            identifier: id,
                            value: *binop.right.clone(),
                        });
                    } else {
                        return Err(StatementError::AssignmentToNonId);
                    }
                } else {
                    block_statements.push(expr.into());
                }
            }
            TokenType::Op(Operator::Minus) | TokenType::Literal(_) => {
                block_statements.push(parse_expression(tokens, 0)?.into());
            }
            TokenType::Symbol(Symbol::Semicolon) => {
                tokens.next();
            }
            TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Left))) => {
                tokens.next(); // consume token and descend into block.
                block_statements.push(Statement::Block {
                    statements: generate_ast_block(tokens)?,
                });
                // expect closing brace. consumes it.
                if !matches!(tokens.next(), Some(token) if token.token_type == TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right))))
                {
                    return Err(StatementError::ExpectedClosingBrace);
                }
            }
            TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right))) => break,
            // do not consume closing brace so that block calling can check for its existance
            TokenType::EOF => {
                tokens.next(); // consume the token.
                break;
            }
            _ => {
                return Err(StatementError::UnexpectedToken);
            }
        }
    }

    Ok(block_statements)
}

#[cfg(test)]
mod tests {
    use std::fmt::Binary;
    use std::os::linux::raw::stat;

    use super::*;
    use crate::lexer::lexer::scan;
    use crate::lexer::token::*;
    use crate::parser::expression::*;

    #[test]
    fn test_build_expression_1() {
        let input = "int a; a = 15*x+3;";
        let mut tokens = scan(input);
        let ast = generate_ast(&mut tokens.iter().peekable()).unwrap();
        assert!(matches!(ast, Statement::Root { statements: _ }));
        if let Statement::Root { statements } = ast.clone() {
            assert!(statements.len() == 2);
            let decl = &statements[0];
            assert!(matches!(decl, Statement::Declaration(_)));
            if let Statement::Declaration(d) = decl {
                assert_eq!(d.identifier, "a");
                assert_eq!(d.keyword, DeclarationKeyword::Int);
            }
            let ass = &statements[1];
            assert!(matches!(ass, Statement::Assignment { identifier, value }));
            if let Statement::Assignment { identifier, value } = decl {
                assert_eq!(identifier, "a");
                assert!(matches!(value, Expression::BinOp(_)));
            }
        }
        println!("{:?}", ast);
    }
}
