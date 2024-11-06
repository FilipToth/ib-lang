use std::fs;

extern crate ibc;

fn main() {
    let content = fs::read_to_string("lexer_test.ib").unwrap();
    let tokens = ibc::analysis::syntax::lexer::lex(content);

    ibc::analysis::syntax::parser_custom::parse_tokens(tokens);
}
