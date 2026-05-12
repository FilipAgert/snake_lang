use crate::{
    diagnostics::{diagnostic::Diagnostic, error::*, span::Span},
    lexer::token::BuiltInType,
    parser::{
        expression::{Expression, ExpressionT, ValueExpression},
        statement::{ExpressionType, Statement, StatementT},
    },
    semantics::semantic_analyser::DecTables,
};

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
#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::fmt::Binary;
    use std::os::linux::raw::stat;

    use super::*;
    use crate::diagnostics::{diagnostic::*, error::*};
    use crate::lexer::lexer::scan;
    use crate::parser::parse_state::ParseState;
    use crate::parser::statement::generate_ast;
    use crate::semantics::semantic_analyser::get_dec_tables;
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
}
