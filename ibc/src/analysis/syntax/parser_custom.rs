use std::{cell::RefCell, iter::Peekable, rc::Rc, slice::Iter};

use super::lexer::{LexerToken, LexerTokenKind};
type LexerTokens<'a> = Rc<RefCell<Peekable<Iter<'a, LexerToken>>>>;

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

fn parse_binary_expression(rc_tokens: LexerTokens, parent_precedence: usize) -> Option<ParsedToken> {
    let mut tokens = rc_tokens.borrow_mut();
    let next_peek = tokens.next();

    let Some(next_peek) = next_peek else {
        return None;
    };

    let unary_precedence = next_peek.kind.unary_operator_precedence();
    let mut lhs = if unary_precedence != 0 {
        // unary expression
        let next = tokens.next().unwrap();
        let operator = match parse_operator(next) {
            Some(o) => o,
            None => return None,
        };

        let rhs = match parse_binary_expression(rc_tokens.clone(), 0) {
            Some(r) => r,
            None => return None,
        };

        let unary_kind = ParsedTokenKind::UnaryExpression {
            op: operator,
            rhs: Box::new(rhs),
        };

        let token = ParsedToken::new(unary_kind);
        Some(token)
    } else {
        parse_primary_expression(rc_tokens.clone())
    };

    while lhs.is_some() {
        let precedence = next_peek.kind.binary_operator_precedence();
        if precedence == 0 || precedence <= parent_precedence {
            break;
        }

        let operator_token = match tokens.next() {
            Some(t) => t,
            None => return None
        };

        let operator = match parse_operator(operator_token) {
            Some(o) => o,
            None => return None
        };

        let rhs = match parse_binary_expression(rc_tokens.clone(), precedence) {
            Some(r) => r,
            None => return None,
        };

        let lhs_owned = match lhs.take() {
            Some(l) => l,
            None => return None,
        };

        let bin_expr_kind = ParsedTokenKind::BinaryExpression { lhs: Box::new(lhs_owned), op: operator, rhs: Box::new(rhs) };
        let bin_expr = ParsedToken::new(bin_expr_kind);
        lhs = Some(bin_expr);
    }

    None
}

fn parse_primary_expression(rc_tokens: LexerTokens) -> Option<ParsedToken> {
    let mut tokens = rc_tokens.borrow_mut();
    let next = match tokens.next() {
        Some(n) => n,
        None => return None
    };

    let kind = match &next.kind {
        LexerTokenKind::IdentifierToken(id) => ParsedTokenKind::ReferenceExpression(id.clone()),
        LexerTokenKind::IntegerLiteralToken(val) => ParsedTokenKind::IntegerLiteralExpression(val.clone()),
        _ => return None
    };

    let token = ParsedToken::new(kind);
    Some(token)
}

fn parse_operator(token: &LexerToken) -> Option<Operator> {
    match token.kind {
        LexerTokenKind::PlusToken => Some(Operator::Addition),
        LexerTokenKind::MinusToken => Some(Operator::Subtraction),
        LexerTokenKind::StarToken => Some(Operator::Multiplication),
        LexerTokenKind::SlashToken => Some(Operator::Division),
        _ => None,
    }
}

fn parse_statement(rc_tokens: LexerTokens) -> Option<ParsedToken> {
    let mut tokens = rc_tokens.borrow_mut();
    let peek = tokens.peek();

    let Some(peek) = peek else { return None };

    match peek.kind {
        LexerTokenKind::OutputKeyword => {}
        LexerTokenKind::IfKeyword => {}
        LexerTokenKind::FunctionKeyword => {}
        LexerTokenKind::IdentifierToken(_) | LexerTokenKind::IntegerLiteralToken(_) => {
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

pub fn parse_tokens(tokens: Vec<LexerToken>) {
    let mut iter = tokens.iter().peekable();
    parse_global_scope(Rc::new(RefCell::new(iter)));
}
