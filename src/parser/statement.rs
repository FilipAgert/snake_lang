use crate::lexer::token::*;
use crate::parser::expression::*;
use crate::parser::parse_state::ParseState;
use crate::parser::semantic_analyser::SemanticError;
use std::iter::Peekable;
use std::slice::Iter;
use std::str::Matches;
#[derive(Debug, Clone)]

pub enum StatementT {
    Root {
        statements: Vec<Statement>,
    },
    Declaration {
        identifier: Box<str>,
        keyword: DeclarationKeyword,
        assignment: Option<Expression>,
    },
    FunctionDeclaration {
        identifier: Box<str>,
        parameters: Vec<Statement>,
        return_type: DeclarationKeyword,
        body: Vec<Statement>,
    },
    Assignment {
        identifier: Box<str>,
        value: Expression,
    },
    Block {
        statements: Vec<Statement>,
    },
    ReturnStatement(ExpressionT),
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
    UnexpectedToken(TokenType),
    ExpectedToken { expected: TokenType, got: TokenType },
    AssignmentToNonId,
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

fn parse_declaration(state: &mut ParseState) -> Result<Statement, StatementError> {
    let peeked_token = state.peek();
    let (declaration_token, declaration_value) = match peeked_token.token_type {
        TokenType::Keyword(Keyword::Declaration(decl)) => (Some(state.next()), Some(decl)),
        _ => {
            state.report::<StatementError>(
                peeked_token.span,
                StatementError::ExpectedToken {
                    expected: TokenType::Keyword(Keyword::Declaration(DeclarationKeyword::Error)),
                    got: peeked_token.token_type.clone(),
                },
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
            let span = peeked_token.span.clone();
            state.report::<StatementError>(
                span,
                StatementError::ExpectedToken {
                    expected: TokenType::Identifier(Box::from("")),
                    got: peeked_token.token_type.clone(),
                },
            );
            (state.next_anon_var(), span)
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
            return Err(StatementError::UnexpectedToken(
                assignment_token.token_type.clone(),
            ));
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
    if !matches!(
        function_keyword.token_type,
        TokenType::Keyword(Keyword::FunctionDeclaration),
    ) {
        let err = StatementError::ExpectedToken {
            expected: TokenType::Keyword(Keyword::FunctionDeclaration),
            got: function_keyword.token_type,
        };
        state.report(function_keyword.span, err.clone());
        return Err(err);
    }

    let identifier = state.peek();
    let id = if let TokenType::Identifier(id) = identifier.token_type.clone() {
        state.next();
        id
    } else {
        state.report(
            identifier.span,
            StatementError::ExpectedToken {
                expected: TokenType::Identifier(Box::from("Function id")),
                got: identifier.token_type.clone(),
            },
        );
        state.next_anon_fun() // give it an anonymous name.
    };

    // expect parameter list (int a, int b, bool c = true)
    let opening_brace = state.peek();
    let l_par = TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Left)));
    if !matches!(&opening_brace.token_type, l_par) {
        state.report(
            opening_brace.span,
            StatementError::ExpectedToken {
                expected: l_par,
                got: opening_brace.token_type.clone(),
            },
        );
    } else {
        state.next(); // consume opening brace.
    }
    let mut parameters: Vec<Statement> = Vec::new();
    const SYNC_TOKENS: [TokenType; 4] = [
        TokenType::Symbol(Symbol::Comma),
        TokenType::Symbol(Symbol::Colon),
        TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Right))),
        TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Left))),
    ];

    loop {
        let next = state.peek();
        match next.token_type {
            TokenType::Symbol(Symbol::Comma) => {
                state.next();
            }
            TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Right))) => {
                state.next();
                break;
            }
            TokenType::Keyword(Keyword::Declaration(..)) | TokenType::Identifier(..) => {
                if let Ok(decl) = parse_declaration(state) {
                    parameters.push(decl);
                } else {
                    state.synchronize_to(&SYNC_TOKENS);
                }
                //id here is an error, but may still parse it as a declaration without a real type.
            }
            TokenType::Symbol(Symbol::Colon)
            | TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right))) => {
                state.report(
                    next.span,
                    StatementError::UnexpectedToken(next.token_type.clone()),
                );
                break;
            }
            _ => {
                state.report(
                    next.span,
                    StatementError::UnexpectedToken(next.token_type.clone()),
                );
                //try to recover: next comma, next ), next : or next {
                state.synchronize_to(&SYNC_TOKENS);
            }
        }
    }

    // now expect a : ReturnType keyword:
    let colon = state.peek();
    if !matches!(colon.token_type, TokenType::Symbol(Symbol::Colon)) {
        state.report(
            colon.span,
            StatementError::ExpectedToken {
                expected: TokenType::Symbol(Symbol::Colon),
                got: colon.token_type.clone(),
            },
        );
    } else {
        state.next();
    }
    // expect a ReturnType keyword
    let ret_type = state.peek();
    let ret_type = if let TokenType::Keyword(Keyword::Declaration(decl)) = ret_type.token_type {
        state.next();
        decl
    } else {
        state.report(
            ret_type.span,
            StatementError::ExpectedToken {
                expected: TokenType::Keyword(Keyword::Declaration(DeclarationKeyword::Void)),
                got: ret_type.token_type.clone(),
            },
        );
        DeclarationKeyword::Error
    };

    // now we expect a block statement.
    let opening_brace = state.peek();
    if !matches!(
        opening_brace.token_type,
        TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Left)))
    ) {
        state.report(
            opening_brace.span,
            StatementError::ExpectedToken {
                expected: TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Left))),
                got: opening_brace.token_type.clone(),
            },
        );
        state.synchronize_to(&[TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(
            Side::Left,
        )))]);
    }

    let brace = state.next(); // consume opening brace.
    let block = generate_ast_block(state);
    let (statements, span) = if let Ok((mut statements, mut span)) = block {
        let closing_brace = state.peek();

        let span = if !matches!(
            closing_brace.token_type,
            TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right)))
        ) {
            let span = closing_brace.span.clone();
            state.report(
                span.clone(),
                StatementError::ExpectedToken {
                    expected: TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right))),
                    got: closing_brace.token_type.clone(),
                },
            );
            span
        } else {
            let b = state.next();
            b.span
        };
        (statements, Some(Span::merge(&brace.span, &span)))
    } else {
        state.report(
            Span::merge(&brace.span, &state.peek().span),
            block.expect_err("In error branch"),
        );
        state.synchronize_to(&[
            TokenType::Symbol(Symbol::Semicolon),
            TokenType::Keyword(Keyword::FunctionDeclaration),
            TokenType::Keyword(Keyword::Declaration(DeclarationKeyword::Bool)),
            TokenType::Keyword(Keyword::Declaration(DeclarationKeyword::Int)),
            TokenType::Keyword(Keyword::Declaration(DeclarationKeyword::Void)),
        ]);
        (Vec::new(), None)
    };

    // fun_span
    let fun_span = Span::merge(&function_keyword.span, &span.unwrap_or(brace.span));
    Ok(Statement {
        span: fun_span,
        node_id: state.next_id(),
        stype: StatementT::FunctionDeclaration {
            identifier: id,
            parameters: parameters,
            return_type: ret_type,
            body: statements,
        },
    })
}
pub fn generate_ast(state: &mut ParseState) -> Result<(Statement, usize), StatementError> {
    let root_id = state.next_id();
    let (root_statements, span) = generate_ast_block(state)?;
    let token = state.next();
    if !matches!(token.token_type, TokenType::EOF) {
        return Err(StatementError::ExpectedToken {
            expected: TokenType::EOF,
            got: token.token_type,
        });
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
                    state.report(
                        next_token.span.clone(),
                        StatementError::ExpectedToken {
                            expected: TokenType::Symbol(Symbol::Semicolon),
                            got: next_token.token_type.clone(),
                        },
                    );
                    // if not a semicolon, report it, but continue as usual.
                } else {
                    state.next(); // consume semicolon if its there. 
                }
            }
            TokenType::Keyword(Keyword::FunctionDeclaration) => {
                //  do not consume function declaration keyword
                block_statements.push(parse_fn_declaration(state)?);
            }
            TokenType::Keyword(Keyword::Return) => {
                let ret = state.next();
                let ret_expr = parse_expression(state, 0);
                block_statements.push(Statement {
                    stype: StatementT::ReturnStatement(ret_expr.etype),
                    span: Span::merge(&ret.span, &ret_expr.span),
                    node_id: state.next_id(),
                });
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
                    return Err(StatementError::ExpectedToken {
                        expected: TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(
                            Side::Right,
                        ))),
                        got: closing_brace.token_type.clone(),
                    });
                }
                let closing_brace = state.next();
                let span = Span::merge(&brace.span, &closing_brace.span);
                block_statements.push(Statement {
                    stype: StatementT::Block { statements: block },
                    span: span,
                    node_id: state.next_id(),
                });
            }
            TokenType::Symbol(Symbol::Bracket(Bracket::CurlyBrace(Side::Right)))
            | TokenType::Keyword(Keyword::Return) => break,
            // do not consume closing brace so that block calling can check for its existance
            TokenType::EOF => {
                state.next(); // consume the token.
                break;
            }
            _ => {
                state.report(
                    token.span,
                    StatementError::UnexpectedToken(token.token_type.clone()),
                );
                state.synchronize_to(&[TokenType::Symbol(Symbol::Semicolon)]);
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
                assert_eq!(*identifier, "a".into());
                assert_eq!(*keyword, DeclarationKeyword::Int);
            }
            let ass = &statements[1].stype;
            assert!(matches!(ass, StatementT::Assignment { .. }));
            if let StatementT::Assignment { identifier, value } = &decl.stype {
                assert_eq!(*identifier, "a".into());
                assert!(matches!(&value.etype, ExpressionT::BinOp { .. }));
            }
        }
        println!("{:?}", ast);
    }

    #[test]
    fn test_build_fn() {
        let input = "fn f(int a, int b, bool c): {
        int x = 2;
        int a = 4;
        c = b + y;
        return c
     }";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (ast, _) = generate_ast(&mut state).unwrap();
        assert_eq!(diag.get_errors().len(), 2);
    }
}
