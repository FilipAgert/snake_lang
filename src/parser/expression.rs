use crate::lexer::token::*;
use std::iter::Peekable;
use std::slice::Iter;

// We should do binary operations first...
// Asignment,

#[derive(Debug, PartialEq)]
pub enum ValueExpression {
    Literal(Literal),
    Identifier(String),
    CallExpression(CallExpression),
}
#[derive(Debug, PartialEq)]
pub enum Expression {
    // an expression is something which returns a value. It is NOT a statement.
    // An assignment, for example, is a binary statement which assigns a variable to the result of an expression.
    ValueExpression(ValueExpression), // Literal or a variable.
    BinOp(BinaryExpression),          // A binary operation.
    UnOp(UnaryExpression),
}

#[derive(Debug, PartialEq)]
pub struct CallExpression {
    identifier: String,
    arguments: Vec<Expression>,
}
#[derive(Debug, PartialEq)]
pub struct BinaryExpression {
    op: Operator,
    left: Box<Expression>,
    right: Box<Expression>,
}
#[derive(Debug, PartialEq)]
pub struct UnaryExpression {
    op: Operator,
    operand: Box<Expression>,
}

#[derive(Debug)]
pub enum ExpressionError {
    IncompatibleTypes,
    InvalidExpressionToken,
    MissingOperand,
    OperandOnLhsError,
    UnimplementedError,
    UnexpectedKeyword,
    UnexpectedSymbol,
    UnexpectedEOF,
    MissingClosingBrace,
}
fn parse_argument_list(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Vec<Expression>, ExpressionError> {
    // first token is left brace, stops on the corresponding right brace.
    let mut exprs = Vec::<Expression>::new();
    // scan until we hit the same level of opening brace.
    // seperate arguments at highest level by commas.
    // call parse_expression on each comma seperated list
    tokens.next(); // consume the opening brace.
    while let Some(token) = tokens.peek() {
        if token.token_type == TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Right)))
        {
            tokens.next(); // Consume the closing brace.
            return Ok(exprs);
        } else if token.token_type == TokenType::Symbol(Symbol::Colon) {
            // Consume comma separated list
            tokens.next();
        } else {
            exprs.push(parse_expression(tokens, 0)?);
        }
    }
    Err(ExpressionError::UnexpectedEOF)
}

fn parse_primary(tokens: &mut Peekable<Iter<Token>>) -> Result<Expression, ExpressionError> {
    match &tokens
        .next()
        .ok_or(ExpressionError::MissingOperand)?
        .token_type
    {
        TokenType::Identifier(id) => {
            if let Some(token) = tokens.peek() {
                if token.token_type
                    == TokenType::Symbol(Symbol::Bracket(Bracket::Parenthesis(Side::Left)))
                {
                    Ok(Expression::ValueExpression(
                        ValueExpression::CallExpression(CallExpression {
                            identifier: id.clone(),
                            arguments: parse_argument_list(tokens)?,
                        }),
                    ))
                } else {
                    Ok(Expression::ValueExpression(ValueExpression::Identifier(
                        id.clone(),
                    )))
                }
            } else {
                Ok(Expression::ValueExpression(ValueExpression::Identifier(
                    id.clone(),
                )))
            }
        }
        TokenType::Literal(literal) => Ok(Expression::ValueExpression(ValueExpression::Literal(
            literal.clone(),
        ))),
        TokenType::Op(op) => match op {
            Operator::Minus => Err(ExpressionError::UnimplementedError),
            _ => Err(ExpressionError::OperandOnLhsError),
        },
        TokenType::Keyword(_) => Err(ExpressionError::UnexpectedKeyword),
        TokenType::Symbol(s) => match s {
            Symbol::Bracket(Bracket::Parenthesis(Side::Left)) => {
                let expr = parse_expression(tokens, 0)?;
                let next = tokens.next().ok_or(ExpressionError::UnexpectedEOF)?;
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

fn parse_expression(
    tokens: &mut Peekable<Iter<Token>>,
    min_precedence: i32,
) -> Result<Expression, ExpressionError> {
    let mut left = parse_primary(tokens)?;

    while let Some(token) = tokens.peek() {
        if let TokenType::Op(op) = &token.token_type {
            let precedence = op.precedence_value();
            if precedence < min_precedence {
                break;
            }

            tokens.next();
            let right = parse_expression(tokens, precedence + 1)?;

            left = Expression::BinOp(BinaryExpression {
                op: op.clone(),
                left: Box::new(left),
                right: Box::new(right),
            });
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
        let mut tokens = scan(input);
        let expression = parse_expression(&mut tokens.iter().peekable(), 0).unwrap();

        assert!(matches!(expression, Expression::BinOp(_)));
        if let Expression::BinOp(binop) = expression {
            assert!(matches!(*binop.left, Expression::BinOp(_)));
            assert!(matches!(
                *binop.right,
                Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(3)))
            ));
            assert_eq!(binop.op, Operator::Plus);

            if let Expression::BinOp(binop_inner) = *binop.left {
                assert_eq!(
                    *binop_inner.left,
                    Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(15)))
                );
                assert_eq!(
                    *binop_inner.right,
                    Expression::ValueExpression(ValueExpression::Identifier("x".to_string()))
                );
                assert_eq!(binop_inner.op, Operator::Times);
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
        let mut tokens = scan(input);
        let expression = parse_expression(&mut tokens.iter().peekable(), 0).unwrap();

        if let Expression::BinOp(root) = expression {
            assert_eq!(root.op, Operator::Times);
            assert_eq!(
                *root.left,
                Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(10)))
            );

            // Right side should be the result of the parenthesis: (5 + 3)
            if let Expression::BinOp(inner) = *root.right {
                assert_eq!(inner.op, Operator::Plus);
                assert_eq!(
                    *inner.left,
                    Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(5)))
                );
                assert_eq!(
                    *inner.right,
                    Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(3)))
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
        let mut tokens = scan(input);
        let expression = parse_expression(&mut tokens.iter().peekable(), 0).unwrap();

        if let Expression::BinOp(root) = expression {
            assert_eq!(root.op, Operator::Plus);
            // Right side is the final + 4
            assert_eq!(
                *root.right,
                Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(4)))
            );

            // Left side is (1 + (2 * 3))
            if let Expression::BinOp(middle) = *root.left {
                assert_eq!(middle.op, Operator::Plus);
                // Verify the multiplication is nested inside this right branch
                assert_eq!(
                    *middle.left,
                    Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(1)))
                );
                assert!(matches!(*middle.right, Expression::BinOp(_)));
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
        let mut tokens = scan(input);
        let expression = parse_expression(&mut tokens.iter().peekable(), 0).unwrap();

        if let Expression::BinOp(root) = expression {
            assert_eq!(root.op, Operator::Times);
            assert!(matches!(
                *root.left,
                Expression::ValueExpression(ValueExpression::CallExpression(_))
            ));
            if let Expression::ValueExpression(ValueExpression::CallExpression(call)) = *root.left {
                assert_eq!(call.identifier, "my_func");
                assert_eq!(
                    call.arguments[0],
                    Expression::ValueExpression(ValueExpression::Identifier("a".to_string()))
                );
                assert_eq!(
                    call.arguments[1],
                    Expression::ValueExpression(ValueExpression::Identifier("b".to_string()))
                );
                assert_eq!(call.arguments.len(), 2);
            }
            assert_eq!(
                *root.right,
                Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(2)))
            );
        } else {
            panic!("should be a binary expression")
        }
    }

    #[test]
    fn test_deeply_nested_parentheses() {
        let input = "(((10)))";
        let mut tokens = scan(input);
        let expression = parse_expression(&mut tokens.iter().peekable(), 0).unwrap();

        assert_eq!(
            expression,
            Expression::ValueExpression(ValueExpression::Literal(Literal::Integer(10)))
        );
    }

    #[test]
    fn test_operator_precedence_descending() {
        // 10 / 2 - 1 should be ((10 / 2) - 1)
        let input = "10/2-1";
        let mut tokens = scan(input);
        let expression = parse_expression(&mut tokens.iter().peekable(), 0).unwrap();

        if let Expression::BinOp(root) = expression {
            assert_eq!(root.op, Operator::Minus);
            if let Expression::BinOp(left_branch) = *root.left {
                assert_eq!(left_branch.op, Operator::Divide);
            } else {
                panic!("Division should be on the left branch");
            }
        } else {
            panic!("Should be a binary expression")
        }
    }
}
