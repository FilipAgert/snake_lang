use std::collections::HashMap;

use crate::parser::diagnostic::Diagnostic;
use crate::parser::parse_state::Counter;
use crate::{
    error::SemanticError,
    lexer::token::{BuiltInType, Span},
    parser::{
        expression::{Expression, ExpressionT, ValueExpression},
        statement::*,
    },
};

#[derive(Clone, PartialEq, Debug)]
pub enum ExpressionType {
    Standard(BuiltInType),
    Pointer(Box<ExpressionType>),
    Custom(Box<str>), // custom datatype
    Error,            // Compiler could not determine type.
}

impl From<BuiltInType> for ExpressionType {
    fn from(value: BuiltInType) -> Self {
        if BuiltInType::Error == value {
            ExpressionType::Error
        } else {
            ExpressionType::Standard(value)
        }
    }
}

// need datastructures now. given a root expression, need to populate ALLnodes into tables.
// we have symbol table. it is transient.
struct Symbol {
    symbol_id: usize,
    depth: usize,
    identifier: Box<str>,
}

struct SymbolTable {
    scopes: Vec<HashMap<Box<str>, Symbol>>,
}

impl SymbolTable {
    pub const GLOBAL_SCOPE_DEPTH: usize = 1;
    fn define(self: &mut Self, id: Box<str>, node_id: usize) {
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
pub struct DecTables<'a> {
    link_table: Vec<usize>,
    type_table: Vec<ExpressionType>,
    ref_table: Vec<&'a Statement>,
}

pub fn type_check_pass(node: &Statement, diag: &mut Diagnostic, dec_tables: &DecTables) {
    match &node.stype {
        StatementT::ErrorStatement => {}
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
            if let ExpressionType::Custom(..) = lhs_type {
                todo!("not implemented custom types yet.");
            }
            let rhs_type = &type_check_pass_expr(value, diag, dec_tables);
            if lhs_type != rhs_type
                && *lhs_type != ExpressionType::Error
                && *rhs_type != ExpressionType::Error
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
        StatementT::Declaration { .. } => {
            let lhs_type = &dec_tables.type_table[dec_tables.link_table[node.node_id]];
            if let ExpressionType::Custom(..) = lhs_type {
                todo!("not implemented custom types yet.");
            }
        } // already checked in above branch.
        StatementT::ExpressionStatement(expr) | StatementT::ReturnStatement(expr) => {
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
        StatementT::If {
            conditional,
            then,
            el,
        } => {
            let conditional_eval = type_check_pass_expr(conditional, diag, dec_tables);
            if conditional_eval != ExpressionType::Standard(BuiltInType::Bool) {
                diag.push(
                    conditional.span.clone(),
                    SemanticError::NonBoolExpressionInConditional,
                );
            }

            type_check_pass(then, diag, dec_tables);
            if let Some(el_block) = el {
                type_check_pass(el_block, diag, dec_tables);
            }
        }
        StatementT::FunctionDeclaration {
            body,
            parameters,
            return_type,
            signature_span,
            ..
        } => {
            for parameter in parameters {
                type_check_pass(parameter, diag, dec_tables);
            }

            for statement in body {
                match &statement.stype {
                    StatementT::ReturnStatement(expr) => {
                        let ret_type = type_check_pass_expr(
                            &Expression {
                                etype: expr.clone(),
                                span: statement.span,
                                node_id: statement.node_id,
                            },
                            diag,
                            dec_tables,
                        );

                        let fn_rettype: &ExpressionType = return_type;
                        if ret_type != *fn_rettype
                            && ret_type != ExpressionType::Error
                            && *fn_rettype != ExpressionType::Error
                        {
                            diag.push(
                                statement.span,
                                SemanticError::IncompatibleReturnType {
                                    fun_ret_type: fn_rettype.clone(),
                                    attempted: ret_type,
                                    function_signature_span: signature_span.clone(),
                                },
                            );
                        }
                    }
                    _ => type_check_pass(statement, diag, dec_tables),
                }
            }
        }
    }
}

fn type_check_pass_expr(
    expr: &Expression,
    diag: &mut Diagnostic,
    dec_tables: &DecTables,
) -> ExpressionType {
    match &expr.etype {
        ExpressionT::Error => ExpressionType::Standard(BuiltInType::Error),
        ExpressionT::ValueExpression(val) => match val {
            ValueExpression::CallExpression { arguments, .. } => {
                let fun_decl_statement = dec_tables.ref_table[dec_tables.link_table[expr.node_id]];
                let parameters: Option<(Vec<(&ExpressionType, &Span)>, Span)> =
                    match &fun_decl_statement.stype {
                        StatementT::ErrorStatement => None, //
                        StatementT::FunctionDeclaration {
                            parameters,
                            signature_span,
                            ..
                        } => {
                            let argtypes: Vec<(&ExpressionType, &Span)> = parameters
                                .iter()
                                .map(|f| {
                                    if let (StatementT::Declaration { datatype, .. }, span) =
                                        (&f.stype, &f.span)
                                    {
                                        (datatype, span)
                                    } else {
                                        panic!("We expect all to be declarations");
                                    }
                                })
                                .collect();
                            Some((argtypes, signature_span.clone()))
                        }
                        _ => {
                            panic!("Should not enter this branch. Linker stage fucked up.")
                        }
                    };

                //check if too many or too few arguments supplied.
                if let Some((v, fns)) = &parameters {
                    let num_arguments = arguments.len();
                    let num_parameters = v.len();
                    if num_arguments > num_parameters {
                        // we know we have at least one argument.
                        let num_excess = num_arguments - num_parameters;
                        let extra_span = Span::merge(
                            &arguments[num_arguments - 1].span,
                            &arguments[num_arguments - num_excess].span,
                        );

                        diag.push(
                            extra_span,
                            SemanticError::TooManyArguments {
                                limit: num_parameters,
                                provided: num_arguments,
                                function_signature_span: *fns,
                            },
                        );
                    } else if num_arguments < num_parameters {
                        let callee_span = expr.span.clone();
                        let num_excess = num_parameters - num_arguments;
                        let missing_params_span =
                            Span::merge(v[num_parameters - 1].1, v[num_parameters - num_excess].1);
                        let missing_params_types: Vec<ExpressionType> = v
                            [num_parameters - num_excess..num_parameters]
                            .iter()
                            .map(|(first, _)| (*first).clone())
                            .collect();

                        diag.push(
                            callee_span,
                            SemanticError::TooFewArguments {
                                desired: num_parameters,
                                provided: num_arguments,
                                missing_span: missing_params_span,
                                missing_types: missing_params_types,
                            },
                        );
                    }
                }

                for (i, argument) in arguments.iter().enumerate() {
                    // check argument of function (that they are not constructed of bad expressions)
                    let argtype = type_check_pass_expr(argument, diag, dec_tables);

                    // check that argument types match parameters
                    if let Some((parameters, ..)) = &parameters {
                        if let Some((param_type, span)) = parameters.get(i) {
                            if argtype != **param_type
                                && argtype != ExpressionType::Error
                                && **param_type != ExpressionType::Error
                            {
                                diag.push(
                                    argument.span.clone(),
                                    SemanticError::InvalidArgumentType {
                                        arg_type: argtype,
                                        parameter_type: (*param_type).clone(),
                                        parameter_span: (*span).clone(),
                                    },
                                );
                            }
                        }
                    }
                }
                dec_tables.type_table[dec_tables.link_table[expr.node_id]].clone() // return type of function.
            }
            ValueExpression::Literal(l) => {
                let decl: BuiltInType = l.clone().into();
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
                if left_type != ExpressionType::Error && right_type != ExpressionType::Error {
                    diag.push(
                        expr.span,
                        SemanticError::IncompatibleTypes {
                            left: left_type,
                            right: right_type,
                        },
                    );
                }
                ExpressionType::Error
            } else {
                left_type
            }
        }
        ExpressionT::UnOp { operand, .. } => type_check_pass_expr(&operand, diag, dec_tables),
    }
}

pub fn get_dec_tables<'a>(
    root: &'a Statement,
    diag: &mut Diagnostic,
    num_ids: usize,
) -> DecTables<'a> {
    let mut symbol_ctr = Counter::new();
    let mut symbol_table = SymbolTable::new();
    let mut dec_tables = DecTables {
        link_table: vec![usize::MAX; num_ids],
        type_table: Vec::new(),
        ref_table: Vec::new(),
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
    id: Box<str>,
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
fn populate_link_table<'a>(
    statement: &'a Statement,
    dec_tables: &mut DecTables<'a>,
    symbol_ctr: &mut Counter,
    symbol_table: &mut SymbolTable,
    diag: &mut Diagnostic,
) {
    match &statement.stype {
        StatementT::ErrorStatement => {}
        StatementT::Root { statements } => {
            // we need to first forward declare all functions and global variables.
            symbol_table.push_empty();
            for statement in statements {
                match &statement.stype {
                    StatementT::Declaration {
                        identifier,
                        datatype: id_type,
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
                        dec_tables.ref_table.push(statement);
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
            pop_link_tab_exp(&value, dec_tables, symbol_ctr, symbol_table, diag); // expression should also be filled.
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
                dec_tables.type_table.push(ExpressionType::Error);
                dec_tables.ref_table.push(statement);
                diag.push(
                    statement.span,
                    SemanticError::UseBeforeDefinition(identifier.clone()),
                );
            }
        }
        StatementT::Declaration {
            identifier,
            datatype: keyword,
            assignment,
        } => {
            if let Some(exp) = assignment {
                pop_link_tab_exp(exp, dec_tables, symbol_ctr, symbol_table, diag);
            }
            // check assignment FIRST since then we will catch errors for self-referencing in assigment.
            // e.g. int x=  x+1 not allowed.
            // it does not work for global scope since these are added in the first pass.
            if let Some(symbol) = symbol_table.lookup(identifier)
                && symbol.depth == symbol_table.depth()
                && symbol.depth > SymbolTable::GLOBAL_SCOPE_DEPTH
            // to ensure we do not throw error on global defs
            {
                diag.push(
                    statement.span,
                    SemanticError::AlreadyDefinedInScope(symbol.identifier.clone()),
                );
                dec_tables.link_table[statement.node_id] = symbol.symbol_id;
            } else {
                define_symbol(
                    identifier.clone(),
                    statement.node_id,
                    symbol_ctr,
                    &mut dec_tables.link_table,
                    symbol_table,
                );
                dec_tables.type_table.push(keyword.clone().into());
                dec_tables.ref_table.push(statement);
            }
        }
        StatementT::ExpressionStatement(expr) | StatementT::ReturnStatement(expr) => {
            pop_link_tab_exp(
                &Expression {
                    etype: expr.clone(),
                    span: statement.span,
                    node_id: statement.node_id,
                },
                dec_tables,
                symbol_ctr,
                symbol_table,
                diag,
            )
        }
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
            ..
        } => {
            if let Some(symbol) = symbol_table.lookup(identifier)
                && symbol.depth == symbol_table.depth()
                && symbol.depth > SymbolTable::GLOBAL_SCOPE_DEPTH
            {
                //
                diag.push(
                    statement.span,
                    SemanticError::AlreadyDefinedInScope(symbol.identifier.clone()),
                );
                dec_tables.link_table[statement.node_id] = symbol.symbol_id;
            } else {
                define_symbol(
                    identifier.clone(),
                    statement.node_id,
                    symbol_ctr,
                    &mut dec_tables.link_table,
                    symbol_table,
                );
                dec_tables.type_table.push(return_type.clone().into());
                dec_tables.ref_table.push(statement);
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
        StatementT::If {
            conditional,
            then,
            el,
        } => {
            pop_link_tab_exp(conditional, dec_tables, symbol_ctr, symbol_table, diag);
            symbol_table.push_empty();
            populate_link_table(then.as_ref(), dec_tables, symbol_ctr, symbol_table, diag);
            symbol_table.pop();
            if let Some(else_block) = el {
                symbol_table.push_empty();
                populate_link_table(else_block, dec_tables, symbol_ctr, symbol_table, diag);
                symbol_table.pop();
            }
        }
    }
}

fn pop_link_tab_exp(
    expression: &Expression,
    dec_tables: &mut DecTables,
    symbol_ctr: &mut Counter,
    symbol_table: &mut SymbolTable,
    diag: &mut Diagnostic,
) {
    match &expression.etype {
        ExpressionT::Error => {}
        ExpressionT::ValueExpression(val) => match val {
            ValueExpression::Literal(..) => {}
            ValueExpression::CallExpression { id, arguments } => {
                if let Some(symbol) = symbol_table.lookup(&id) {
                    dec_tables.link_table[expression.node_id] = symbol.symbol_id;
                } else {
                    diag.push(
                        expression.span,
                        SemanticError::UseBeforeDefinition(id.clone()),
                    );
                    dec_tables.link_table[expression.node_id] = symbol_ctr.next_id();
                    dec_tables.type_table.push(ExpressionType::Error);
                    dec_tables.ref_table.push(&ERROR_STATEMENT);
                    // should we define the symbol here? unclear. probably not.
                }
                for arg in arguments {
                    pop_link_tab_exp(arg, dec_tables, symbol_ctr, symbol_table, diag);
                }
            }
            ValueExpression::Identifier(id) => {
                if let Some(symbol) = symbol_table.lookup(&id) {
                    dec_tables.link_table[expression.node_id] = symbol.symbol_id;
                } else {
                    diag.push(
                        expression.span,
                        SemanticError::UseBeforeDefinition(id.clone()),
                    );
                    dec_tables.link_table[expression.node_id] = symbol_ctr.next_id();
                    dec_tables.type_table.push(ExpressionType::Error);
                    dec_tables.ref_table.push(&ERROR_STATEMENT);
                    // should we define the symbol here? unclear. probably not.
                }
            }
        },
        ExpressionT::BinOp { left, op: _, right } => {
            pop_link_tab_exp(left.as_ref(), dec_tables, symbol_ctr, symbol_table, diag);
            pop_link_tab_exp(right.as_ref(), dec_tables, symbol_ctr, symbol_table, diag);
        }
        ExpressionT::UnOp { operand, .. } => {
            pop_link_tab_exp(&operand, dec_tables, symbol_ctr, symbol_table, diag)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::fmt::Binary;
    use std::os::linux::raw::stat;

    use super::*;
    use crate::error::*;
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

        let (decl_id, usage_id) = if let StatementT::Root { statements } = &root.stype {
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
            ExpressionType::Standard(BuiltInType::Int)
        ));
    }
    #[test]
    fn test_link_tables_2() {
        let input = "{
            int c = d + 4;
            int d;
            }";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (root, size) = generate_ast(&mut state).unwrap();
        let tables: DecTables = get_dec_tables(&root, &mut diag, size);
        println!("num errors: {}", diag.get_errors().len());
        diag.print_errors(input);
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

        if let StatementT::Root { statements } = &root.stype {
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
            ErrorT::SemanticError(SemanticError::UseBeforeDefinition(..)) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_call() {
        let input = "fn foo(int x) : bool {
                            bool a = true;
                            return a;
                            }
                            fn main() : void {
                                int x = foo(4);
                            }";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (mut root, mut size) = generate_ast(&mut state).unwrap();
        let result = get_dec_tables(&root, &mut diag, size);
        diag.print_errors(input);
        assert!(!diag.has_errors());
        type_check_pass(&root, &mut diag, &result);
        let err = &diag.get_errors()[0];
        diag.print_errors(input);
        match &err.error_t {
            ErrorT::SemanticError(SemanticError::IncompatibleTypes { left, right }) => {
                assert!(true)
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_semicolon_error() {
        let input = "{int a}";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (mut root, mut size) = generate_ast(&mut state).unwrap();
        assert!(diag.has_errors());
        let err = &diag.get_errors()[0];
        diag.print_errors(input);
        match &err.error_t {
            ErrorT::StatementError(StatementError::ExpectedToken { .. }) => {
                assert!(true)
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_function_call_type_error() {
        let input = "fn f(int x, bool b): int {
        return 2;
    }
    fn main(): int{
    int a = 2;
    int x = f(1,aparse_expression,3);
    }

    ";
        let mut diag = Diagnostic::new();
        let mut state = ParseState::new(scan(input), &mut diag);
        let (mut root, mut size) = generate_ast(&mut state).unwrap();
        let result = get_dec_tables(&root, &mut diag, size);
        diag.print_errors(input);
        assert!(!diag.has_errors());
        type_check_pass(&root, &mut diag, &result);
        let err = &diag.get_errors()[0];
        diag.print_errors(input);
        match &err.error_t {
            ErrorT::SemanticError(SemanticError::InvalidArgumentType { .. }) => {
                assert!(true)
            }
            _ => assert!(false),
        }
    }
}
