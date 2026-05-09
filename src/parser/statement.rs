use crate::lexer::token::*;
use crate::parser::expression::*;
use crate::parser::parse_state::ParseState;
use std::iter::Peekable;
use std::slice::Iter;
#[derive(Debug, Clone)]

pub enum StatementT {
    Root {
        statements: Vec<Statement>,
    },
    Declaration {
        identifier: String,
        keyword: DeclarationKeyword,
        assignment: Option<Expression>,
    },
    FunctionDeclaration {
        identifier: String,
        parameters: Vec<Statement>,
        return_type: DeclarationKeyword,
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
    pub node_id: usize,
}

#[derive(Debug, Clone)]
pub enum StatementError {
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
            node_id: expr.node_id,
        }
    }
}
#[derive(Debug, Clone)]
enum DeclarationError {
    MissingDeclarationKeyword,
    MissingIdentifier,
}

fn parse_declaration(state: &mut ParseState) -> Result<Statement, StatementError> {
    let peeked_token = state.peek();
    let (declaration_token, declaration_value) = match peeked_token.token_type {
        TokenType::Keyword(Keyword::Declaration(decl)) => (Some(state.next()), Some(decl)),
        _ => {
            state.report::<StatementError>(
                peeked_token.span,
                DeclarationError::MissingDeclarationKeyword.into(),
            );
            (None, None)
        }
    };
    let declaration_span = declaration_token.map(|d| d.span);

    let peeked_token = state.peek();
    let (id, id_span) = match peeked_token.token_type.clone() {
        TokenType::Identifier(id) => {
            let token = state.next();
            (id, token.span)
        }
        _ => {
            state.report::<StatementError>(
                peeked_token.span,
                DeclarationError::MissingIdentifier.into(),
            );
            return Err(DeclarationError::MissingIdentifier.into());
        }
    };

    let assignment_token = state.peek();
    let assignment: Option<Expression> = match assignment_token.token_type {
        TokenType::Symbol(Symbol::Semicolon | Symbol::Comma) => None, //
        TokenType::Op(Operator::Equal) => {
            state.next(); // Consume equal
            let assignment_expr = parse_expression(state, 0);

            Some(assignment_expr)
        }
        _ => {
            return Err(StatementError::UnexpectedToken);
        }
    };
    let span = Span::merge(
        &declaration_span.unwrap_or(id_span),
        &assignment.clone().map(|a| a.span).unwrap_or(id_span),
    );

    let statement = Statement {
        node_id: state.next_id(),
        stype: StatementT::Declaration {
            identifier: id,
            keyword: declaration_value.unwrap_or(DeclarationKeyword::Error),
            assignment: assignment,
        },
        span: span,
    };
    Ok(statement)
}
fn parse_fn_declaration(state: &mut ParseState) -> Result<Statement, StatementError> {
    // expect the identifier to be the first token.
    // then we expect an argument list made of several declarations
    // then we expect a return type by ': type'
    // then a function body surrounded by braces.
    let function_keyword = state.next();

    todo!();
}
pub fn generate_ast(state: &mut ParseState) -> Result<(Statement, usize), StatementError> {
    let root_id = state.next_id();
    let (root_statements, span) = generate_ast_block(state)?;
    if !matches!(state.next().token_type, TokenType::EOF) {
        return Err(StatementError::UnexpectedToken);
    }
    let size = state.next_id();

    Ok((
        Statement {
            stype: StatementT::Root {
                statements: root_statements,
            },
            span: span.unwrap_or(Span { start: 0, end: 1 }),
            node_id: root_id,
        },
        size,
    ))
}
fn generate_ast_block(
    state: &mut ParseState,
) -> Result<(Vec<Statement>, Option<Span>), StatementError> {
    let mut block_statements = Vec::<Statement>::new();
    loop {
        let token = state.peek();
        match token.token_type {
            TokenType::Keyword(Keyword::Declaration(_)) => {
                let decl = parse_declaration(state);
                if let Ok(decl) = decl {
                    block_statements.push(decl);
                } else {
                    state.synchronize_to(&[TokenType::Symbol(Symbol::Semicolon)]);
                }
                //expect and consume semicolon.
                let next_token = state.peek();
                if !matches!(next_token.token_type, TokenType::Symbol(Symbol::Semicolon),) {
                    state.report(next_token.span.clone(), StatementError::ExpectedSemiColon);
                    // if not a semicolon, report it, but continue as usual.
                } else {
                    state.next(); // consume semicolon if its there. 
                }
            }
            TokenType::Keyword(Keyword::FunctionDeclaration) => {
                //  do not consume function declaration keyword
                block_statements.push(parse_fn_declaration(state)?);
            }
            TokenType::Identifier(_) => {
                // This must be an expression. If it is a binop expression with operator =, turn it into an assignment.
                if let TokenType::Op(op) = &state.peek_at(1).token_type
                    && *op == Operator::Equal
                {
                    let token_span = token.span;
                    let id = match state.next().token_type {
                        // consume id
                        TokenType::Identifier(id) => id,
                        _ => unreachable!("Already checked for id!"),
                    };
                    state.next(); // consume equal sign
                    let assignment = parse_expression(state, 0);
                    block_statements.push(Statement {
                        span: Span::merge(&token_span, &assignment.span),
                        stype: StatementT::Assignment {
                            identifier: id,
                            value: assignment,
                        },
                        node_id: state.next_id(),
                    });
                } else {
                    block_statements.push(parse_expression(state, 0).into());
                }
            }
            TokenType::Op(Operator::Minus) | TokenType::Literal(_) => {
                block_statements.push(parse_expression(state, 0).into());
            }
            TokenType::Symbol(Symbol::Semicolon) => {
                state.next();
            }
            TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Left))) => {
                let brace = state.next(); // consume token and descend into block.
                let (block, _) = generate_ast_block(state)?;
                // expect closing brace. consumes it.
                let closing_brace = state.peek();
                if !matches!(
                    closing_brace.token_type,
                    TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right)))
                ) {
                    return Err(StatementError::ExpectedClosingBrace);
                }
                let closing_brace = state.next();
                let span = Span::merge(&brace.span, &closing_brace.span);
                block_statements.push(Statement {
                    stype: StatementT::Block { statements: block },
                    span: span,
                    node_id: state.next_id(),
                });
            }
            TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right))) => break,
            // do not consume closing brace so that block calling can check for its existance
            TokenType::EOF => {
                state.next(); // consume the token.
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
    use crate::parser::diagnostic::Diagnostic;
    use crate::parser::expression::*;

    #[test]
    fn test_build_expression_1() {
        let input = "int a; a = 15*x+3;";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (ast, _) = generate_ast(&mut state).unwrap();
        println!("{:?}", diag.get_errors());
        assert_eq!(diag.get_errors().len(), 0);
        assert!(matches!(ast.stype, StatementT::Root { statements: _ }));
        if let StatementT::Root { statements } = ast.stype.clone() {
            assert!(statements.len() == 2);
            let decl = &statements[0];
            assert!(matches!(decl.stype, StatementT::Declaration { .. }));
            if let StatementT::Declaration {
                identifier,
                keyword,
                ..
            } = &decl.stype
            {
                assert_eq!(identifier, "a");
                assert_eq!(*keyword, DeclarationKeyword::Int);
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
