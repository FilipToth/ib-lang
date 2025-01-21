use std::{fs, io::stdin};

use eval::eval;

mod analysis;
mod eval;

fn input_handler() -> String {
    let mut input = String::new();

    stdin()
        .read_line(&mut input)
        .expect("Failed to get user input");

    // remove newline
    input.pop();

    input
}

fn parse_file() {
    let contents = fs::read_to_string("test.ib").unwrap();
    let result = analysis::analyze(contents);
    result.errors.report();

    let Some(root) = &result.root else {
        return;
    };

    // evaluate
    let output = eval(root, input_handler);
    println!("{}", output);
}

fn main() {
    parse_file();
}
