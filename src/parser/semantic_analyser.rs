use crate::parser::statement::*;

enum SyntaxError {}
pub fn analyze_statement(statement: &Statement) -> Result<(), SyntaxError> {
    match statement {
        Statement::Root { statements } => (),
        _ => todo!(),
    }

    Ok(())
}
