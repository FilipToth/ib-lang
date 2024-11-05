use std::{cell::RefCell, iter::Peekable, rc::Rc, slice::Iter};

use super::lexer::{SyntaxToken, SyntaxTokenKind};
type LexerTokens<'a> = Rc<RefCell<Peekable<Iter<'a, SyntaxToken>>>>;

pub struct ParsedToken {
    pub kind: ParsedTokenKind,
}

impl ParsedToken {
    fn new(kind: ParsedTokenKind) -> ParsedToken {
        ParsedToken { kind: kind }
    }
}

pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Not,
    Equality,
}

pub enum ParsedTokenKind {
    ReferenceExpression(String),
    IntegerLiteralExpression(i64),
    BinaryExpression {
        lhs: Box<ParsedToken>,
        op: Operator,
        rhs: Box<ParsedToken>,
    },
    UnaryExpression {
        op: Operator,
        rhs: Box<ParsedToken>,
    },
}

fn parse_expression(rc_tokens: LexerTokens) -> Option<ParsedToken> {
    // binary or unary, reference or literal
    let mut tokens = rc_tokens.borrow_mut();

    let first = tokens.peek();
    let Some(first) = first else {
        return None;
    };

    match first.kind {
        SyntaxTokenKind::BangToken => {
            // unary operator
        }
        _ => {
            // check second, if operator, then binary, else reference...
            let first = tokens.next().unwrap();
            let second = tokens.peek();

            let Some(second) = second else {
                // reference or litera
                return None;
            };

            match second.kind {
                SyntaxTokenKind::PlusToken
                | SyntaxTokenKind::MinusToken
                | SyntaxTokenKind::StarToken
                | SyntaxTokenKind::SlashToken => {
                    // binary expression
                    parse_binary_expression(rc_tokens.clone());
                }
                _ => {
                    // reference or literal
                }
            }
        }
    }

    None
}

fn parse_binary_expression(rc_tokens: LexerTokens) -> Option<ParsedToken> {
    let mut tokens = rc_tokens.borrow_mut();
    let next = tokens.next();

    let Some(next) = next else {
        return None;
    };

    let unary_precedence = next.kind.unary_operator_precedence();
    if unary_precedence != 0 {
        // unary expression
        let operator = match parse_operator(next) {
            Some(o) => o,
            None => return None,
        };

        let rhs = match parse_expression(rc_tokens.clone()) {
            Some(r) => r,
            None => return None,
        };

        let unary_kind = ParsedTokenKind::UnaryExpression {
            op: operator,
            rhs: Box::new(rhs),
        };

        let token = ParsedToken::new(unary_kind);
        return Some(token);
    }

    None
}

fn parse_operator(token: &SyntaxToken) -> Option<Operator> {
    match token.kind {
        SyntaxTokenKind::PlusToken => Some(Operator::Addition),
        SyntaxTokenKind::MinusToken => Some(Operator::Subtraction),
        SyntaxTokenKind::StarToken => Some(Operator::Multiplication),
        SyntaxTokenKind::SlashToken => Some(Operator::Division),
        _ => None,
    }
}

fn parse_statement(rc_tokens: LexerTokens) -> Option<ParsedToken> {
    let mut tokens = rc_tokens.borrow_mut();
    let peek = tokens.peek();

    let Some(peek) = peek else { return None };

    match peek.kind {
        SyntaxTokenKind::OutputKeyword => {}
        SyntaxTokenKind::IfKeyword => {}
        SyntaxTokenKind::FunctionKeyword => {}
        SyntaxTokenKind::IdentifierToken(_) | SyntaxTokenKind::IntegerToken(_) => {
            // parse expression
        }
        _ => unreachable!(),
    };

    None
}

fn parse_global_scope(rc_tokens: LexerTokens) {
    loop {
        let statement = parse_statement(rc_tokens.clone());
        match statement {
            Some(s) => {}
            None => break,
        }
    }
}

pub fn parse_tokens(tokens: Vec<SyntaxToken>) {
    let mut iter = tokens.iter().peekable();
    parse_global_scope(Rc::new(RefCell::new(iter)));
}
