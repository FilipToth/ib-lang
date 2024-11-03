extern crate ibc;

fn main() {
    let content = "12 + 32".to_string();
    let tokens = ibc::analysis::syntax::lexer::lex(content);
    println!("{:?}", tokens);
}
