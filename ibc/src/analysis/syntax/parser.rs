use std::{iter::Peekable, slice::Iter};

use crate::analysis::{operator::Operator, CodeLocation};

use super::{lexer::{LexerToken, LexerTokenKind}, syntax_token::{SyntaxKind, SyntaxToken}};

type LexerTokens<'a> = Peekable<Iter<'a, LexerToken>>;

struct Parser<'a> {
    tokens: LexerTokens<'a>,
}

impl<'a> Parser<'a> {
    fn new(tokens: LexerTokens<'a>) -> Self {
        Parser { tokens: tokens }
    }

    fn parse_expression(&mut self) -> Option<SyntaxToken> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, parent_precedence: usize) -> Option<SyntaxToken> {
        let unary_precedence = match self.tokens.peek() {
            Some(t) => t.kind.unary_operator_precedence(),
            None => return None,
        };

        let mut lhs = if unary_precedence != 0 && unary_precedence >= parent_precedence {
            // unary expression
            let next = self.tokens.next().unwrap();
            let next_loc = next.loc.clone();

            let operator = match self.parse_operator(next) {
                Some(o) => o,
                None => return None,
            };

            let rhs = match self.parse_binary_expression(unary_precedence) {
                Some(r) => r,
                None => return None,
            };

            let unary_kind = SyntaxKind::UnaryExpression {
                op: operator,
                rhs: Box::new(rhs),
            };

            let token = SyntaxToken::new(unary_kind, next_loc);
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

            let loc = lhs_token.loc.clone();
            let bin_expr_kind = SyntaxKind::BinaryExpression {
                lhs: Box::new(lhs_token),
                op: operator,
                rhs: Box::new(rhs),
            };

            let bin_expr = SyntaxToken::new(bin_expr_kind, loc);
            lhs = Some(bin_expr);
        }

        lhs
    }

    fn parse_primary_expression(&mut self) -> Option<SyntaxToken> {
        match self.tokens.peek() {
            Some(p) => {
                // check if kind is valid
                let loc = p.loc.clone();
                match &p.kind {
                    LexerTokenKind::OpenParenthesisToken => self.parse_parenthesis_expression(),
                    LexerTokenKind::IdentifierToken(_) => {
                        self.parse_reference_or_call_or_assignment()
                    }
                    LexerTokenKind::IntegerLiteralToken(val) => {
                        // consume token
                        self.tokens.next();

                        let kind = SyntaxKind::IntegerLiteralExpression(val.clone());
                        let token = SyntaxToken::new(kind, loc);
                        Some(token)
                    }
                    LexerTokenKind::TrueKeyword => {
                        // consume token
                        self.tokens.next();

                        let kind = SyntaxKind::BooleanLiteralExpression(true);
                        let token = SyntaxToken::new(kind, loc);
                        Some(token)
                    }
                    LexerTokenKind::FalseKeyword => {
                        // consume token
                        self.tokens.next();

                        let kind = SyntaxKind::BooleanLiteralExpression(false);
                        let token = SyntaxToken::new(kind, loc);
                        Some(token)
                    }
                    _ => return None,
                }
            }
            None => return None,
        }
    }

    fn parse_reference_or_call_or_assignment(&mut self) -> Option<SyntaxToken> {
        let (identifier, loc) = match self.parse_identifier() {
            Some(i) => i,
            None => return None,
        };

        let reference_kind = SyntaxKind::ReferenceExpression(identifier.clone());
        let reference = SyntaxToken::new(reference_kind, loc.clone());

        let peek = match self.tokens.peek() {
            Some(p) => p,
            None => return Some(reference),
        };

        match peek.kind {
            LexerTokenKind::OpenParenthesisToken => {
                // call expression
                let arguments = match self.parse_argument_list() {
                    Some(a) => a,
                    None => return None,
                };

                let kind = SyntaxKind::CallExpression {
                    identifier: identifier,
                    args: arguments,
                };

                let token = SyntaxToken::new(kind, loc.clone());
                Some(token)
            }
            LexerTokenKind::EqualsToken => {
                // assignment expression
                let _equals = self.tokens.next();
                let value = match self.parse_expression() {
                    Some(e) => e,
                    None => return None,
                };

                let kind = SyntaxKind::AssignmentExpression {
                    identifier: identifier,
                    value: Box::new(value),
                };

                let token = SyntaxToken::new(kind, loc.clone());
                Some(token)
            }
            _ => Some(reference),
        }
    }

    fn parse_argument_list(&mut self) -> Option<Vec<SyntaxToken>> {
        let _open_paren = self.tokens.next();

        let mut args: Vec<SyntaxToken> = Vec::new();
        let mut prev_comma = false;

        loop {
            let peek = match self.tokens.peek() {
                Some(p) => p,
                None => return None,
            };

            if let LexerTokenKind::CloseParenthesisToken = peek.kind {
                if prev_comma {
                    return None;
                }

                break;
            }

            let expr = match self.parse_expression() {
                Some(e) => e,
                None => return None,
            };

            let peek = match self.tokens.peek() {
                Some(p) => p,
                None => return None,
            };

            prev_comma = false;
            match peek.kind {
                LexerTokenKind::CommaToken => {
                    prev_comma = true;
                    self.tokens.next();
                }
                LexerTokenKind::CloseParenthesisToken => {}
                _ => return None,
            };

            args.push(expr);
        }

        Some(args)
    }

    fn parse_parenthesis_expression(&mut self) -> Option<SyntaxToken> {
        let _left_paren = self.tokens.next();
        match self.parse_expression() {
            Some(expr) => {
                let right_paren = self.tokens.next();
                match right_paren {
                    Some(_) => {
                        let loc = expr.loc.clone();
                        let kind = SyntaxKind::ParenthesizedExpression {
                            inner: Box::new(expr),
                        };

                        let token = SyntaxToken::new(kind, loc);
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

    fn parse_output_statement(&mut self) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();

        let expr = self.parse_expression();
        match expr {
            Some(expr) => {
                let kind = SyntaxKind::OutputStatement {
                    expr: Box::new(expr),
                };
                let token = SyntaxToken::new(kind, keyword.loc.clone());
                Some(token)
            }
            None => None,
        }
    }

    fn parse_if_statement(&mut self) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let condition = match self.parse_expression() {
            Some(c) => c,
            None => return None,
        };

        // then keyword
        if !self.expect_next_token(LexerTokenKind::ThenKeyword) {
            return None;
        }

        let body = match self.parse_scope() {
            Some(b) => b,
            None => return None,
        };

        // end keyword
        if !self.expect_next_token(LexerTokenKind::EndKeyword) {
            return None;
        }

        let kind = SyntaxKind::IfStatement {
            condition: Box::new(condition),
            body: Box::new(body),
        };

        let token = SyntaxToken::new(kind, keyword.loc.clone());
        Some(token)
    }

    fn parse_return_statement(&mut self) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let expr = match self.parse_expression() {
            Some(e) => Some(Box::new(e)),
            None => None,
        };

        let kind = SyntaxKind::ReturnStatement { expr: expr };
        let token = SyntaxToken::new(kind, keyword.loc.clone());
        Some(token)
    }

    fn parse_function_declaration(&mut self) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();

        // identifier
        let identifier = match self.parse_identifier() {
            Some((i, _)) => i,
            None => return None,
        };

        // parameter list
        let parameters = match self.parse_parameter_list() {
            Some(p) => p,
            None => return None,
        };

        let mut return_type: Option<String> = None;
        if self.expect_next_token_peek(LexerTokenKind::ArrowToken) {
            let _arrow = self.tokens.next();
            match self.parse_identifier() {
                Some((i, _)) => return_type = Some(i.clone()),
                None => return None,
            };
        }

        // body
        let body = match self.parse_scope() {
            Some(b) => b,
            None => return None,
        };

        // end keyword
        match self.tokens.next() {
            Some(t) => match &t.kind {
                LexerTokenKind::EndKeyword => {}
                _ => return None,
            },
            None => return None,
        };

        let kind = SyntaxKind::FunctionDeclaration {
            identifier: identifier,
            parameters: parameters,
            return_type: return_type,
            body: Box::new(body),
        };

        let token = SyntaxToken::new(kind, keyword.loc.clone());
        Some(token)
    }

    fn parse_parameter_list(&mut self) -> Option<Vec<SyntaxToken>> {
        if !self.expect_next_token(LexerTokenKind::OpenParenthesisToken) {
            return None;
        }

        let mut params: Vec<SyntaxToken> = Vec::new();
        let mut prev_comma = false;

        loop {
            let peek = match self.tokens.peek() {
                Some(t) => t,
                None => return None,
            };

            match peek.kind {
                LexerTokenKind::CloseParenthesisToken => {
                    if prev_comma {
                        // comma, but no further
                        // params provided
                        return None;
                    }

                    self.tokens.next();
                    break;
                }
                LexerTokenKind::IdentifierToken(_) => {}
                _ => {
                    // expected identifier
                    return None;
                }
            };

            let (identifier, loc) = match self.parse_identifier() {
                Some(i) => i,
                None => return None,
            };

            if !self.expect_next_token(LexerTokenKind::ColonToken) {
                return None;
            }

            let type_annotation = match self.parse_identifier() {
                Some((i, _)) => i,
                None => return None,
            };

            prev_comma = false;
            match self.tokens.peek() {
                Some(t) => match t.kind {
                    LexerTokenKind::CommaToken => {
                        prev_comma = true;
                        self.tokens.next();
                    }
                    LexerTokenKind::CloseParenthesisToken => {}
                    _ => {
                        // expected comma
                        return None;
                    }
                },
                None => {}
            };

            let kind = SyntaxKind::Parameter {
                identifier: identifier,
                type_annotation: type_annotation,
            };

            let token = SyntaxToken::new(kind, loc);
            params.push(token);
        }

        Some(params)
    }

    fn parse_statement(&mut self) -> Option<SyntaxToken> {
        let peek = self.tokens.peek();

        let Some(peek) = peek else { return None };
        match peek.kind {
            LexerTokenKind::OutputKeyword => self.parse_output_statement(),
            LexerTokenKind::IfKeyword => self.parse_if_statement(),
            LexerTokenKind::ReturnKeyword => self.parse_return_statement(),
            LexerTokenKind::FunctionKeyword => self.parse_function_declaration(),
            _ => self.parse_expression(),
        }
    }

    fn parse_scope(&mut self) -> Option<SyntaxToken> {
        let mut parsed: Vec<SyntaxToken> = Vec::new();
        loop {
            let statement = self.parse_statement();
            match statement {
                Some(s) => parsed.push(s),
                None => break,
            };
        }

        let loc = match parsed.first() {
            Some(f) => f.loc.clone(),
            None => CodeLocation::new(0, 0),
        };

        let scope_kind = SyntaxKind::Scope { subtokens: parsed };
        let token = SyntaxToken::new(scope_kind, loc);
        Some(token)
    }

    fn parse_identifier(&mut self) -> Option<(String, CodeLocation)> {
        match self.tokens.next() {
            Some(t) => match &t.kind {
                LexerTokenKind::IdentifierToken(id) => Some((id.clone(), t.loc.clone())),
                _ => return None,
            },
            None => return None,
        }
    }

    fn expect_next_token(&mut self, kind: LexerTokenKind) -> bool {
        match self.tokens.next() {
            Some(t) => {
                if t.kind == kind {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn expect_next_token_peek(&mut self, kind: LexerTokenKind) -> bool {
        match self.tokens.peek() {
            Some(t) => {
                if t.kind == kind {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }
}

pub fn parse(tokens: Vec<LexerToken>) -> Option<SyntaxToken> {
    let iter = tokens.iter().peekable();
    let mut parser = Parser::new(iter);

    parser.parse_scope()
}
