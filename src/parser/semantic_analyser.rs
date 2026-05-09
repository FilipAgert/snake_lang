use std::{collections::HashMap, hash::Hash};

use crate::{
    lexer::token::{DeclarationKeyword, Literal, Span},
    parser::{
        expression::{Expression, ExpressionT, ValueExpression},
        statement::{self, *},
    },
};

#[derive(Debug)]
enum SyntaxError {
    IncompatibleTypes(Span),
    UnexpectedRoot(Span),
    UseBeforeDefinition(Span),
    AlreadyDefinedInScope(Span),
}
#[derive(Clone, PartialEq)]
enum ReturnType {
    Standard(DeclarationKeyword),
    Custom(usize), // custom datatype: by string.
}

impl From<DeclarationKeyword> for ReturnType {
    fn from(value: DeclarationKeyword) -> Self {
        ReturnType::Standard(value)
    }
}

// need datastructures now. given a root expression, need to populate ALLnodes into tables.
// we have symbol table. it is transient.
struct Symbol {
    symbol_id: usize,
    depth: usize,
    identifier: String,
}

struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    fn define(self: &mut Self, id: String, node_id: usize) {
        let depth = self.depth();
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(
                id.clone(),
                Symbol {
                    symbol_id: node_id,
                    depth,
                    identifier: id,
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

    fn push_empty(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn new() -> Self {
        let mut s = Self { scopes: Vec::new() };
        s.push_empty();
        s
    }

    fn depth(&self) -> usize {
        self.scopes.len() - 1
    }
}

struct SymTables {
    link_table: Vec<usize>,
    type_table: Vec<Option<ReturnType>>,
    depth_table: Vec<Option<usize>>,
}

impl SymTables {
    fn new(num_ids: usize) -> Self {
        Self {
            link_table: vec![0; num_ids],
            type_table: vec![None; num_ids],
            depth_table: vec![Some(0); num_ids],
        }
    }
}

pub fn get_tables(root: &Statement, num_ids: usize) -> Result<SymTables, SyntaxError> {
    let mut tables = SymTables::new(num_ids);
    let mut symbol_table = SymbolTable::new();
    match &root.stype {
        StatementT::Root { statements } => {
            for statement in statements {
                populate_tables(statement, &mut tables, &mut symbol_table)?;
            }
        }
        _ => panic!("Should only call this method on the root"),
    }
    Ok(tables)
}

fn populate_tables(
    statement: &Statement,
    tables: &mut SymTables,
    symbol_table: &mut SymbolTable,
) -> Result<(), SyntaxError> {
    match &statement.stype {
        StatementT::Root { .. } => return Err(SyntaxError::UnexpectedRoot(statement.span)),
        StatementT::Assignment { identifier, value } => {
            populate_tables_expression(&value, tables, symbol_table)?; // expression should also be filled.
            if let Some(symbol) = symbol_table.lookup(&identifier) {
                tables.link_table[statement.node_id] = symbol.symbol_id;
            } else {
                return Err(SyntaxError::UseBeforeDefinition(statement.span));
            }
        }
        StatementT::Declaration {
            identifier,
            keyword,
            assignment,
        } => {
            if let Some(symbol) = symbol_table.lookup(identifier)
                && symbol.depth == symbol_table.depth()
            {
                return Err(SyntaxError::AlreadyDefinedInScope(statement.span));
            } else {
                symbol_table.define(identifier.clone(), statement.node_id);
                tables.link_table[statement.node_id] = statement.node_id;
                tables.type_table[statement.node_id] = Some((*keyword).into());
                tables.depth_table[statement.node_id] = Some(symbol_table.depth());
            }
            assignment
                .as_ref()
                .map(|a| populate_tables_expression(&a, tables, symbol_table));
        }
        StatementT::ExpressionStatement(expr) => populate_tables_expression(
            &Expression {
                etype: expr.clone(),
                span: statement.span,
                node_id: statement.node_id,
            },
            tables,
            symbol_table,
        )?,
        StatementT::Block { statements } => {
            tables.depth_table[statement.node_id] = Some(symbol_table.depth());
            tables.link_table[statement.node_id] = statement.node_id;

            symbol_table.push_empty();
            for statement in statements {
                populate_tables(statement, tables, symbol_table)?
            }
            symbol_table.pop();
        }
        StatementT::FunctionDeclaration {
            identifier,
            parameters,
            return_type,
            body,
        } => {
            if let Some(symbol) = symbol_table.lookup(identifier)
                && symbol.depth == symbol_table.depth()
            {
                return Err(SyntaxError::AlreadyDefinedInScope(statement.span));
            }
            tables.depth_table[statement.node_id] = Some(symbol_table.depth());
            tables.link_table[statement.node_id] = statement.node_id;
            tables.type_table[statement.node_id] = Some((*return_type).into());
            symbol_table.push_empty();
            for parameter in parameters {
                populate_tables(parameter, tables, symbol_table)?;
            }
            for statement in body {
                populate_tables(statement, tables, symbol_table)?;
            }
            symbol_table.pop();
        }
    }
    Ok(())
}

fn populate_tables_expression(
    expression: &Expression,
    tables: &mut SymTables,
    symbol_table: &mut SymbolTable,
) -> Result<(), SyntaxError> {
    tables.depth_table[expression.node_id] = Some(symbol_table.depth());
    match &expression.etype {
        ExpressionT::ValueExpression(val) => match val {
            ValueExpression::Literal(l) => {
                tables.link_table[expression.node_id] = expression.node_id;
                tables.type_table[expression.node_id] =
                    Some(ReturnType::Standard(l.clone().into()));
            }
            ValueExpression::Identifier(id) | ValueExpression::CallExpression { id, .. } => {
                if let Some(symbol) = symbol_table.lookup(&id) {
                    tables.link_table[expression.node_id] = symbol.symbol_id;
                } else {
                    return Err(SyntaxError::UseBeforeDefinition(expression.span));
                }
            }
        },
        ExpressionT::BinOp { left, op, right } => {
            populate_tables_expression(left.as_ref(), tables, symbol_table)?;
            populate_tables_expression(right.as_ref(), tables, symbol_table)?;
        }
        ExpressionT::UnOp { op, operand } => {
            populate_tables_expression(&operand, tables, symbol_table)?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fmt::Binary;
    use std::os::linux::raw::stat;

    use super::*;
    use crate::lexer::lexer::scan;
    use crate::parser::parse_state::ParseState;
    use crate::parser::statement::generate_ast;

    #[test]
    fn test_tables() {
        let input = "int a; a = 15;";
        let mut state = ParseState::new(scan(input));
        let (root, size) = generate_ast(&mut state).unwrap();

        let tables = get_tables(&root, size).expect("Generated syntax error");

        // Assume you know through inspection that:
        // Node 0 is 'int a' (Declaration)
        // Node 1 is 'a' in 'a = 15' (Usage)
        let (decl_id, usage_id) = if let StatementT::Root { statements } = root.stype {
            (statements[0].node_id, statements[1].node_id)
        } else {
            panic!("Waah!")
        };
        // 1. Test LinkTable: Usage must point to Declaration
        assert_eq!(
            tables.link_table[usage_id], decl_id,
            "Usage of 'a' should link to its declaration"
        );

        assert!(matches!(
            tables.type_table[decl_id],
            Some(ReturnType::Standard(DeclarationKeyword::Int))
        ));

        // 2. Test DepthTable: Both should be at the same scope depth
        assert_eq!(
            tables.depth_table[decl_id],
            Some(0),
            "Declaration 'a' should be at depth 0"
        );
        assert_eq!(
            tables.depth_table[usage_id],
            Some(0),
            "Usage of 'a' should inherit depth 0"
        );

        // 3. Test TypeTable: Declaration should have the 'int' type
        // Use your ReturnType enum variants here
        assert!(tables.type_table[decl_id].is_some());
    }
}
