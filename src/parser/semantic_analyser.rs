use std::collections::HashMap;

use crate::{
    lexer::token::{DeclarationKeyword, Literal, Span},
    parser::{
        expression::{Expression, ExpressionT, ValueExpression},
        statement::*,
    },
};

enum SyntaxError {
    IncompatibleTypes(Span),
}
enum ReturnType {
    Literal(Literal),
    Custom(String), // custom datatype: by string.
}

impl From<Literal> for ReturnType {
    fn from(value: Literal) -> Self {
        ReturnType::Literal(value)
    }
}

// need datastructures now. given a root expression, need to populate ALLnodes into tables.
// we have symbol table. it is transient.
struct Symbol {
    symbol_id: usize,
    depth: usize,
    identifier: String,
    symbol_type: SymbolType,
}
enum SymbolType {
    Variable,
    Function,
    Type, // int, bool, CustomStruct
}
struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    fn define(self: &mut Self, id: String, node_id: usize, symbol_type: SymbolType) {
        let depth = self.scopes.len() - 1;
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(
                id.clone(),
                Symbol {
                    symbol_id: node_id,
                    depth,
                    identifier: id,
                    symbol_type: symbol_type,
                },
            );
        }
    }

    fn lookup(&self, id: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(id) {
                return Some(symbol);
            }
        }
        None
    }
}

pub fn analyze_value_expression(value_exp: &ValueExpression) -> Result<ReturnType, SyntaxError> {
    match value_exp {
        ValueExpression::Literal(literal) => Ok(literal.clone().into()),
        _ => todo!(),
    }
}
pub fn analyze_expression(expression: &Expression) -> Result<ReturnType, SyntaxError> {
    match &expression.etype {
        ExpressionT::BinOp { left, op, right } => todo!(),
        ExpressionT::UnOp { op, operand } => todo!(),
        ExpressionT::ValueExpression(val_exp) => match val_exp {
            ValueExpression::Literal(literal) => todo!(),
            _ => todo!(),
        },
    }
}
pub fn analyze_statement(statement: &Statement) -> Result<(), SyntaxError> {
    match &statement.stype {
        StatementT::Root { statements } | StatementT::Block { statements } => (),

        _ => todo!(),
    }

    Ok(())
}
