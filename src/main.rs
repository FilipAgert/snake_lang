mod lexer;

use lexer::lexer::scan;
use lexer::token::{Keyword, Operator, Token};

fn main() {
    let input = "int x = 5;";

    let tokens = scan(input);
}
