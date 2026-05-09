use std::{collections::HashMap, hash::Hash};

use crate::{
    lexer::token::{DeclarationKeyword, Literal, Span},
    parser::{
        expression::{Expression, ExpressionT, ValueExpression},
        statement::{self, *},
    },
};

#[derive(Debug)]
pub enum SyntaxError {
    IncompatibleTypes,
    UnexpectedRoot,
    UseBeforeDefinition,
    AlreadyDefinedInScope,
}
#[derive(Clone, PartialEq, Debug)]
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

#[derive(Debug)]
struct SymTables {
    link_table: Vec<usize>,
    type_table: Vec<Option<ReturnType>>,
    depth_table: Vec<Option<usize>>,
}

impl SymTables {
    fn new(num_ids: usize) -> Self {
        Self {
            link_table: vec![usize::MAX; num_ids],
            type_table: vec![None; num_ids],
            depth_table: vec![Some(0); num_ids],
        }
    }
}

pub fn get_tables(root: &Statement, num_ids: usize) -> Result<SymTables, SyntaxError> {
    let mut tables = SymTables::new(num_ids);
    let mut symbol_table = SymbolTable::new();
    tables.depth_table[root.node_id] = Some(symbol_table.depth());
    tables.link_table[root.node_id] = root.node_id;
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

        let tables: SymTables = get_tables(&root, size).expect("Generated syntax error");

        println!("{:?}", root);
        println!("{:?}", tables);

        for l in &tables.link_table {
            assert_ne!(*l, usize::MAX);
        }

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

    #[test]
    fn test_scope_shadowing() {
        let input = "int a; { int a; a = 5; } a = 10;";
        let mut state = ParseState::new(scan(input));
        let (root, size) = generate_ast(&mut state).unwrap();
        let tables = get_tables(&root, size).expect("Failed to generate tables");

        if let StatementT::Root { statements } = root.stype {
            let global_decl_id = statements[0].node_id;
            let block_node = &statements[1];
            let global_usage_id = statements[2].node_id;

            // Extracting IDs from the block { int a; a = 5; }
            if let StatementT::Block {
                statements: block_stmts,
            } = &block_node.stype
            {
                let local_decl_id = block_stmts[0].node_id;
                let local_usage_id = block_stmts[1].node_id;

                // 1. Local usage must link to local declaration
                assert_eq!(tables.link_table[local_usage_id], local_decl_id);
                assert_eq!(tables.depth_table[local_decl_id], Some(1));

                // 2. Global usage must link to global declaration
                assert_eq!(tables.link_table[global_usage_id], global_decl_id);
                assert_eq!(tables.depth_table[global_decl_id], Some(0));

                // 3. Ensure they link to different nodes
                assert_ne!(
                    tables.link_table[local_usage_id],
                    tables.link_table[global_usage_id]
                );
            }
        }
    }
    #[test]
    fn test_undeclared_variable() {
        let input = "a = 5;int a = 4;";
        let mut state = ParseState::new(scan(input));
        let (root, size) = generate_ast(&mut state).unwrap();

        // Depending on your implementation, this should return an Err
        // or the link_table entry should remain usize::MAX
        let result = get_tables(&root, size);

        match result {
            Err(err) => assert!(matches!(
                err,
                SyntaxError::UseBeforeDefinition(Span { start: 0, end: 5 })
            )), // Correctly caught as a syntax/semantic error
            _ => assert!(false),
        }
    }
}
