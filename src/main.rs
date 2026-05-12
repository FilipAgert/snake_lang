mod diagnostics;
mod lexer;
mod parser;
mod semantics;
use crate::diagnostics::diagnostic::Diagnostic;
use crate::lexer::lexer::scan;
use crate::parser::parse_state::ParsingState;
use crate::parser::statement::generate_ast;
use crate::semantics::semantic_analyser::semantic_analysis;
use std::fs;
use std::{env, process};
fn run_compiler(input: &str) -> i32 {
    let mut diag = Diagnostic::new();
    let mut state = ParsingState::new(scan(input), &mut diag);
    let (root, size) = generate_ast(&mut state).unwrap();
    semantic_analysis(&root, size, &mut diag);

    if diag.has_errors() {
        println!("{} errors found: ", diag.get_errors().len());
        diag.print_errors(input);
        return 1;
    } else {
        println!("Success! No compilation errors :)");
        return 0;
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();

    // Basic validation
    if args.len() < 3 {
        eprintln!("Usage: program -f <file_path> OR program -s <string_content>");
        return;
    }

    let flag = &args[1];
    let value = &args[2];

    let input = match flag.as_str() {
        "-s" => value.to_string(),
        "-f" => match fs::read_to_string(value) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Error reading file {}: {}", value, e);
                return;
            }
        },
        _ => {
            eprintln!("Unknown flag: {}", flag);
            return;
        }
    };

    // Pass 'input' to your lexer
    let err_code = run_compiler(&input);
    process::exit(err_code);
}
