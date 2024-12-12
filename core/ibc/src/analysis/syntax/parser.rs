use std::{iter::Peekable, slice::Iter};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    span::{Location, Span},
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
            let next_span = next.span.clone();

            let operator = match self.parse_operator(next) {
                Some(o) => o,
                None => {
                    let error_kind = ErrorKind::UnknownOperator(next.kind.to_string());
                    errors.add(error_kind, next_span);
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

            let token = SyntaxToken::new(unary_kind, next_span);
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
                    errors.add(error_kind, lhs_token.span);
                    break;
                }
            };

            let operator = match self.parse_operator(operator_token) {
                Some(o) => o,
                None => {
                    let error_kind = ErrorKind::UnknownOperator(operator_token.kind.to_string());
                    errors.add(error_kind, operator_token.span);
                    return None;
                }
            };

            let rhs = match self.parse_binary_expression(precedence, errors) {
                Some(r) => r,
                None => {
                    let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                    errors.add(error_kind, operator_token.span);
                    return None;
                }
            };

            let start_loc = lhs_token.span.start.clone();
            let end_loc = rhs.span.end.clone();
            let span = Span::from_loc(start_loc, end_loc);

            let bin_expr_kind = SyntaxKind::BinaryExpression {
                lhs: Box::new(lhs_token),
                op: operator,
                rhs: Box::new(rhs),
            };

            let bin_expr = SyntaxToken::new(bin_expr_kind, span);
            lhs = Some(bin_expr);
        }

        lhs
    }

    fn parse_primary_expression(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        match self.tokens.peek() {
            Some(p) => {
                // check if kind is valid
                let span = p.span.clone();
                match &p.kind {
                    LexerTokenKind::OpenParenthesisToken => {
                        self.parse_parenthesis_expression(errors)
                    }
                    LexerTokenKind::IdentifierToken(_) => self.parse_reference_based_token(errors),
                    LexerTokenKind::IntegerLiteralToken(val) => {
                        // consume token
                        self.tokens.next();

                        let kind = SyntaxKind::IntegerLiteralExpression(val.clone());
                        let token = SyntaxToken::new(kind, span);
                        Some(token)
                    }
                    LexerTokenKind::TrueKeyword => {
                        // consume token
                        self.tokens.next();

                        let kind = SyntaxKind::BooleanLiteralExpression(true);
                        let token = SyntaxToken::new(kind, span);
                        Some(token)
                    }
                    LexerTokenKind::FalseKeyword => {
                        // consume token
                        self.tokens.next();

                        let kind = SyntaxKind::BooleanLiteralExpression(false);
                        let token = SyntaxToken::new(kind, span);
                        Some(token)
                    }
                    LexerTokenKind::NewKeyword => self.parse_instantiation_expression(errors),
                    _ => return None,
                }
            }
            None => {
                // TODO: Add location, use some sort of last parsed token location
                let error_kind = ErrorKind::ExpectedPrimaryExpression;
                errors.add(error_kind, Span::new(0, 0, 0, 0, 0, 0));
                return None;
            }
        }
    }

    fn parse_reference_based_token(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        // theoretically shouldn't be none
        let (identifier, identifier_span) = match self.parse_identifier() {
            Some(i) => i,
            None => {
                // TODO: Agains use some sort of last token loc
                let error_kind = ErrorKind::ExpectedToken("identifier".to_string());
                errors.add(error_kind, Span::new(0, 0, 0, 0, 0, 0));
                return None;
            }
        };

        let reference_kind = SyntaxKind::ReferenceExpression(identifier.clone());
        let reference = SyntaxToken::new(reference_kind, identifier_span);

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

                let last_loc = match arguments.last() {
                    Some(t) => t.span.end,
                    None => identifier_span.end,
                };

                let kind = SyntaxKind::CallExpression {
                    identifier: identifier,
                    args: arguments,
                };

                let span = Span::from_loc(identifier_span.start, last_loc);
                let token = SyntaxToken::new(kind, span);
                Some(token)
            }
            LexerTokenKind::EqualsToken => {
                // assignment expression
                let equals = self.tokens.next().unwrap();
                let value = match self.parse_expression(errors) {
                    Some(e) => e,
                    None => {
                        let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                        errors.add(error_kind, equals.span);
                        return None;
                    }
                };

                let end_loc = value.span.end;
                let kind = SyntaxKind::AssignmentExpression {
                    identifier: identifier,
                    value: Box::new(value),
                };

                let span = Span::from_loc(identifier_span.start, end_loc);
                let token = SyntaxToken::new(kind, span);
                Some(token)
            }
            LexerTokenKind::DotToken => {
                // object call expression
                let dot = self.tokens.next().unwrap();
                let next_expr = match self.parse_reference_based_token(errors) {
                    Some(n) => n,
                    None => {
                        let error_kind = ErrorKind::ExpectedToken("object member".to_string());
                        errors.add(error_kind, dot.span);
                        return None;
                    }
                };

                let span = Span::from_loc(reference.span.start, next_expr.span.end);
                let kind = SyntaxKind::ObjectMemberExpression {
                    base: Box::new(reference),
                    next: Box::new(next_expr),
                };

                let token = SyntaxToken::new(kind, span);
                Some(token)
            }
            _ => Some(reference),
        }
    }

    fn parse_instantiation_expression(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let new_keyword = self.tokens.next().unwrap();

        let (identifier, identifier_span) = match self.parse_identifier() {
            Some((i, s)) => (i, s),
            None => {
                let error_kind = ErrorKind::ExpectedToken("Identifier".to_string());
                errors.add(error_kind, new_keyword.span);
                return None;
            }
        };

        let peek = match self.tokens.peek() {
            Some(i) => i,
            None => {
                let error_kind = ErrorKind::ExpectedToken("Argument List".to_string());
                errors.add(error_kind, identifier_span);
                return None;
            }
        };

        let type_param = match peek.kind {
            LexerTokenKind::LesserThanToken => {
                let le_token = self.tokens.next().unwrap();
                let (identifier, _) = match self.parse_identifier() {
                    Some(i) => i,
                    None => {
                        let error_kind = ErrorKind::ExpectedToken("Type Identifier".to_string());
                        errors.add(error_kind, le_token.span);
                        return None;
                    }
                };

                // consume ge token
                self.expect_next_token(LexerTokenKind::GreaterThanToken);

                Some(identifier)
            }
            _ => None,
        };

        let arg_list = match self.parse_argument_list(errors) {
            Some(a) => a,
            None => {
                let error_kind = ErrorKind::ExpectedToken("Argument List".to_string());
                errors.add(error_kind, identifier_span);
                return None;
            }
        };

        let end_loc = match arg_list.last() {
            Some(l) => l.span.end,
            None => identifier_span.end,
        };

        let kind = SyntaxKind::InstantiationExpression {
            type_name: identifier,
            type_param: type_param,
            args: arg_list,
        };

        let span = Span::from_loc(new_keyword.span.start, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_argument_list(&mut self, errors: &mut ErrorBag) -> Option<Vec<SyntaxToken>> {
        let _open_paren = self.tokens.next().unwrap();

        let mut args: Vec<SyntaxToken> = Vec::new();
        let mut prev_comma = false;

        loop {
            let peek = match self.tokens.peek() {
                Some(p) => p,
                None => return None,
            };

            if let LexerTokenKind::CloseParenthesisToken = peek.kind {
                let peek_span = peek.span.clone();
                self.tokens.next();

                if prev_comma {
                    let error_kind = ErrorKind::ExpectedArgument;
                    errors.add(error_kind, peek_span);
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
                    errors.add(error_kind, expr.span);
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
                    errors.add(error_kind, peek.span);
                    return None;
                }
            };

            args.push(expr);
        }

        Some(args)
    }

    fn parse_parenthesis_expression(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let left_paren = self.tokens.next().unwrap();
        let start_loc = left_paren.span.start;

        match self.parse_expression(errors) {
            Some(expr) => {
                let right_paren = self.tokens.next();
                match right_paren {
                    Some(r) => {
                        let kind = SyntaxKind::ParenthesizedExpression {
                            inner: Box::new(expr),
                        };

                        let span = Span::from_loc(start_loc, r.span.end);
                        let token = SyntaxToken::new(kind, span);
                        Some(token)
                    }
                    None => {
                        let error_kind = ErrorKind::UnclosedParenthesisExpression;
                        errors.add(error_kind, expr.span);
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
            LexerTokenKind::LesserThanToken => Some(Operator::LesserThan),
            LexerTokenKind::GreaterThanToken => Some(Operator::GreaterThan),
            _ => None,
        }
    }

    fn parse_output_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let start_loc = keyword.span.start.clone();

        let expr = self.parse_expression(errors);
        match expr {
            Some(expr) => {
                let end_loc = expr.span.end.clone();
                let kind = SyntaxKind::OutputStatement {
                    expr: Box::new(expr),
                };

                let span = Span::from_loc(start_loc, end_loc);
                let token = SyntaxToken::new(kind, span);
                Some(token)
            }
            None => {
                let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                errors.add(error_kind, keyword.span);
                return None;
            }
        }
    }

    fn parse_if_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let start_loc = keyword.span.start.clone();

        let condition = match self.parse_expression(errors) {
            Some(c) => c,
            None => {
                let error_kind = ErrorKind::ExpectedToken("condition expression".to_string());
                errors.add(error_kind, keyword.span);
                return None;
            }
        };

        // then keyword
        if !self.expect_next_token(LexerTokenKind::ThenKeyword) {
            let error_kind = ErrorKind::ExpectedToken("then keyword".to_string());
            errors.add(error_kind, condition.span);
            return None;
        }

        let body = match self.parse_scope(errors) {
            Some(b) => b,
            None => {
                let error_kind = ErrorKind::ExpectedScope;
                errors.add(error_kind, condition.span);
                return None;
            }
        };

        // end keyword
        if !self.expect_next_token(LexerTokenKind::EndKeyword) {
            let error_kind = ErrorKind::ExpectedToken("end keyword".to_string());
            errors.add(error_kind, body.span);
            return None;
        }

        let end_loc = body.span.end.clone();
        let kind = SyntaxKind::IfStatement {
            condition: Box::new(condition),
            body: Box::new(body),
        };

        let span = Span::from_loc(start_loc, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_return_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let (expr, end_loc) = match self.parse_expression(errors) {
            Some(e) => {
                let end_loc = e.span.end.clone();
                (Some(Box::new(e)), end_loc)
            }
            None => (None, keyword.span.end),
        };

        let span = Span::from_loc(keyword.span.start, end_loc);
        let kind = SyntaxKind::ReturnStatement { expr: expr };

        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_function_declaration(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let start_loc = keyword.span.start.clone();

        // identifier
        let identifier = match self.parse_identifier() {
            Some((i, _)) => i,
            None => {
                let error_kind = ErrorKind::ExpectedToken("identifier".to_string());
                errors.add(error_kind, keyword.span);
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
                    errors.add(error_kind, arrow.span);
                }
            };
        }

        // body
        let body = match self.parse_scope(errors) {
            Some(b) => b,
            None => {
                let error_kind = ErrorKind::ExpectedToken("function body".to_string());
                errors.add(error_kind, keyword.span);
                return None;
            }
        };

        // end keyword
        match self.tokens.next() {
            Some(t) => match &t.kind {
                LexerTokenKind::EndKeyword => {}
                _ => {
                    let error_kind = ErrorKind::ExpectedToken("end".to_string());
                    errors.add(error_kind, keyword.span);
                    return None;
                }
            },
            None => {
                let error_kind = ErrorKind::ExpectedToken("end".to_string());
                errors.add(error_kind, keyword.span);
                return None;
            }
        };

        let end_loc = body.span.end.clone();
        let kind = SyntaxKind::FunctionDeclaration {
            identifier: identifier,
            parameters: parameters,
            return_type: return_type,
            body: Box::new(body),
        };

        let span = Span::from_loc(start_loc, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_parameter_list(&mut self, errors: &mut ErrorBag) -> Option<Vec<SyntaxToken>> {
        if !self.expect_next_token(LexerTokenKind::OpenParenthesisToken) {
            // TODO: Use last token span to report error
            let error_kind = ErrorKind::ExpectedToken("open parenthesis '('".to_string());

            let span = Span::new(0, 0, 0, 0, 0, 0);
            errors.add(error_kind, span);
            return None;
        }

        let mut params: Vec<SyntaxToken> = Vec::new();
        let mut prev_comma = false;

        loop {
            let peek = match self.tokens.peek() {
                Some(t) => t.clone(),
                None => return None,
            };

            match peek.kind {
                LexerTokenKind::CloseParenthesisToken => {
                    if prev_comma {
                        // comma, but no further
                        // params provided
                        let error_kind = ErrorKind::ExpectedParameter;
                        errors.add(error_kind, peek.span);
                        return None;
                    }

                    self.tokens.next();
                    break;
                }
                LexerTokenKind::IdentifierToken(_) => {}
                _ => {
                    // expected identifier
                    let error_kind = ErrorKind::ExpectedToken("identifier".to_string());
                    errors.add(error_kind, peek.span);
                    return None;
                }
            };

            let (identifier, loc) = match self.parse_identifier() {
                Some(i) => i,
                None => return None,
            };

            if !self.expect_next_token(LexerTokenKind::ColonToken) {
                let error_kind = ErrorKind::ExpectedToken("type annotation".to_string());
                errors.add(error_kind, peek.span);
                return None;
            }

            let type_annotation = match self.parse_identifier() {
                Some((i, _)) => i,
                None => {
                    let error_kind = ErrorKind::ExpectedToken("type annotation".to_string());
                    errors.add(error_kind, peek.span);
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
                        // expected commas
                        let error_kind = ErrorKind::ExpectedToken(
                            "comma ',' or close parenthesis ')'".to_string(),
                        );

                        errors.add(error_kind, t.span);
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

    fn parse_loop(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let loop_token = self.tokens.next().unwrap();
        match self.tokens.peek() {
            Some(p) => match p.kind {
                LexerTokenKind::IdentifierToken(_) => self.parse_for_loop(errors),
                LexerTokenKind::WhileKeyword => self.parse_while_loop(errors),
                _ => {
                    let kind = ErrorKind::ExpectedLoop;
                    errors.add(kind, loop_token.span);
                    return None;
                }
            },
            None => {
                let kind = ErrorKind::ExpectedLoop;
                errors.add(kind, loop_token.span);
                return None;
            }
        }
    }

    fn parse_for_loop(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let (identifier, identifier_span) = self.parse_identifier().unwrap();

        if !self.expect_next_token(LexerTokenKind::FromKeyword) {
            return None;
        }

        let lower_bound = match self.tokens.next() {
            Some(t) => match t.kind {
                LexerTokenKind::IntegerLiteralToken(val) => val,
                _ => {
                    let kind = ErrorKind::ExpectedLoopLowerBound;
                    errors.add(kind, identifier_span);
                    return None;
                }
            },
            None => {
                let kind = ErrorKind::ExpectedLoopLowerBound;
                errors.add(kind, identifier_span);
                return None;
            }
        };

        if !self.expect_next_token(LexerTokenKind::ToKeyword) {
            return None;
        }

        let (upper_bound, upper_bound_span) = match self.tokens.next() {
            Some(t) => {
                let span = t.span.clone();
                match t.kind {
                    LexerTokenKind::IntegerLiteralToken(val) => (val, span),
                    _ => {
                        let kind = ErrorKind::ExpectedLoopUpperBound;
                        errors.add(kind, identifier_span);
                        return None;
                    }
                }
            }
            None => {
                let kind = ErrorKind::ExpectedLoopUpperBound;
                errors.add(kind, identifier_span);
                return None;
            }
        };

        let body = match self.parse_scope(errors) {
            Some(b) => b,
            None => {
                let error_kind = ErrorKind::ExpectedScope;
                errors.add(error_kind, upper_bound_span);
                return None;
            }
        };

        if !self.expect_next_token(LexerTokenKind::EndKeyword) {
            let error_kind = ErrorKind::ExpectedToken("end keyword".to_string());
            errors.add(error_kind, body.span);
            return None;
        }

        let end_loc = body.span.end.clone();
        let kind = SyntaxKind::ForLoop {
            identifier: identifier,
            lower_bound: lower_bound as usize,
            upper_bound: upper_bound as usize,
            body: Box::new(body),
        };

        let span = Span::from_loc(identifier_span.start, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_while_loop(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let while_keyword = self.tokens.next().unwrap();

        let expr = match self.parse_expression(errors) {
            Some(e) => e,
            None => return None,
        };

        let body = match self.parse_scope(errors) {
            Some(b) => b,
            None => {
                let error_kind = ErrorKind::ExpectedScope;
                errors.add(error_kind, expr.span);
                return None;
            }
        };

        if !self.expect_next_token(LexerTokenKind::EndKeyword) {
            let error_kind = ErrorKind::ExpectedToken("end keyword".to_string());
            errors.add(error_kind, body.span);
            return None;
        }

        let end_loc = body.span.end.clone();
        let kind = SyntaxKind::WhileLoop {
            expr: Box::new(expr),
            body: Box::new(body),
        };

        let span = Span::from_loc(while_keyword.span.start, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let peek = self.tokens.peek();

        let Some(peek) = peek else { return None };
        match peek.kind {
            LexerTokenKind::OutputKeyword => self.parse_output_statement(errors),
            LexerTokenKind::IfKeyword => self.parse_if_statement(errors),
            LexerTokenKind::ReturnKeyword => self.parse_return_statement(errors),
            LexerTokenKind::FunctionKeyword => self.parse_function_declaration(errors),
            LexerTokenKind::LoopKeyword => self.parse_loop(errors),
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

        let first_loc = match parsed.first() {
            Some(f) => f.span.start.clone(),
            None => Location::new(0, 0, 0),
        };

        let last_loc = match parsed.last() {
            Some(l) => l.span.end.clone(),
            None => Location::new(0, 0, 0),
        };

        let span = Span::from_loc(first_loc, last_loc);

        let scope_kind = SyntaxKind::Scope { subtokens: parsed };
        let token = SyntaxToken::new(scope_kind, span);
        Some(token)
    }

    fn parse_identifier(&mut self) -> Option<(String, Span)> {
        match self.tokens.next() {
            Some(t) => match &t.kind {
                LexerTokenKind::IdentifierToken(id) => Some((id.clone(), t.span.clone())),
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
