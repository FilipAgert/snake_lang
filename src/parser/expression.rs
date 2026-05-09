use super::parse_state::*;
use crate::lexer::token::Span;
use crate::lexer::token::*;
use std::fmt::Binary;
use std::iter::Peekable;
use std::slice::Iter;

// We should do binary operations first...
// Asignment,

#[derive(Debug, PartialEq, Clone)]
pub enum ValueExpression {
    Literal(Literal),
    Identifier(String),
    CallExpression {
        id: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub etype: ExpressionT,
    pub span: Span,
    pub node_id: usize,
}
#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionT {
    // an expression is something which returns a value. It is NOT a statement.
    // An assignment, for example, is a binary statement which assigns a variable to the result of an expression.
    ValueExpression(ValueExpression), // Literal or a variable.
    BinOp {
        left: Box<Expression>,
        op: Operator,
        right: Box<Expression>,
    }, // A binary operation.
    UnOp {
        op: Operator,
        operand: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum ExpressionError {
    MissingOperand,
    OperandOnLhsError,
    UnexpectedKeyword,
    UnexpectedSymbol,
    UnexpectedEOF,
    MissingClosingBrace,
}
fn parse_argument_list(state: &mut ParseState) -> Result<Vec<Expression>, ExpressionError> {
    // first token is left brace, stops on the corresponding right brace.
    let mut exprs = Vec::<Expression>::new();
    // scan until we hit the same level of opening brace.
    // seperate arguments at highest level by commas.
    // call parse_expression on each comma seperated list
    state.next(); // consume the opening brace.
    loop {
        let token = state.peek();
        if token.token_type == TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Right)))
        {
            state.next(); // Consume the closing brace.
            return Ok(exprs);
        } else if token.token_type == TokenType::Symbol(Symbol::Comma) {
            // Consume comma separated list
            state.next();
        } else if token.token_type == TokenType::EOF {
            break;
        } else {
            exprs.push(parse_expression(state, 0)?);
        }
    }
    Err(ExpressionError::UnexpectedEOF)
}

fn parse_primary(state: &mut ParseState) -> Result<Expression, ExpressionError> {
    let next_token = state.next();
    match &next_token.token_type {
        TokenType::Identifier(id) => {
            // function call or variable
            let token = state.peek();
            if token.token_type
                == TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Left)))
                && token.token_type != TokenType::EOF
            {
                let arguments = parse_argument_list(state)?;
                let last_span = arguments.last().map(|e| e.span).unwrap_or(next_token.span);
                let fn_span = Span::merge(
                    &next_token.span,
                    &Span {
                        start: last_span.start,
                        end: last_span.end + 1,
                    },
                );
                // + 1 to consume the brace.
                Ok(Expression {
                    etype: ExpressionT::ValueExpression(ValueExpression::CallExpression {
                        id: id.clone(),
                        arguments: arguments,
                    }),
                    span: fn_span,
                    node_id: state.next_id(),
                })
            } else {
                // variable
                Ok(Expression {
                    etype: ExpressionT::ValueExpression(ValueExpression::Identifier(id.clone())),
                    span: next_token.span,
                    node_id: state.next_id(),
                })
            }
        }
        TokenType::Literal(literal) => Ok(Expression {
            span: next_token.span,
            etype: ExpressionT::ValueExpression(ValueExpression::Literal(literal.clone())),
            node_id: state.next_id(),
        }),
        TokenType::Op(op) => match op {
            Operator::Minus => todo!(),
            _ => Err(ExpressionError::OperandOnLhsError),
        },
        TokenType::Keyword(_) => Err(ExpressionError::UnexpectedKeyword),
        TokenType::Symbol(s) => match s {
            Symbol::Bracket(Bracket::Parenthesis(Side::Left)) => {
                let expr = parse_expression(state, 0)?;
                let next = state.next();
                if next.token_type
                    == TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Right)))
                {
                    Ok(expr)
                } else {
                    Err(ExpressionError::MissingClosingBrace)
                }
            }
            _ => Err(ExpressionError::UnexpectedSymbol),
        },
        TokenType::EOF => Err(ExpressionError::UnexpectedEOF),
    }
}

pub fn parse_expression(
    state: &mut ParseState,
    min_precedence: i32,
) -> Result<Expression, ExpressionError> {
    let mut left = parse_primary(state)?;

    loop {
        let token_type = state.peek().token_type.clone();
        if let TokenType::Op(op) = token_type {
            let precedence = op.precedence_value();
            if precedence < min_precedence {
                break;
            }

            state.next();
            let right = parse_expression(state, precedence + 1)?;

            left = Expression {
                span: Span::merge(&left.span, &right.span),
                etype: ExpressionT::BinOp {
                    op: op.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                },
                node_id: state.next_id(),
            };
        } else {
            break;
        }
    }
    Ok(left)
}

#[cfg(test)]
mod tests {
    use std::fmt::Binary;

    use super::*;
    use crate::lexer::lexer::scan;
    use crate::lexer::token::*;
    use crate::parser::expression::*;

    #[test]
    fn test_build_expression_1() {
        let input = "15*x+3";
        let mut state = ParseState::new(scan(input));
        let expression = parse_expression(&mut state, 0).unwrap();

        assert!(matches!(expression.etype, ExpressionT::BinOp { .. }));
        if let ExpressionT::BinOp { left, op, right } = expression.etype {
            assert!(matches!(left.etype, ExpressionT::BinOp { .. }));
            assert!(matches!(
                right.etype,
                ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(3)))
            ));
            assert_eq!(op, Operator::Plus);

            if let ExpressionT::BinOp {
                left: left_inner,
                op: op_inner,
                right: right_inner,
            } = left.etype
            {
                assert_eq!(
                    left_inner.etype,
                    ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(15)))
                );
                assert_eq!(
                    right_inner.etype,
                    ExpressionT::ValueExpression(ValueExpression::Identifier("x".to_string()))
                );
                assert_eq!(op_inner, Operator::Times);
            } else {
                panic!("Expected binary expression here");
            }
        } else {
            panic!("Expected binary expression here.")
        }
    }
    #[test]
    fn test_parenthesis_precedence() {
        // (5 + 3) should be evaluated first, making it a child of '*'
        let input = "10*(5+3)";
        let mut state = ParseState::new(scan(input));
        let expression = parse_expression(&mut state, 0).unwrap();

        if let ExpressionT::BinOp { left, op, right } = expression.etype {
            assert_eq!(op, Operator::Times);
            assert_eq!(
                left.etype,
                ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(10)))
            );

            // Right side should be the result of the parenthesis: (5 + 3)
            if let ExpressionT::BinOp { left, op, right } = right.etype {
                assert_eq!(op, Operator::Plus);
                assert_eq!(
                    left.etype,
                    ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(5)))
                );
                assert_eq!(
                    right.etype,
                    ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(3)))
                );
            } else {
                panic!("Expected nested BinOp from parenthesis");
            }
        } else {
            panic!("Expected root multiplication");
        }
    }

    #[test]
    fn test_long_expression_chain() {
        // 1 + 2 * 3 + 4 should result in ((1 + (2 * 3)) + 4)
        let input = "1+2*3+4";
        let mut state = ParseState::new(scan(input));
        let expression = parse_expression(&mut state, 0).unwrap();

        if let ExpressionT::BinOp { left, op, right } = expression.etype {
            assert_eq!(op, Operator::Plus);
            // Right side is the final + 4
            assert_eq!(
                right.etype,
                ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(4)))
            );

            // Left side is (1 + (2 * 3))
            if let ExpressionT::BinOp { left, op, right } = left.etype {
                assert_eq!(op, Operator::Plus);
                // Verify the multiplication is nested inside this right branch
                assert_eq!(
                    left.etype,
                    ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(1)))
                );
                assert!(matches!(right.etype, ExpressionT::BinOp { .. }));
            } else {
                panic!("Should be a binop")
            }
        } else {
            panic!("Should be a binop");
        }
    }

    #[test]
    fn test_function_call_in_expression() {
        // Testing: my_func(a, b) * 2
        // Note: parse_argument_list must be implemented for this to pass
        let input = "my_func(a, b) * 2";
        let mut state = ParseState::new(scan(input));
        let expression = parse_expression(&mut state, 0).unwrap();

        if let ExpressionT::BinOp { left, op, right } = expression.etype {
            assert_eq!(op, Operator::Times);
            assert!(matches!(
                left.etype,
                ExpressionT::ValueExpression(ValueExpression::CallExpression { .. })
            ));
            if let ExpressionT::ValueExpression(ValueExpression::CallExpression { id, arguments }) =
                left.etype
            {
                assert_eq!(id, "my_func");
                assert_eq!(
                    arguments[0].etype,
                    ExpressionT::ValueExpression(ValueExpression::Identifier("a".to_string()))
                );
                assert_eq!(
                    arguments[1].etype,
                    ExpressionT::ValueExpression(ValueExpression::Identifier("b".to_string()))
                );
                assert_eq!(arguments.len(), 2);
            }
            assert_eq!(
                right.etype,
                ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(2)))
            );
        } else {
            panic!("should be a binary expression")
        }
    }

    #[test]
    fn test_deeply_nested_parentheses() {
        let input = "(((10)))";
        let mut state = ParseState::new(scan(input));
        let expression = parse_expression(&mut state, 0).unwrap();

        assert_eq!(
            expression.etype,
            ExpressionT::ValueExpression(ValueExpression::Literal(Literal::Integer(10)))
        );
    }

    #[test]
    fn test_operator_precedence_descending() {
        // 10 / 2 - 1 should be ((10 / 2) - 1)
        let input = "10/2-1";
        let mut state = ParseState::new(scan(input));
        let expression = parse_expression(&mut state, 0).unwrap();

        if let ExpressionT::BinOp { left, op, right } = expression.etype {
            assert_eq!(op, Operator::Minus);
            if let ExpressionT::BinOp { left, op, right } = left.etype {
                assert_eq!(op, Operator::Divide);
            } else {
                panic!("Division should be on the left branch");
            }
        } else {
            panic!("Should be a binary expression")
        }
    }
}
