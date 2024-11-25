pub mod lexer;
pub mod parser;
pub mod syntax_token;

use self::syntax_token::SyntaxToken;

use super::error_bag::ErrorBag;

pub fn parse(content: String, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let tokens = lexer::lex(content);
    parser::parse(tokens, errors)
}
