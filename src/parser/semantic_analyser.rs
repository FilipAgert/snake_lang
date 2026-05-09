use core::num;
use std::any::Any;
use std::{collections::HashMap, hash::Hash};

use crate::parser::diagnostic::Diagnostic;
use crate::parser::expression;
use crate::parser::parse_state::Counter;
use crate::{
    lexer::token::{DeclarationKeyword, Literal, Span},
    parser::{
        expression::{Expression, ExpressionT, ValueExpression},
        statement::{self, *},
    },
};
#[derive(Debug)]
pub enum SemanticError {
    IncompatibleTypes { left: ReturnType, right: ReturnType },
    UnexpectedRoot,
    UseBeforeDefinition,
    AlreadyDefinedInScope,
}
#[derive(Clone, PartialEq, Debug)]
enum ReturnType {
    Standard(DeclarationKeyword),
    Custom(usize), // custom datatype: by string.
    Error,         // Compiler could not determine type.
}

impl From<DeclarationKeyword> for ReturnType {
    fn from(value: DeclarationKeyword) -> Self {
        match value {
            DeclarationKeyword::Error => ReturnType::Error,
            _ => ReturnType::Standard(value),
        }
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
    pub const GLOBAL_SCOPE_DEPTH: usize = 1;
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
pub struct DecTables {
    link_table: Vec<usize>,
    type_table: Vec<ReturnType>,
}

pub fn type_check_pass(node: &Statement, diag: &mut Diagnostic, dec_tables: &DecTables) {
    match &node.stype {
        StatementT::Root { statements } | StatementT::Block { statements } => {
            for statement in statements {
                type_check_pass(&statement, diag, dec_tables);
            }
        }
        StatementT::Assignment { value, .. }
        | StatementT::Declaration {
            assignment: Some(value),
            ..
        } => {
            let lhs_type = &dec_tables.type_table[dec_tables.link_table[node.node_id]];
            let rhs_type = &type_check_pass_expr(value, diag, dec_tables);
            if lhs_type != rhs_type
                && *lhs_type != ReturnType::Error
                && *rhs_type != ReturnType::Error
            {
                // check for error so we do not spawn unecessarily many errors.
                diag.push(
                    node.span,
                    SemanticError::IncompatibleTypes {
                        left: lhs_type.clone(),
                        right: rhs_type.clone(),
                    },
                );
            }
        }
        StatementT::Declaration { .. } => {} // already checked in above branch.
        StatementT::ExpressionStatement(expr) => {
            type_check_pass_expr(
                &Expression {
                    etype: expr.clone(),
                    span: node.span,
                    node_id: node.node_id,
                },
                diag,
                dec_tables,
            );
        }
        StatementT::FunctionDeclaration { .. } => {
            // need to check arguments match
            // need to check all statements in body
            // need to check return type matches function signature.
            todo!(); // still need to check for return type in body here...
        }
    }
}

fn type_check_pass_expr(
    expr: &Expression,
    diag: &mut Diagnostic,
    dec_tables: &DecTables,
) -> ReturnType {
    match &expr.etype {
        ExpressionT::Error => ReturnType::Standard(DeclarationKeyword::Error),
        ExpressionT::ValueExpression(val) => match val {
            ValueExpression::CallExpression { arguments, .. } => {
                for arg in arguments {
                    type_check_pass_expr(arg, diag, dec_tables);
                }
                dec_tables.type_table[dec_tables.link_table[expr.node_id]].clone() // return type of function.
            }
            ValueExpression::Literal(l) => {
                let decl: DeclarationKeyword = l.clone().into();
                decl.into()
            }
            ValueExpression::Identifier(_) => {
                dec_tables.type_table[dec_tables.link_table[expr.node_id]].clone()
            }
        },
        ExpressionT::BinOp { left, op: _, right } => {
            let left_type = type_check_pass_expr(&left, diag, dec_tables);
            let right_type = type_check_pass_expr(&right, diag, dec_tables);

            if left_type != right_type {
                if left_type != ReturnType::Error && right_type != ReturnType::Error {
                    diag.push(
                        expr.span,
                        SemanticError::IncompatibleTypes {
                            left: left_type,
                            right: right_type,
                        },
                    );
                }
                ReturnType::Error
            } else {
                left_type
            }
        }
        ExpressionT::UnOp { operand, .. } => type_check_pass_expr(&operand, diag, dec_tables),
    }
}

pub fn get_dec_tables(root: &Statement, diag: &mut Diagnostic, num_ids: usize) -> DecTables {
    let mut symbol_ctr = Counter::new();
    let mut symbol_table = SymbolTable::new();
    let mut dec_tables = DecTables {
        link_table: vec![usize::MAX; num_ids],
        type_table: Vec::new(),
    };
    populate_link_table(
        root,
        &mut dec_tables,
        &mut symbol_ctr,
        &mut symbol_table,
        diag,
    );
    assert_eq!(symbol_ctr.num_allocated(), dec_tables.type_table.len());
    dec_tables
}

fn define_symbol(
    id: String,
    node_id: usize,
    symbol_ctr: &mut Counter,
    link_table: &mut Vec<usize>,
    symbol_table: &mut SymbolTable,
) {
    let symbol_id = symbol_ctr.next_id();
    symbol_table.define(id, symbol_id);
    link_table[node_id] = symbol_id;
}

// first pass of compiler through AST.
// Defines all links and types of all declarations.
fn populate_link_table(
    statement: &Statement,
    dec_tables: &mut DecTables,
    symbol_ctr: &mut Counter,
    symbol_table: &mut SymbolTable,
    diag: &mut Diagnostic,
) {
    match &statement.stype {
        StatementT::Root { statements } => {
            // we need to first forward declare all functions and global variables.
            symbol_table.push_empty();
            for statement in statements {
                match &statement.stype {
                    StatementT::Declaration {
                        identifier,
                        keyword: id_type,
                        ..
                    }
                    | StatementT::FunctionDeclaration {
                        identifier,
                        return_type: id_type,
                        ..
                    } => {
                        define_symbol(
                            identifier.clone(),
                            statement.node_id,
                            symbol_ctr,
                            &mut dec_tables.link_table,
                            symbol_table,
                        );
                        dec_tables.type_table.push(id_type.clone().into());
                    }
                    _ => (),
                }
            }

            for statement in statements {
                populate_link_table(statement, dec_tables, symbol_ctr, symbol_table, diag);
            }
            symbol_table.pop();
        }
        StatementT::Assignment { identifier, value } => {
            pop_link_tab_exp(&value, &mut dec_tables.link_table, symbol_table, diag); // expression should also be filled.
            if let Some(symbol) = symbol_table.lookup(&identifier) {
                dec_tables.link_table[statement.node_id] = symbol.symbol_id;
            } else {
                // this branch is to minimize errors later.
                // we define x even though its not a good assignment.
                define_symbol(
                    identifier.clone(),
                    statement.node_id,
                    symbol_ctr,
                    &mut dec_tables.link_table,
                    symbol_table,
                );
                dec_tables
                    .type_table
                    .push(ReturnType::Standard(DeclarationKeyword::Error));
                diag.push(statement.span, SemanticError::UseBeforeDefinition);
            }
        }
        StatementT::Declaration {
            identifier,
            keyword,
            assignment,
        } => {
            assignment
                .as_ref()
                .map(|a| pop_link_tab_exp(&a, &mut dec_tables.link_table, symbol_table, diag));
            // check assignment FIRST since then we will catch errors for self-referencing in assigment.
            // e.g. int x=  x+1 not allowed.
            // it does not work for global scope since these are added in the first pass.
            if let Some(symbol) = symbol_table.lookup(identifier)
                && symbol.depth == symbol_table.depth()
                && symbol.depth > SymbolTable::GLOBAL_SCOPE_DEPTH
            // to ensure we do not throw error on global defs
            {
                diag.push(statement.span, SemanticError::AlreadyDefinedInScope);
            } else {
                define_symbol(
                    identifier.clone(),
                    statement.node_id,
                    symbol_ctr,
                    &mut dec_tables.link_table,
                    symbol_table,
                );
                dec_tables.type_table.push(keyword.clone().into());
            }
        }
        StatementT::ExpressionStatement(expr) => pop_link_tab_exp(
            &Expression {
                etype: expr.clone(),
                span: statement.span,
                node_id: statement.node_id,
            },
            &mut dec_tables.link_table,
            symbol_table,
            diag,
        ),
        StatementT::Block { statements } => {
            symbol_table.push_empty();
            for statement in statements {
                populate_link_table(statement, dec_tables, symbol_ctr, symbol_table, diag)
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
                && symbol.depth > SymbolTable::GLOBAL_SCOPE_DEPTH
            {
                //
                diag.push(statement.span, SemanticError::AlreadyDefinedInScope);
            }
            define_symbol(
                identifier.clone(),
                statement.node_id,
                symbol_ctr,
                &mut dec_tables.link_table,
                symbol_table,
            );
            dec_tables.type_table.push(return_type.clone().into());
            symbol_table.push_empty();
            for parameter in parameters {
                populate_link_table(parameter, dec_tables, symbol_ctr, symbol_table, diag);
            }
            for statement in body {
                populate_link_table(statement, dec_tables, symbol_ctr, symbol_table, diag);
            }
            symbol_table.pop();
        }
    }
}

fn pop_link_tab_exp(
    expression: &Expression,
    link_table: &mut Vec<usize>,
    symbol_table: &mut SymbolTable,
    diag: &mut Diagnostic,
) {
    match &expression.etype {
        ExpressionT::Error => {}
        ExpressionT::ValueExpression(val) => match val {
            ValueExpression::Literal(..) => {}
            ValueExpression::Identifier(id) | ValueExpression::CallExpression { id, .. } => {
                if let Some(symbol) = symbol_table.lookup(&id) {
                    link_table[expression.node_id] = symbol.symbol_id;
                } else {
                    diag.push(expression.span, SemanticError::UseBeforeDefinition);
                    // should we define the symbol here? unclear. probably not.
                }
            }
        },
        ExpressionT::BinOp { left, op, right } => {
            pop_link_tab_exp(left.as_ref(), link_table, symbol_table, diag);
            pop_link_tab_exp(right.as_ref(), link_table, symbol_table, diag);
        }
        ExpressionT::UnOp { op, operand } => {
            pop_link_tab_exp(&operand, link_table, symbol_table, diag)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::fmt::Binary;
    use std::os::linux::raw::stat;

    use super::*;
    use crate::lexer::lexer::scan;
    use crate::parser::diagnostic::*;
    use crate::parser::parse_state::ParseState;
    use crate::parser::statement::generate_ast;

    #[test]
    fn test_link_tables() {
        let input = "int a; a = 15;";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (root, size) = generate_ast(&mut state).unwrap();

        let tables: DecTables = get_dec_tables(&root, &mut diag, size);

        println!("{:?}", root);
        println!("{:?}", tables);

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
            ReturnType::Standard(DeclarationKeyword::Int)
        ));
    }
    #[test]
    fn test_link_tables_2() {
        let input = "{
            int a = 5;
            int b = 4;
            int c = d + 4;
            int d;
            }";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (root, size) = generate_ast(&mut state).unwrap();

        let tables: DecTables = get_dec_tables(&root, &mut diag, size);
        type_check_pass(&root, &mut diag, &tables);
        assert_eq!(diag.get_errors().len(), 1);
    }

    #[test]
    fn test_scope_shadowing() {
        let input = "int a; { int a; a = 5; } a = 10;";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (root, size) = generate_ast(&mut state).unwrap();

        let tables: DecTables = get_dec_tables(&root, &mut diag, size);

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

                // 2. Global usage must link to global declaration
                assert_eq!(tables.link_table[global_usage_id], global_decl_id);

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
        let input = "{a = 5;int a = 4;}";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (root, size) = generate_ast(&mut state).unwrap();

        // Depending on your implementation, this should return an Err
        // or the link_table entry should remain usize::MAX
        let result = get_dec_tables(&root, &mut diag, size);
        assert!(diag.has_errors());
        let err = &diag.get_errors()[0];
        match err.error_t {
            ErrorT::SemanticError(SemanticError::UseBeforeDefinition) => assert!(true),
            _ => assert!(false),
        }
    }
}
