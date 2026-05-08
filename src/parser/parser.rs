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
    MissingClosingBrace,
}
fn parse_argument_list(
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Vec<Expression>, ExpressionError> {
    // first token is left brace, stops on the corresponding right brace.
    let exprs = Vec::<Expression>::new();
    Err(ExpressionError::UnimplementedError)
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
                let next = tokens.next().ok_or(ExpressionError::MissingOperand)?;
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
        TokenType::EOF => Err(ExpressionError::MissingOperand),
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
            let right = parse_expression(tokens, precedence)?;

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
    use crate::parser::parser::*;

    #[test]
    fn test_sep_strings() {
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
                    *binop_inner.left,
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
}
