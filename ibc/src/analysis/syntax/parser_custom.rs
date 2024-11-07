use std::{cell::RefCell, iter::Peekable, rc::Rc, slice::Iter};

use super::lexer::{LexerToken, LexerTokenKind};

type LexerTokens<'a> = Peekable<Iter<'a, LexerToken>>;

#[derive(Debug)]
pub struct ParsedToken {
    pub kind: ParsedTokenKind,
}

impl ParsedToken {
    fn new(kind: ParsedTokenKind) -> ParsedToken {
        ParsedToken { kind: kind }
    }
}

#[derive(Debug)]
pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Not,
    Equality,
}

#[derive(Debug)]
pub enum ParsedTokenKind {
    Scope {
        subtokens: Vec<ParsedToken>,
    },
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
    ParenthesizedExpression {
        inner: Box<ParsedToken>,
    },
    OutputStatement {
        expr: Box<ParsedToken>,
    },
    IfStatement {
        condition: Box<ParsedToken>,
        body: Box<ParsedToken>,
    },
}

struct Parser<'a> {
    tokens: LexerTokens<'a>,
}

impl<'a> Parser<'a> {
    fn new(tokens: LexerTokens<'a>) -> Self {
        Parser { tokens: tokens }
    }

    fn parse_expression(&mut self) -> Option<ParsedToken> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, parent_precedence: usize) -> Option<ParsedToken> {
        let unary_precedence = match self.tokens.peek() {
            Some(t) => t.kind.unary_operator_precedence(),
            None => return None,
        };

        let mut lhs = if unary_precedence != 0 && unary_precedence >= parent_precedence {
            // unary expression
            let next = self.tokens.next().unwrap();
            let operator = match self.parse_operator(next) {
                Some(o) => o,
                None => return None,
            };

            let rhs = match self.parse_binary_expression(unary_precedence) {
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
            self.parse_primary_expression()
        };

        while let Some(lhs_token) = lhs.take() {
            let precedence = match self.tokens.peek() {
                Some(t) => t.kind.binary_operator_precedence(),
                None => {
                    lhs = Some(lhs_token);
                    break;
                }
            };

            if precedence == 0 || precedence <= parent_precedence {
                lhs = Some(lhs_token);
                break;
            }

            let operator_token = match self.tokens.next() {
                Some(t) => t,
                None => break,
            };

            let operator = match self.parse_operator(operator_token) {
                Some(o) => o,
                None => return None,
            };

            let rhs = match self.parse_binary_expression(precedence) {
                Some(r) => r,
                None => break,
            };

            let bin_expr_kind = ParsedTokenKind::BinaryExpression {
                lhs: Box::new(lhs_token),
                op: operator,
                rhs: Box::new(rhs),
            };
            let bin_expr = ParsedToken::new(bin_expr_kind);
            lhs = Some(bin_expr);
        }

        lhs
    }

    fn parse_primary_expression(&mut self) -> Option<ParsedToken> {
        // do not consume token yet, check if
        // peek is valid for consumption
        match self.tokens.peek() {
            Some(p) => {
                // check if kind is valid
                match &p.kind {
                    LexerTokenKind::OpenParenthesisToken
                    | LexerTokenKind::IdentifierToken(_)
                    | LexerTokenKind::IntegerLiteralToken(_) => {}
                    _ => return None,
                }
            }
            None => return None,
        };

        let next = self.tokens.next().unwrap();
        if let LexerTokenKind::OpenParenthesisToken = &next.kind {
            return self.parse_parenthesis_expression();
        }

        let kind = match &next.kind {
            LexerTokenKind::IdentifierToken(id) => ParsedTokenKind::ReferenceExpression(id.clone()),
            LexerTokenKind::IntegerLiteralToken(val) => {
                ParsedTokenKind::IntegerLiteralExpression(val.clone())
            }
            _ => return None,
        };

        let token = ParsedToken::new(kind);
        Some(token)
    }

    fn parse_parenthesis_expression(&mut self) -> Option<ParsedToken> {
        match self.parse_expression() {
            Some(expr) => {
                let right_paren = self.tokens.next();
                match right_paren {
                    Some(_) => {
                        let kind = ParsedTokenKind::ParenthesizedExpression {
                            inner: Box::new(expr),
                        };
                        let token = ParsedToken::new(kind);
                        Some(token)
                    }
                    None => None,
                }
            }
            None => None,
        }
    }

    fn parse_operator(&self, token: &LexerToken) -> Option<Operator> {
        match token.kind {
            LexerTokenKind::PlusToken => Some(Operator::Addition),
            LexerTokenKind::MinusToken => Some(Operator::Subtraction),
            LexerTokenKind::StarToken => Some(Operator::Multiplication),
            LexerTokenKind::SlashToken => Some(Operator::Division),
            LexerTokenKind::EqualsEqualsToken => Some(Operator::Equality),
            LexerTokenKind::BangToken => Some(Operator::Not),
            _ => None,
        }
    }

    fn parse_output_statement(&mut self) -> Option<ParsedToken> {
        let _keyword = self.tokens.next();
        let expr = self.parse_expression();
        match expr {
            Some(expr) => {
                let kind = ParsedTokenKind::OutputStatement {
                    expr: Box::new(expr),
                };
                let token = ParsedToken::new(kind);
                Some(token)
            }
            None => None,
        }
    }

    fn parse_if_statement(&mut self) -> Option<ParsedToken> {
        let _keyword = self.tokens.next();
        let condition = match self.parse_expression() {
            Some(c) => c,
            None => return None,
        };

        // then keyword
        match self.tokens.next() {
            Some(n) => match &n.kind {
                LexerTokenKind::ThenKeyword => {}
                _ => return None,
            },
            None => return None,
        };

        let body = match self.parse_scope() {
            Some(b) => b,
            None => return None,
        };

        // end keyword
        match self.tokens.next() {
            Some(n) => match &n.kind {
                LexerTokenKind::EndKeyword => {}
                _ => return None,
            },
            None => return None,
        };

        let kind = ParsedTokenKind::IfStatement {
            condition: Box::new(condition),
            body: Box::new(body),
        };

        let token = ParsedToken::new(kind);
        Some(token)
    }

    fn parse_statement(&mut self) -> Option<ParsedToken> {
        let peek = self.tokens.peek();

        let Some(peek) = peek else { return None };
        match peek.kind {
            LexerTokenKind::OutputKeyword => self.parse_output_statement(),
            LexerTokenKind::IfKeyword => self.parse_if_statement(),
            LexerTokenKind::FunctionKeyword => None,
            _ => self.parse_expression(),
        }
    }

    fn parse_scope(&mut self) -> Option<ParsedToken> {
        let mut parsed: Vec<ParsedToken> = Vec::new();
        loop {
            let statement = self.parse_statement();
            match statement {
                Some(s) => parsed.push(s),
                None => break,
            };
        }

        let scope_kind = ParsedTokenKind::Scope { subtokens: parsed };

        let token = ParsedToken::new(scope_kind);
        Some(token)
    }
}

pub fn parse_tokens(tokens: Vec<LexerToken>) {
    let iter = tokens.iter().peekable();
    let mut parser = Parser::new(iter);
    let res = parser.parse_scope();

    println!("{:#?}", res);
}
