use crate::{lexer::token::{Literal, Span}, parser::{expression::{Expression, ExpressionT, ValueExpression},statement::*}};

enum SyntaxError {
    IncompatibleTypes(Span),
}
enum ReturnType{
    Literal(Literal),
    Custom(String), // custom datatype: by string.
}

impl From<Literal> for ReturnType{
    fn from(value: Literal) -> Self {
        ReturnType::Literal(value)
    }
}

pub fn analyze_value_expression(value_exp:& ValueExpression) -> Result<ReturnType, SyntaxError>{
    match value_exp {
        ValueExpression::Literal(literal) => Ok(literal.clone().into()),

    }
}
pub fn analyze_expression(expression: &Expression) -> Result<ReturnType, SyntaxError>{
    match expression.etype{
        ExpressionT::BinOp { left, op, right } => todo!(),
        ExpressionT::UnOp { op, operand } => todo!(),
        ExpressionT::ValueExpression(val_exp) => match val_exp {
            ValueExpression::Literal(literal) => 
        }
    }
}
pub fn analyze_statement(statement: &Statement) -> Result<(), SyntaxError> {
    match &statement.stype {
        StatementT::Root { statements } | StatementT::Block { statements } => (),
        StatementT::
        _ => todo!(),
    }

    Ok(())
}
