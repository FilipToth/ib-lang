#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate pest_derive;

extern crate pest;

use std::fs;
use pest::{
    pratt_parser::PrattParser,
    Parser,
    iterators::{Pairs, Pair}
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IbParser;

#[derive(Debug)]
pub enum ParseError {
    Failed
}

#[derive(Debug)]
pub enum SyntaxToken {
    Module {
        children: Box<Vec<SyntaxToken>>
    },
    IfStatement {
        condition: Box<SyntaxToken>,
        next: Box<SyntaxToken>
    },
    OutputStatement {
        expr: Box<SyntaxToken>
    },
    AssignmentExpression {
        reference: Box<SyntaxToken>,
        value: Box<SyntaxToken>
    },
    ReferenceExpression(String),
    BinaryExpression {
        lhs: Box<SyntaxToken>,
        op: Operator,
        rhs: Box<SyntaxToken>
    },
    UnaryExpression {
        op: Operator,
        rhs: Box<SyntaxToken>
    },
    LiteralExpression(Box<SyntaxToken>),
    IdentifierToken(String),
    NumberToken(i32)
}

#[derive(Debug, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Not,
    Equality
}

fn parse_module(module: Pair<Rule>) -> SyntaxToken {
    let subtokens = module.into_inner();

    let mut tokens: Vec<SyntaxToken> = Vec::new();
    for subtoken in subtokens {
        let parsed = parse(Pairs::single(subtoken));
        tokens.push(parsed);
    }

    SyntaxToken::Module { children: Box::new(tokens) }
}

fn parse_if_statement(statement: Pair<Rule>) -> SyntaxToken {
    let mut subtokens = statement.into_inner();

    let condition = match subtokens.nth(0) {
        Some(c) => parse(Pairs::single(c)),
        None => unreachable!("Cannot find condition in if statement")
    };

    let next = match subtokens.nth(0) {
        Some(n) => parse(Pairs::single(n)),
        None => unreachable!("Cannot find next block in if statement")
    };

    SyntaxToken::IfStatement {
        condition: Box::new(condition),
        next: Box::new(next)
    }
}

fn parse_output_statement(statement: Pair<Rule>) -> SyntaxToken {
    let mut subtokens = statement.into_inner();

    let expr = match subtokens.nth(0) {
        Some(e) => parse(Pairs::single(e)),
        None => unreachable!("Cannot find expression in output statement")
    };

    SyntaxToken::OutputStatement { expr: Box::new(expr) }
}

fn parse_assignment_expression(expr: Pair<Rule>) -> SyntaxToken {
    let mut subtokens = expr.into_inner();
    let reference = match subtokens.nth(0) {
        Some(i) => parse_reference_expression(i),
        None => unreachable!("Cannot find identifier in assignment expression")
    };

    let _assignment = subtokens.nth(0);

    let value = match subtokens.nth(0) {
        Some(v) => parse(Pairs::single(v)),
        None => unreachable!("Cannot find value expression in assignment expression")
    };

    SyntaxToken::AssignmentExpression { reference: Box::new(reference), value: Box::new(value) }
}

fn parse_reference_expression(reference: Pair<Rule>) -> SyntaxToken {
    let identifier = match reference.into_inner().nth(0) {
        Some(i) => String::from(i.as_str()),
        None => unreachable!("Cannot find identifier when parsing reference expression")
    };

    SyntaxToken::ReferenceExpression(identifier)
}

fn parse_identifier_token(identifier: Pair<Rule>) -> SyntaxToken {
    SyntaxToken::IdentifierToken(String::from(identifier.as_str()))
}

fn parse_literal_expression(literal: Pair<Rule>) -> SyntaxToken {
    let inner = match literal.into_inner().nth(0) {
        Some(i) => parse(Pairs::single(i)),
        None => unreachable!("Cannot find literal expression inner token")
    };

    SyntaxToken::LiteralExpression(Box::new(inner))
}

fn parse_number_token(pairs: Pair<Rule>) -> SyntaxToken {
    SyntaxToken::NumberToken(pairs.as_str().parse::<i32>().unwrap())
}

fn parse(pairs: Pairs<Rule>) -> SyntaxToken {
    PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::module => parse_module(primary),
            Rule::expression_statement => parse(primary.into_inner()),
            Rule::if_statement => parse_if_statement(primary),
            Rule::output_statement => parse_output_statement(primary),
            Rule::expression => parse(primary.into_inner()),
            Rule::assignment_expression => parse_assignment_expression(primary),
            Rule::reference_expression => parse_reference_expression(primary),
            Rule::literal_expression => parse_literal_expression(primary),
            Rule::number_token => parse_number_token(primary),
            Rule::identifier_token => parse_identifier_token(primary),
            rule => unreachable!("Unexpected parser rule type: {:?}", rule)
        })
        .map_infix(|lhs, op, rhs| {
            let operator = match op.as_rule() {
                Rule::addition => Operator::Addition,
                Rule::subtraction => Operator::Subtraction,
                Rule::multiplication => Operator::Multiplication,
                Rule::division => Operator::Division,
                Rule::equality => Operator::Equality,
                _ => unreachable!()
            };

            SyntaxToken::BinaryExpression {
                lhs: Box::new(lhs),
                op: operator,
                rhs: Box::new(rhs)
            }
        })
        .map_prefix(|op, rhs| {
            let operator = match op.as_rule() {
                Rule::not => Operator::Not,
                _ => unreachable!()
            };

            SyntaxToken::UnaryExpression {
                op: operator,
                rhs: Box::new(rhs)
            }
        })
        .parse(pairs)
}

lazy_static! {
    static ref PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Op, Assoc::*};
        use Rule::*;

        // instantiate pratt parser and define
        // operator precedences
        PrattParser::new()
            .op(Op::infix(equality, Left))
            .op(Op::infix(addition, Left) | Op::infix(subtraction, Left))
            .op(Op::infix(multiplication, Left) | Op::infix(division, Left))
            .op(Op::prefix(not))
    };
}

fn parse_file() -> Result<SyntaxToken, ParseError> {
    let contents = fs::read_to_string("test.ib").unwrap();
    println!("{}", contents.as_str().clone());
    match IbParser::parse(Rule::module, contents.as_str()) {
        Ok(mut pairs) => {
            println!("{:#?}", pairs.clone());
            let token = parse(Pairs::single(pairs.next().unwrap()));
            return Ok(token);
        },
        Err(_) => return Err(ParseError::Failed)
    }
}

fn main() {
    let root = parse_file().unwrap();
    println!("{:?}", root);
}
