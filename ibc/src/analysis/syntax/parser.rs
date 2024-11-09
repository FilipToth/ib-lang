use std::{iter::Peekable, slice::Iter};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    CodeLocation,
};

use super::{
    lexer::{LexerToken, LexerTokenKind},
    syntax_token::{SyntaxKind, SyntaxToken},
};

type LexerTokens<'a> = Peekable<Iter<'a, LexerToken>>;

struct Parser<'a> {
    tokens: LexerTokens<'a>,
}

impl<'a> Parser<'a> {
    fn new(tokens: LexerTokens<'a>) -> Self {
        Parser { tokens: tokens }
    }

    fn parse_expression(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        self.parse_binary_expression(0, errors)
    }

    fn parse_binary_expression(
        &mut self,
        parent_precedence: usize,
        errors: &mut ErrorBag,
    ) -> Option<SyntaxToken> {
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
                None => {
                    let error_kind = ErrorKind::UnknownOperator(next.kind.to_string());
                    errors.add(error_kind, next.loc.line, next.loc.col);
                    return None;
                }
            };

            let rhs = match self.parse_binary_expression(unary_precedence, errors) {
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
            self.parse_primary_expression(errors)
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
                None => {
                    let error_kind = ErrorKind::ExpectedToken("binary operator".to_string());
                    errors.add(error_kind, lhs_token.loc.line, lhs_token.loc.col);
                    break;
                }
            };

            let operator = match self.parse_operator(operator_token) {
                Some(o) => o,
                None => {
                    let error_kind = ErrorKind::UnknownOperator(operator_token.kind.to_string());
                    errors.add(error_kind, operator_token.loc.line, operator_token.loc.col);
                    return None;
                }
            };

            let rhs = match self.parse_binary_expression(precedence, errors) {
                Some(r) => r,
                None => {
                    let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                    errors.add(error_kind, operator_token.loc.line, operator_token.loc.col);
                    return None;
                }
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

    fn parse_primary_expression(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        match self.tokens.peek() {
            Some(p) => {
                // check if kind is valid
                let loc = p.loc.clone();
                match &p.kind {
                    LexerTokenKind::OpenParenthesisToken => {
                        self.parse_parenthesis_expression(errors)
                    }
                    LexerTokenKind::IdentifierToken(_) => {
                        self.parse_reference_or_call_or_assignment(errors)
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
            None => {
                // TODO: Add location, use some sort of last parsed token location
                let error_kind = ErrorKind::ExpectedPrimaryExpression;
                errors.add(error_kind, 0, 0);
                return None;
            }
        }
    }

    fn parse_reference_or_call_or_assignment(
        &mut self,
        errors: &mut ErrorBag,
    ) -> Option<SyntaxToken> {
        // theoretically shouldn't be none
        let (identifier, loc) = match self.parse_identifier() {
            Some(i) => i,
            None => {
                // TODO: Agains use some sort of last token loc
                let error_kind = ErrorKind::ExpectedToken("identifier".to_string());
                errors.add(error_kind, 0, 0);
                return None;
            }
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
                let arguments = match self.parse_argument_list(errors) {
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
                let equals = self.tokens.next().unwrap();
                let value = match self.parse_expression(errors) {
                    Some(e) => e,
                    None => {
                        let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                        errors.add(error_kind, equals.loc.line, equals.loc.col);
                        return None;
                    }
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

    fn parse_argument_list(&mut self, errors: &mut ErrorBag) -> Option<Vec<SyntaxToken>> {
        let _open_paren = self.tokens.next();

        let mut args: Vec<SyntaxToken> = Vec::new();
        let mut prev_comma = false;

        loop {
            let peek = match self.tokens.peek() {
                Some(p) => p,
                None => return None,
            };

            if let LexerTokenKind::CloseParenthesisToken = peek.kind {
                let peek_loc = peek.loc.clone();
                self.tokens.next();

                if prev_comma {
                    let error_kind = ErrorKind::ExpectedArgument;
                    errors.add(error_kind, peek_loc.line, peek_loc.col);
                    return None;
                }

                break;
            }

            let expr = match self.parse_expression(errors) {
                Some(e) => e,
                None => return None,
            };

            let peek = match self.tokens.peek() {
                Some(p) => p,
                None => {
                    let error_kind = ErrorKind::ExpectedToken("close parenthesis ')'".to_string());
                    errors.add(error_kind, expr.loc.line, expr.loc.col);
                    return None;
                }
            };

            prev_comma = false;
            match peek.kind {
                LexerTokenKind::CommaToken => {
                    prev_comma = true;
                    self.tokens.next();
                }
                LexerTokenKind::CloseParenthesisToken => {}
                _ => {
                    let error_kind = ErrorKind::ExpectedToken("close parenthesis ')'".to_string());
                    errors.add(error_kind, peek.loc.line, peek.loc.col);
                    return None;
                }
            };

            args.push(expr);
        }

        Some(args)
    }

    fn parse_parenthesis_expression(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let _left_paren = self.tokens.next();
        match self.parse_expression(errors) {
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
                    None => {
                        let error_kind = ErrorKind::UnclosedParenthesisExpression;
                        errors.add(error_kind, expr.loc.line, expr.loc.col);
                        None
                    }
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

    fn parse_output_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();

        let expr = self.parse_expression(errors);
        match expr {
            Some(expr) => {
                let kind = SyntaxKind::OutputStatement {
                    expr: Box::new(expr),
                };

                let token = SyntaxToken::new(kind, keyword.loc.clone());
                Some(token)
            }
            None => {
                let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                errors.add(error_kind, keyword.loc.line, keyword.loc.col);
                return None;
            }
        }
    }

    fn parse_if_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let condition = match self.parse_expression(errors) {
            Some(c) => c,
            None => {
                let error_kind = ErrorKind::ExpectedToken("condition expression".to_string());
                errors.add(error_kind, keyword.loc.line, keyword.loc.col);
                return None;
            }
        };

        // then keyword
        if !self.expect_next_token(LexerTokenKind::ThenKeyword) {
            let error_kind = ErrorKind::ExpectedToken("then keyword".to_string());
            errors.add(error_kind, condition.loc.line, condition.loc.col);
            return None;
        }

        let body = match self.parse_scope(errors) {
            Some(b) => b,
            None => {
                let error_kind = ErrorKind::ExpectedScope;
                errors.add(error_kind, condition.loc.line, condition.loc.col);
                return None;
            }
        };

        // end keyword
        if !self.expect_next_token(LexerTokenKind::EndKeyword) {
            let error_kind = ErrorKind::ExpectedToken("end keyword".to_string());
            errors.add(error_kind, condition.loc.line, condition.loc.col);
            return None;
        }

        let kind = SyntaxKind::IfStatement {
            condition: Box::new(condition),
            body: Box::new(body),
        };

        let token = SyntaxToken::new(kind, keyword.loc.clone());
        Some(token)
    }

    fn parse_return_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let expr = match self.parse_expression(errors) {
            Some(e) => Some(Box::new(e)),
            None => None,
        };

        let kind = SyntaxKind::ReturnStatement { expr: expr };
        let token = SyntaxToken::new(kind, keyword.loc.clone());
        Some(token)
    }

    fn parse_function_declaration(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();

        // identifier
        let identifier = match self.parse_identifier() {
            Some((i, _)) => i,
            None => {
                let error_kind = ErrorKind::ExpectedToken("identifier".to_string());
                errors.add(error_kind, keyword.loc.line, keyword.loc.col);
                return None;
            }
        };

        // parameter list
        let parameters = match self.parse_parameter_list(errors) {
            Some(p) => p,
            None => return None,
        };

        let mut return_type: Option<String> = None;
        if self.expect_next_token_peek(LexerTokenKind::ArrowToken) {
            let arrow = self.tokens.next().unwrap();
            match self.parse_identifier() {
                Some((i, _)) => return_type = Some(i.clone()),
                None => {
                    let error_kind = ErrorKind::ExpectedToken("identifier".to_string());
                    errors.add(error_kind, arrow.loc.line, arrow.loc.col);
                }
            };
        }

        // body
        let body = match self.parse_scope(errors) {
            Some(b) => b,
            None => {
                let error_kind = ErrorKind::ExpectedToken("function body".to_string());
                errors.add(error_kind, keyword.loc.line, keyword.loc.col);
                return None;
            }
        };

        // end keyword
        match self.tokens.next() {
            Some(t) => match &t.kind {
                LexerTokenKind::EndKeyword => {}
                _ => {
                    let error_kind = ErrorKind::ExpectedToken("end".to_string());
                    errors.add(error_kind, t.loc.line, t.loc.col);
                    return None;
                }
            },
            None => {
                let error_kind = ErrorKind::ExpectedToken("end".to_string());
                errors.add(error_kind, keyword.loc.line, keyword.loc.col);
                return None;
            }
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

    fn parse_parameter_list(&mut self, errors: &mut ErrorBag) -> Option<Vec<SyntaxToken>> {
        if !self.expect_next_token(LexerTokenKind::OpenParenthesisToken) {
            // TODO: Use last token loc to report error
            let error_kind = ErrorKind::ExpectedToken("open parenthesis '('".to_string());
            errors.add(error_kind, 0, 0);
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
                        let error_kind = ErrorKind::ExpectedParameter;
                        errors.add(error_kind, peek.loc.line, peek.loc.col);
                        return None;
                    }

                    self.tokens.next();
                    break;
                }
                LexerTokenKind::IdentifierToken(_) => {}
                _ => {
                    // expected identifier
                    let error_kind = ErrorKind::ExpectedToken("identifier".to_string());
                    errors.add(error_kind, peek.loc.line, peek.loc.col);
                    return None;
                }
            };

            let (identifier, loc) = match self.parse_identifier() {
                Some(i) => i,
                None => return None,
            };

            if !self.expect_next_token(LexerTokenKind::ColonToken) {
                let error_kind = ErrorKind::ExpectedToken("type annotation".to_string());
                errors.add(error_kind, loc.line, loc.col);
                return None;
            }

            let type_annotation = match self.parse_identifier() {
                Some((i, _)) => i,
                None => {
                    let error_kind = ErrorKind::ExpectedToken("type annotation".to_string());
                    errors.add(error_kind, loc.line, loc.col);
                    return None;
                }
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
                        let error_kind = ErrorKind::ExpectedToken(
                            "comma ',' or close parenthesis ')'".to_string(),
                        );
                        errors.add(error_kind, t.loc.line, t.loc.col);
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

    fn parse_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let peek = self.tokens.peek();

        let Some(peek) = peek else { return None };
        match peek.kind {
            LexerTokenKind::OutputKeyword => self.parse_output_statement(errors),
            LexerTokenKind::IfKeyword => self.parse_if_statement(errors),
            LexerTokenKind::ReturnKeyword => self.parse_return_statement(errors),
            LexerTokenKind::FunctionKeyword => self.parse_function_declaration(errors),
            _ => self.parse_expression(errors),
        }
    }

    fn parse_scope(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let mut parsed: Vec<SyntaxToken> = Vec::new();
        loop {
            let statement = self.parse_statement(errors);
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

pub fn parse(tokens: Vec<LexerToken>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let iter = tokens.iter().peekable();
    let mut parser = Parser::new(iter);

    parser.parse_scope(errors)
}
