use std::{iter::Peekable, slice::Iter};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    span::{Location, Span},
};

use super::{
    lexer::{LexerToken, LexerTokenKind},
    syntax_token::{SyntaxKind, SyntaxToken, TypeAnnotation},
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
        let unary_precedence = self.tokens.peek()?.kind.unary_operator_precedence();

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

            let rhs = self.parse_binary_expression(unary_precedence, errors)?;

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
                    LexerTokenKind::StringLiteralToken(val) => {
                        self.tokens.next();

                        let kind = SyntaxKind::StringLiteralExpression(val.clone());
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
                    LexerTokenKind::BadToken(character) => {
                        let error_kind = ErrorKind::InvalidCharacter(*character);
                        errors.add(error_kind, span);

                        // consume it so the same character is not reported
                        // again by the enclosing construct
                        self.tokens.next();
                        return None;
                    }
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

        let peek = match self.tokens.peek() {
            Some(p) => p,
            None => {
                let kind = SyntaxKind::ReferenceExpression(identifier);
                let token = SyntaxToken::new(kind, identifier_span);
                return Some(token);
            }
        };

        let base = match peek.kind {
            LexerTokenKind::OpenParenthesisToken => {
                self.parse_call_expression(identifier, identifier_span, errors)?
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
                return Some(token);
            }
            _ => {
                let kind = SyntaxKind::ReferenceExpression(identifier);
                SyntaxToken::new(kind, identifier_span)
            }
        };

        self.parse_access_chain(base, errors)
    }

    fn parse_call_expression(
        &mut self,
        identifier: String,
        identifier_span: Span,
        errors: &mut ErrorBag,
    ) -> Option<SyntaxToken> {
        let arguments = self.parse_argument_list(errors)?;

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

    /// Applies member accesses and indexes left to right, so `A[0].pop()`
    /// indexes `A` and then calls `pop` on the element.
    fn parse_access_chain(
        &mut self,
        base: SyntaxToken,
        errors: &mut ErrorBag,
    ) -> Option<SyntaxToken> {
        let mut expr = base;

        loop {
            let peek = match self.tokens.peek() {
                Some(p) => p,
                None => return Some(expr),
            };

            expr = match peek.kind {
                LexerTokenKind::DotToken => self.parse_member_access(expr, errors)?,
                LexerTokenKind::OpenSquareBracketToken => {
                    let indexed = self.parse_index_expression(expr, errors)?;

                    // an assignment is a whole expression, nothing chains off it
                    if let SyntaxKind::IndexAssignmentExpression { .. } = indexed.kind {
                        return Some(indexed);
                    }

                    indexed
                }
                _ => return Some(expr),
            };
        }
    }

    fn parse_member_access(
        &mut self,
        base: SyntaxToken,
        errors: &mut ErrorBag,
    ) -> Option<SyntaxToken> {
        let dot = self.tokens.next().unwrap();
        let (member, member_span) = match self.parse_identifier() {
            Some(m) => m,
            None => {
                let error_kind = ErrorKind::ExpectedToken("object member".to_string());
                errors.add(error_kind, dot.span);
                return None;
            }
        };

        let next = if self.expect_next_token_peek(LexerTokenKind::OpenParenthesisToken) {
            self.parse_call_expression(member, member_span, errors)?
        } else {
            let kind = SyntaxKind::ReferenceExpression(member);
            SyntaxToken::new(kind, member_span)
        };

        let span = Span::from_loc(base.span.start, next.span.end);
        let kind = SyntaxKind::ObjectMemberExpression {
            base: Box::new(base),
            next: Box::new(next),
        };

        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    /// Parses `[index]` after `base`, along with the `= value` that makes it an
    /// index assignment when one follows.
    fn parse_index_expression(
        &mut self,
        base: SyntaxToken,
        errors: &mut ErrorBag,
    ) -> Option<SyntaxToken> {
        let open_bracket = self.tokens.next().unwrap();
        let index = match self.parse_expression(errors) {
            Some(i) => i,
            None => {
                let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                errors.add(error_kind, open_bracket.span);
                return None;
            }
        };

        let close_bracket = match self.tokens.next() {
            Some(t) if t.kind == LexerTokenKind::CloseSquareBracketToken => t,
            _ => {
                let error_kind = ErrorKind::ExpectedToken("close square bracket ']'".to_string());
                errors.add(error_kind, index.span);
                return None;
            }
        };

        if !self.expect_next_token_peek(LexerTokenKind::EqualsToken) {
            let span = Span::from_loc(base.span.start, close_bracket.span.end);
            let kind = SyntaxKind::IndexExpression {
                base: Box::new(base),
                index: Box::new(index),
            };

            let token = SyntaxToken::new(kind, span);
            return Some(token);
        }

        // index assignment expression
        let equals = self.tokens.next().unwrap();
        let value = match self.parse_expression(errors) {
            Some(v) => v,
            None => {
                let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                errors.add(error_kind, equals.span);
                return None;
            }
        };

        let span = Span::from_loc(base.span.start, value.span.end);
        let kind = SyntaxKind::IndexAssignmentExpression {
            base: Box::new(base),
            index: Box::new(index),
            value: Box::new(value),
        };

        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_instantiation_expression(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let new_keyword = self.tokens.next().unwrap();
        let type_annotation = self.parse_type_annotation(new_keyword.span, errors)?;

        if !self.expect_next_token_peek(LexerTokenKind::OpenParenthesisToken) {
            let error_kind = ErrorKind::ExpectedToken("Argument List".to_string());
            errors.add(error_kind, type_annotation.span);
            return None;
        }

        let arg_list = match self.parse_argument_list(errors) {
            Some(a) => a,
            None => {
                let error_kind = ErrorKind::ExpectedToken("Argument List".to_string());
                errors.add(error_kind, type_annotation.span);
                return None;
            }
        };

        let end_loc = match arg_list.last() {
            Some(l) => l.span.end,
            None => type_annotation.span.end,
        };

        let span = Span::from_loc(new_keyword.span.start, end_loc);
        let kind = SyntaxKind::InstantiationExpression {
            type_annotation: type_annotation,
            args: arg_list,
        };

        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    /// Parses a type: a name, optionally followed by a generic parameter in
    /// angle brackets. The parameter is itself a type, so `Array<Stack<Int>>`
    /// nests. `preceding` is where to report a missing name.
    fn parse_type_annotation(
        &mut self,
        preceding: Span,
        errors: &mut ErrorBag,
    ) -> Option<TypeAnnotation> {
        let (name, name_span) = match self.parse_identifier() {
            Some(i) => i,
            None => {
                let error_kind = ErrorKind::ExpectedToken("type".to_string());
                errors.add(error_kind, preceding);
                return None;
            }
        };

        if !self.expect_next_token_peek(LexerTokenKind::LesserThanToken) {
            let annotation = TypeAnnotation {
                name: name,
                generic: None,
                span: name_span,
            };

            return Some(annotation);
        }

        let open_angle = self.tokens.next().unwrap();
        let generic = self.parse_type_annotation(open_angle.span, errors)?;

        let close_angle = match self.tokens.next() {
            Some(t) if t.kind == LexerTokenKind::GreaterThanToken => t,
            _ => {
                let error_kind = ErrorKind::ExpectedToken("close angle bracket '>'".to_string());
                errors.add(error_kind, generic.span);
                return None;
            }
        };

        let annotation = TypeAnnotation {
            name: name,
            generic: Some(Box::new(generic)),
            span: Span::from_loc(name_span.start, close_angle.span.end),
        };

        Some(annotation)
    }

    fn parse_argument_list(&mut self, errors: &mut ErrorBag) -> Option<Vec<SyntaxToken>> {
        let _open_paren = self.tokens.next().unwrap();

        let mut args: Vec<SyntaxToken> = Vec::new();
        let mut prev_comma = false;

        loop {
            let peek = self.tokens.peek()?;

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

            let expr = self.parse_expression(errors)?;

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
            LexerTokenKind::NotKeyword => Some(Operator::Not),
            LexerTokenKind::BangEqualsToken => Some(Operator::Inequality),
            LexerTokenKind::ModKeyword => Some(Operator::Modulo),
            LexerTokenKind::DivKeyword => Some(Operator::IntDivision),
            LexerTokenKind::AndKeyword => Some(Operator::And),
            LexerTokenKind::OrKeyword => Some(Operator::Or),
            LexerTokenKind::LesserThanToken => Some(Operator::LesserThan),
            LexerTokenKind::GreaterThanToken => Some(Operator::GreaterThan),
            LexerTokenKind::GreaterThanEqualsToken => Some(Operator::GreaterThanEquals),
            LexerTokenKind::LesserThanEqualsToken => Some(Operator::LesserThanEquals),
            _ => None,
        }
    }

    /// The spec writes `output NUM , " = " , F`, so an output statement takes a
    /// comma-separated list of expressions, printed on one line.
    fn parse_output_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let keyword = self.tokens.next().unwrap();
        let start_loc = keyword.span.start.clone();

        let first = match self.parse_expression(errors) {
            Some(e) => e,
            None => {
                let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                errors.add(error_kind, keyword.span);
                return None;
            }
        };

        let mut end_loc = first.span.end.clone();
        let mut exprs: Vec<SyntaxToken> = vec![first];

        while self.expect_next_token_peek(LexerTokenKind::CommaToken) {
            let comma = self.tokens.next().unwrap();

            let expr = match self.parse_expression(errors) {
                Some(e) => e,
                None => {
                    let error_kind = ErrorKind::ExpectedToken("expression".to_string());
                    errors.add(error_kind, comma.span);
                    return None;
                }
            };

            end_loc = expr.span.end.clone();
            exprs.push(expr);
        }

        let kind = SyntaxKind::OutputStatement { exprs: exprs };

        let span = Span::from_loc(start_loc, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
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

        // optional else clause. parse_scope stops at the else token without
        // consuming it, the same way it stops at end.
        let else_body = if self.expect_next_token_peek(LexerTokenKind::ElseKeyword) {
            // consume the else keyword
            self.tokens.next();

            let else_scope = match self.parse_scope(errors) {
                Some(s) => s,
                None => {
                    let error_kind = ErrorKind::ExpectedScope;
                    errors.add(error_kind, body.span);
                    return None;
                }
            };

            Some(Box::new(else_scope))
        } else {
            None
        };

        // end keyword
        if !self.expect_next_token(LexerTokenKind::EndKeyword) {
            let error_kind = ErrorKind::ExpectedToken("end keyword".to_string());
            errors.add(error_kind, body.span);
            return None;
        }

        // the statement runs to the end of whichever clause came last
        let end_loc = match &else_body {
            Some(e) => e.span.end.clone(),
            None => body.span.end.clone(),
        };

        let kind = SyntaxKind::IfStatement {
            condition: Box::new(condition),
            body: Box::new(body),
            else_body: else_body,
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
        let parameters = self.parse_parameter_list(errors)?;

        let mut return_type: Option<TypeAnnotation> = None;
        if self.expect_next_token_peek(LexerTokenKind::ArrowToken) {
            let arrow = self.tokens.next().unwrap();
            return_type = self.parse_type_annotation(arrow.span, errors);
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
            let peek = self.tokens.peek()?.clone();

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

            let (identifier, loc) = self.parse_identifier()?;

            let type_annotation = if self.expect_next_token_peek(LexerTokenKind::ColonToken) {
                let colon = self.tokens.next().unwrap();
                Some(self.parse_type_annotation(colon.span, errors)?)
            } else {
                None
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
                // the spec writes `loop COUNT from 0 to 5`; `for` is accepted
                // but optional, so either token starts a counted loop
                LexerTokenKind::IdentifierToken(_) => self.parse_for_loop(errors),
                LexerTokenKind::ForKeyword => self.parse_for_loop(errors),
                LexerTokenKind::WhileKeyword => self.parse_while_loop(errors),
                LexerTokenKind::UntilKeyword => self.parse_until_loop(errors),
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
        // `for` is optional: the spec writes `loop COUNT from 0 to 5`, while
        // this language has always accepted `loop for COUNT from 0 to 5`
        let start_span = match self.tokens.peek() {
            Some(p) => p.span.clone(),
            None => {
                let kind = ErrorKind::ExpectedToken("identifier".to_string());
                errors.add(kind, Span::new(0, 0, 0, 0, 0, 0));
                return None;
            }
        };

        if self.expect_next_token_peek(LexerTokenKind::ForKeyword) {
            self.tokens.next();
        }

        let (identifier, identifier_span) = match self.parse_identifier() {
            Some(i) => i,
            None => {
                let kind = ErrorKind::ExpectedToken("identifier".to_string());
                errors.add(kind, start_span);
                return None;
            }
        };

        if !self.expect_next_token(LexerTokenKind::FromKeyword) {
            let kind = ErrorKind::ExpectedToken("from keyword".to_string());
            errors.add(kind, identifier_span);
            return None;
        }

        // bounds are full expressions, so the spec's `from 0 to COUNT-1` works.
        // `to` is a keyword rather than an operator, so it terminates the
        // lower bound's expression on its own.
        let lower_bound = match self.parse_expression(errors) {
            Some(e) => e,
            None => {
                let kind = ErrorKind::ExpectedLoopLowerBound;
                errors.add(kind, identifier_span);
                return None;
            }
        };

        let lower_bound_span = lower_bound.span;

        if !self.expect_next_token(LexerTokenKind::ToKeyword) {
            let kind = ErrorKind::ExpectedToken("to keyword".to_string());
            errors.add(kind, lower_bound_span);
            return None;
        }

        let upper_bound = match self.parse_expression(errors) {
            Some(e) => e,
            None => {
                let kind = ErrorKind::ExpectedLoopUpperBound;
                errors.add(kind, identifier_span);
                return None;
            }
        };

        let upper_bound_span = upper_bound.span;

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
            lower_bound: Box::new(lower_bound),
            upper_bound: Box::new(upper_bound),
            body: Box::new(body),
        };

        let span = Span::from_loc(start_span.start, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_while_loop(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let while_keyword = self.tokens.next().unwrap();

        let expr = self.parse_expression(errors)?;

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

    fn parse_until_loop(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let until_keyword = self.tokens.next().unwrap();

        let expr = self.parse_expression(errors)?;

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
        let kind = SyntaxKind::UntilLoop {
            expr: Box::new(expr),
            body: Box::new(body),
        };

        let span = Span::from_loc(until_keyword.span.start, end_loc);
        let token = SyntaxToken::new(kind, span);
        Some(token)
    }

    fn parse_statement(&mut self, errors: &mut ErrorBag) -> Option<SyntaxToken> {
        let peek = self.tokens.peek()?;
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
            // a scope ends at the end of the file or at a keyword that closes
            // it; the enclosing construct checks it got the one it expected
            let closes_scope = match self.tokens.peek() {
                Some(t) => matches!(t.kind, LexerTokenKind::EndKeyword | LexerTokenKind::ElseKeyword),
                None => true,
            };

            if closes_scope {
                break;
            }

            let error_count = errors.errors.len();
            match self.parse_statement(errors) {
                Some(s) => parsed.push(s),
                None => {
                    // a statement that failed without reporting anything could
                    // not start at all. say so, rather than silently dropping
                    // everything after it.
                    if errors.errors.len() == error_count {
                        if let Some(t) = self.tokens.peek() {
                            errors.add(ErrorKind::UnexpectedToken, t.span);
                        }
                    }

                    break;
                }
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
        let t = self.tokens.next()?;
        match &t.kind {
            LexerTokenKind::IdentifierToken(id) => Some((id.clone(), t.span.clone())),
            _ => None,
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

    let error_count = errors.errors.len();
    let root = parser.parse_scope(errors)?;

    // a nested scope leaves `end` and `else` for its construct to consume. at
    // the top level there is no construct, so one left over is stray.
    if errors.errors.len() == error_count {
        if let Some(t) = parser.tokens.peek() {
            errors.add(ErrorKind::UnexpectedToken, t.span);
        }
    }

    Some(root)
}
