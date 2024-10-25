#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate pest_derive;

extern crate pest;

use std::fs;

use eval::eval;

mod analysis;
mod eval;

fn parse_file() {
    let contents = fs::read_to_string("test.ib").unwrap();
    let result = analysis::analyze(contents);
    result.errors.report();

    let Some(root) = &result.root else {
        return;
    };

    // evaluate
    let output = eval(root);
    println!("{}", output);
}

fn main() {
    parse_file();
}
