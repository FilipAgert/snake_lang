use crate::lexer::token::*;
use crate::parser::expression::*;
use std::iter::Peekable;
use std::slice::Iter;
#[derive(Debug, Clone)]

pub enum StatementT {
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
    ExpressionStatement(ExpressionT),
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub stype: StatementT,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    identifier: String,
    keyword: DeclarationKeyword,
    assignment: Option<Expression>,
}

#[derive(Debug, Clone)]
enum StatementError {
    ExpressionError(ExpressionError),
    UnexpectedEOF,
    UnexpectedToken,
    DeclarationError(DeclarationError),
    ExpectedClosingBrace,
    ExpectedSemiColon,
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
        Self {
            stype: StatementT::ExpressionStatement(expr.etype),
            span: expr.span,
        }
    }
}
#[derive(Debug, Clone)]
enum DeclarationError {
    MissingDeclarationKeyword,
    MissingIdentifier,
}

fn parse_declaration(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<(Declaration, Span), StatementError> {
    let decl_token = tokens
        .next()
        .ok_or(DeclarationError::MissingDeclarationKeyword)?;
    let decl = match decl_token.token_type {
        TokenType::Keyword(Keyword::Declaration(decl)) => Ok(decl),
        _ => Err(DeclarationError::MissingDeclarationKeyword),
    }?;
    let id_token = tokens.next().ok_or(DeclarationError::MissingIdentifier)?;
    let id = match &id_token.token_type {
        TokenType::Identifier(id) => Ok(id.clone()),
        _ => Err(DeclarationError::MissingIdentifier),
    }?;

    let assignment_token = tokens.peek().ok_or(StatementError::UnexpectedEOF)?;
    let mut assignment_span = None;
    let assignment: Option<Expression> = match assignment_token.token_type {
        TokenType::Symbol(Symbol::Semicolon | Symbol::Colon) => None, //
        TokenType::Op(Operator::Equal) => {
            tokens.next(); // Consume equal
            let assignment_expr = parse_expression(tokens, 0)?;
            assignment_span = Some(assignment_expr.span);
            Some(assignment_expr)
        }
        _ => {
            return Err(StatementError::UnexpectedToken);
        }
    };
    let span = Span::merge(&decl_token.span, &assignment_span.unwrap_or(id_token.span));
    Ok((
        Declaration {
            identifier: id,
            keyword: decl,
            assignment: assignment,
        },
        span,
    ))
}
fn parse_fn_declaration(tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, StatementError> {
    // expect the identifier to be the first token.
    // then we expect an argument list made of several declarations
    // then we expect a return type by -> type
    // then a function body
    let function_keyword = tokens.next().ok_or(StatementError::UnexpectedEOF)?;

    todo!();
}
pub fn generate_ast(tokens: &mut Peekable<Iter<Token>>) -> Result<Statement, StatementError> {
    let (root_statements, span) = generate_ast_block(tokens)?;
    if !matches!(tokens.next(), None) {
        return Err(StatementError::UnexpectedToken);
    }

    Ok(Statement {
        stype: StatementT::Root {
            statements: root_statements,
        },
        span: span.unwrap_or(Span { start: 0, end: 1 }),
    })
}
fn generate_ast_block(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<(Vec<Statement>, Option<Span>), StatementError> {
    let mut block_statements = Vec::<Statement>::new();

    while let Some(token) = tokens.peek() {
        match token.token_type {
            TokenType::Keyword(Keyword::Declaration(_)) => {
                let (decl, span) = parse_declaration(tokens)?;
                block_statements.push(Statement {
                    stype: StatementT::Declaration(decl),
                    span: span,
                });
                //expect and consume semicolon.
                if !matches!(
                    tokens
                        .next()
                        .ok_or(StatementError::UnexpectedEOF)?
                        .token_type,
                    TokenType::Symbol(Symbol::Semicolon),
                ) {
                    return Err(StatementError::ExpectedSemiColon);
                }
            }
            TokenType::Keyword(Keyword::FunctionDeclaration) => {
                //  do not consume function declaration keyword
                block_statements.push(parse_fn_declaration(tokens)?);
            }
            TokenType::Identifier(_) => {
                // This must be an expression. If it is a binop expression with operator =, turn it into an assignment.
                let expr = parse_expression(tokens, -1)?;
                if let ExpressionT::BinOp { left, op, right } = expr.clone().etype
                    && op == Operator::Equal
                {
                    if let ExpressionT::ValueExpression(ValueExpression::Identifier(id)) =
                        left.etype
                    {
                        block_statements.push(Statement {
                            stype: StatementT::Assignment {
                                identifier: id,
                                value: *right.clone(),
                            },
                            span: Span::merge(&left.span, &right.span),
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
                let brace = tokens.next().expect("Already checked for existance"); // consume token and descend into block.
                let (block, _) = generate_ast_block(tokens)?;
                // expect closing brace. consumes it.
                let closing_brace = tokens.next().ok_or(StatementError::ExpectedClosingBrace)?;
                if !matches!(
                    closing_brace.token_type,
                    TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right)))
                ) {
                    return Err(StatementError::ExpectedClosingBrace);
                }
                let span = Span::merge(&brace.span, &closing_brace.span);
                block_statements.push(Statement {
                    stype: StatementT::Block { statements: block },
                    span: span,
                });
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
    let span =
        if let (Some(first), Some(last)) = (block_statements.first(), block_statements.last()) {
            Some(Span::merge(&first.span, &last.span))
        } else {
            None
        };

    Ok((block_statements, span))
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
        assert!(matches!(ast.stype, StatementT::Root { statements: _ }));
        if let StatementT::Root { statements } = ast.stype.clone() {
            assert!(statements.len() == 2);
            let decl = &statements[0];
            assert!(matches!(decl.stype, StatementT::Declaration(_)));
            if let StatementT::Declaration(d) = &decl.stype {
                assert_eq!(d.identifier, "a");
                assert_eq!(d.keyword, DeclarationKeyword::Int);
            }
            let ass = &statements[1].stype;
            assert!(matches!(ass, StatementT::Assignment { .. }));
            if let StatementT::Assignment { identifier, value } = &decl.stype {
                assert_eq!(identifier, "a");
                assert!(matches!(&value.etype, ExpressionT::BinOp { .. }));
            }
        }
        println!("{:?}", ast);
    }
}
