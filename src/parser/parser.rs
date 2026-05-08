use crate::lexer::token::*;

// We should do binary operations first...
// Asignment,

pub enum ValueExpression {
    Literal(Literal),
    Identifier(String),
    CallExpression(CallExpression),
}
pub enum Expression {
    // an expression is something which returns a value. It is NOT a statement.
    // An assignment, for example, is a binary statement which assigns a variable to the result of an expression.
    ValueExpression(ValueExpression), // Literal or a variable.
    BinOp(BinaryExpression),          // A binary operation.
    UnOp(UnaryExpression),
}

pub struct CallExpression {
    identifier: String,
    arguments: Vec<Expression>,
}
pub struct BinaryExpression {
    op: Operator,
    left: Box<Expression>,
    right: Box<Expression>,
}
pub struct UnaryExpression {
    op: Operator,
    operand: Box<Expression>,
}

pub enum AssignmentError {
    AssignmentToLiteral,
    IncompatibleTypes,
}
pub enum ExpressionError {
    AssignmentError(AssignmentError),
}

fn parse_expression(tokens: &[Token]) -> Result<Expression, ExpressionError> {
    // assume that tokens is parsed to be a statement between two semicolons?
    // e.g. assignment operation or something.
}
