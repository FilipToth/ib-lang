use pest::{
    iterators::{Pair, Pairs},
    pratt_parser::PrattParser,
    Parser,
};

use super::error_bag::{ErrorBag, ErrorKind};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IbParser;

#[derive(Debug)]
pub enum SyntaxToken {
    Module {
        children: Box<Vec<SyntaxToken>>,
    },
    IfStatement {
        condition: Box<SyntaxToken>,
        next: Box<SyntaxToken>,
    },
    OutputStatement {
        expr: Box<SyntaxToken>,
    },
    AssignmentExpression {
        reference: Box<SyntaxToken>,
        value: Box<SyntaxToken>,
    },
    ReferenceExpression(String),
    BinaryExpression {
        lhs: Box<SyntaxToken>,
        op: Operator,
        rhs: Box<SyntaxToken>,
    },
    UnaryExpression {
        op: Operator,
        rhs: Box<SyntaxToken>,
    },
    LiteralExpression(Box<SyntaxToken>),
    IdentifierToken(String),
    NumberToken(i32),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Not,
    Equality,
}

fn parse_module(module: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let subtokens = module.into_inner();

    let mut tokens: Vec<SyntaxToken> = Vec::new();
    for subtoken in subtokens {
        let parsed = match parse(Pairs::single(subtoken), errors) {
            Some(t) => t,
            None => return None
        };

        tokens.push(parsed);
    }

    let module = SyntaxToken::Module {
        children: Box::new(tokens),
    };

    Some(module)
}

fn parse_if_statement(statement: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = statement.into_inner();

    let condition = match subtokens.nth(0) {
        Some(c) => parse(Pairs::single(c), errors),
        None => unreachable!("Cannot find condition in if statement"),
    };

    let condition = match condition {
        Some(t) => t,
        None => return None
    };

    let next = match subtokens.nth(0) {
        Some(n) => parse(Pairs::single(n), errors),
        None => unreachable!("Cannot find next block in if statement"),
    };

    let next = match next {
        Some(t) => t,
        None => return None
    };

    let if_statement = SyntaxToken::IfStatement {
        condition: Box::new(condition),
        next: Box::new(next),
    };

    Some(if_statement)
}

fn parse_output_statement(statement: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = statement.into_inner();

    let expr = match subtokens.nth(0) {
        Some(e) => parse(Pairs::single(e), errors),
        None => unreachable!("Cannot find expression in output statement"),
    };

    let expr = match expr {
        Some(t) => t,
        None => return None
    };

    let output_statement = SyntaxToken::OutputStatement {
        expr: Box::new(expr),
    };

    Some(output_statement)
}

fn parse_assignment_expression(expr: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = expr.into_inner();
    let reference = match subtokens.nth(0) {
        Some(i) => parse_reference_expression(i, errors),
        None => unreachable!("Cannot find identifier in assignment expression"),
    };

    let reference = match reference {
        Some(t) => t,
        None => return None
    };

    let _assignment = subtokens.nth(0);

    let value = match subtokens.nth(0) {
        Some(v) => parse(Pairs::single(v), errors),
        None => unreachable!("Cannot find value expression in assignment expression"),
    };

    let value = match value {
        Some(t) => t,
        None => return None
    };

    let assignment = SyntaxToken::AssignmentExpression {
        reference: Box::new(reference),
        value: Box::new(value),
    };

    Some(assignment)
}

fn parse_reference_expression(reference: Pair<Rule>, _errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let identifier = match reference.into_inner().nth(0) {
        Some(i) => String::from(i.as_str()),
        None => unreachable!("Cannot find identifier when parsing reference expression"),
    };

    let reference = SyntaxToken::ReferenceExpression(identifier);
    Some(reference)
}

fn parse_identifier_token(identifier: Pair<Rule>, _errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let identifier_token = SyntaxToken::IdentifierToken(String::from(identifier.as_str()));
    Some(identifier_token)
}

fn parse_literal_expression(literal: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let inner = match literal.into_inner().nth(0) {
        Some(i) => parse(Pairs::single(i), errors),
        None => unreachable!("Cannot find literal expression inner token"),
    };

    let inner = match inner {
        Some(t) => t,
        None => return None
    };

    let literal_expr = SyntaxToken::LiteralExpression(Box::new(inner));
    Some(literal_expr)
}

fn parse_number_token(pairs: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let num = match pairs.as_str().parse::<i32>() {
        Ok(n) => n,
        Err(_) => {
            let (line, col) = pairs.line_col();
            errors.add(ErrorKind::NumberParsing, line, col);
            return None;
        }
    };

    let number_token = SyntaxToken::NumberToken(num);
    Some(number_token)
}

fn parse(pairs: Pairs<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::module => parse_module(primary, errors),
            Rule::expression_statement => parse(primary.into_inner(), errors),
            Rule::if_statement => parse_if_statement(primary, errors),
            Rule::output_statement => parse_output_statement(primary, errors),
            Rule::expression => parse(primary.into_inner(), errors),
            Rule::assignment_expression => parse_assignment_expression(primary, errors),
            Rule::reference_expression => parse_reference_expression(primary, errors),
            Rule::literal_expression => parse_literal_expression(primary, errors),
            Rule::number_token => parse_number_token(primary, errors),
            Rule::identifier_token => parse_identifier_token(primary, errors),
            rule => unreachable!("Unexpected parser rule type: {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let operator = match op.as_rule() {
                Rule::addition => Operator::Addition,
                Rule::subtraction => Operator::Subtraction,
                Rule::multiplication => Operator::Multiplication,
                Rule::division => Operator::Division,
                Rule::equality => Operator::Equality,
                _ => unreachable!(),
            };

            let lhs = match lhs {
                Some(t) => t,
                None => return None
            };

            let rhs = match rhs {
                Some(t) => t,
                None => return None
            };

            let expr = SyntaxToken::BinaryExpression {
                lhs: Box::new(lhs),
                op: operator,
                rhs: Box::new(rhs),
            };

            Some(expr)
        })
        .map_prefix(|op, rhs| {
            let operator = match op.as_rule() {
                Rule::not => Operator::Not,
                _ => unreachable!(),
            };

            let rhs = match rhs {
                Some(t) => t,
                None => return None
            };

            let expr = SyntaxToken::UnaryExpression {
                op: operator,
                rhs: Box::new(rhs),
            };

            Some(expr)
        })
        .parse(pairs)
}

lazy_static! {
    static ref PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Op, Assoc::*};
        use analysis::parser::Rule::*;

        // instantiate pratt parser and define
        // operator precedences
        PrattParser::new()
            .op(Op::infix(equality, Left))
            .op(Op::infix(addition, Left) | Op::infix(subtraction, Left))
            .op(Op::infix(multiplication, Left) | Op::infix(division, Left))
            .op(Op::prefix(not))
    };
}

pub fn parse_contents(contents: String, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    match IbParser::parse(Rule::module, contents.as_str()) {
        Ok(mut pairs) => parse(Pairs::single(pairs.next().unwrap()), errors),
        Err(_) => None,
    }
}
