use std::fs;

use ibc::analysis::error_bag::ErrorBag;

extern crate ibc;

fn main() {
    let content = fs::read_to_string("lexer_test.ib").unwrap();
    let tokens = ibc::analysis::syntax::lexer::lex(content);

    for token in &tokens {
        println!("token: {:?}", token);
    }

    println!("");

    let mut error_bag = ErrorBag::new();
    let result = ibc::analysis::syntax::parser::parse(tokens, &mut error_bag);
    println!("{:#?}", result);

    let bound = ibc::analysis::binding::bind_root(&result.unwrap(), &mut error_bag);
    println!("{:#?}", bound);

    for error in error_bag.errors {
        println!("err: {}", error.format());
    }
}
