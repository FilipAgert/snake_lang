use crate::diagnostics::diagnostic::Diagnostic;
use crate::parser::statement::Statement;
use crate::semantics::link_stage::{DeclarationTables, get_dec_tables};
use crate::semantics::type_check::type_check_pass;

pub fn semantic_analysis(root: &Statement, num_nodes: usize, diagnostic: &mut Diagnostic) {
    let tables: DeclarationTables = get_dec_tables(root, diagnostic, num_nodes);

    type_check_pass(root, diagnostic, &tables);
}
