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
    BinaryExpression {
        lhs: Box<SyntaxToken>,
        op: Operator,
        rhs: Box<SyntaxToken>
    },
    IdentifierToken(String),
    NumberToken(i32)
}

#[derive(Debug, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication
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

fn parse_identifier_token(identifier: Pair<Rule>) -> SyntaxToken {
    SyntaxToken::IdentifierToken(String::from(identifier.as_str()))
}

fn parse_number_token(pairs: Pair<Rule>) -> SyntaxToken {
    SyntaxToken::NumberToken(pairs.as_str().parse::<i32>().unwrap())
}

fn parse(pairs: Pairs<Rule>) -> SyntaxToken {
    PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::module => parse_module(primary),
            Rule::expression => parse(primary.into_inner()),
            Rule::identifier_token => parse_identifier_token(primary),
            Rule::number_token => parse_number_token(primary),
            rule => unreachable!("Unexpected parser rule type: {:?}", rule)
        })
        .map_infix(|lhs, op, rhs| {
            let operator = match op.as_rule() {
                Rule::addition => Operator::Addition,
                Rule::subtraction => Operator::Subtraction,
                Rule::multiplication => Operator::Multiplication,
                Rule::division => Operator::Division,
                _ => unreachable!()
            };

            SyntaxToken::BinaryExpression {
                lhs: Box::new(lhs),
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
            .op(Op::infix(addition, Left) | Op::infix(subtraction, Left))
            .op(Op::infix(multiplication, Left) | Op::infix(division, Left))
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
