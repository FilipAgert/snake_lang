use crate::parser::statement::*;

enum SyntaxError {
    IncompatibleTypes,
}
pub fn analyze_statement(statement: &Statement) -> Result<(), SyntaxError> {
    match &statement.stype {
        StatementT::Root { statements } => (),
        _ => todo!(),
    }

    Ok(())
}
