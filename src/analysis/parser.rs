use pest::{
    iterators::{Pair, Pairs},
    pratt_parser::PrattParser,
    Parser,
};

use super::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    CodeLocation,
};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct IbParser;

#[derive(Debug)]
pub struct SyntaxToken {
    pub kind: SyntaxKind,
    pub loc: CodeLocation,
}

impl SyntaxToken {
    fn new(kind: SyntaxKind, rule: &Pair<Rule>) -> SyntaxToken {
        let location = CodeLocation::from_pair(rule);
        SyntaxToken {
            kind: kind,
            loc: location,
        }
    }
}

#[derive(Debug)]
pub enum SyntaxKind {
    Module {
        block: Box<SyntaxToken>,
    },
    Block {
        children: Box<Vec<SyntaxToken>>,
    },
    IfStatement {
        condition: Box<SyntaxToken>,
        block: Box<SyntaxToken>,
    },
    OutputStatement {
        expr: Box<SyntaxToken>,
    },
    ReturnStatement {
        expr: Box<SyntaxToken>,
    },
    FunctionDeclaration {
        identifier: Box<SyntaxToken>,
        params: Box<SyntaxToken>,
        ret_type: String,
        block: Box<SyntaxToken>,
    },
    FunctionTypeAnnotation(String),
    ParameterList {
        params: Vec<ParameterSyntax>,
    },
    AssignmentExpression {
        reference: Box<SyntaxToken>,
        value: Box<SyntaxToken>,
    },
    CallExpression {
        identifier: Box<SyntaxToken>,
        args: Box<Vec<SyntaxToken>>,
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
    BooleanToken(bool),
}

#[derive(Debug)]
pub struct ParameterSyntax {
    pub identifier: String,
    pub type_annotation: String,
    pub location: CodeLocation,
}

fn parse_module(module: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = module.clone().into_inner();
    let block = subtokens.nth(0).unwrap();

    let block = match parse(Pairs::single(block), errors) {
        Some(b) => b,
        None => return None,
    };

    let module_kind = SyntaxKind::Module {
        block: Box::new(block),
    };

    let node = SyntaxToken::new(module_kind, &module);
    Some(node)
}

fn parse_block(block: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let subtokens = block.clone().into_inner();

    let mut tokens: Vec<SyntaxToken> = Vec::new();
    for subtoken in subtokens {
        let subtoken = match parse(Pairs::single(subtoken), errors) {
            Some(t) => t,
            None => return None,
        };

        tokens.push(subtoken);
    }

    let block_kind = SyntaxKind::Block {
        children: Box::new(tokens),
    };

    let node = SyntaxToken::new(block_kind, &block);
    Some(node)
}

fn parse_if_statement(statement: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = statement.clone().into_inner();

    let condition = match subtokens.nth(0) {
        Some(c) => parse(Pairs::single(c), errors),
        None => return None,
    };

    let condition = match condition {
        Some(t) => t,
        None => return None,
    };

    let block = subtokens.nth(0).unwrap();
    let block = match parse(Pairs::single(block), errors) {
        Some(b) => b,
        None => return None,
    };

    let if_kind = SyntaxKind::IfStatement {
        condition: Box::new(condition),
        block: Box::new(block),
    };

    let node = SyntaxToken::new(if_kind, &statement);
    Some(node)
}

fn parse_output_statement(statement: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = statement.clone().into_inner();

    let expr = match subtokens.nth(0) {
        Some(e) => parse(Pairs::single(e), errors),
        None => return None,
    };

    let expr = match expr {
        Some(t) => t,
        None => return None,
    };

    let output_kind = SyntaxKind::OutputStatement {
        expr: Box::new(expr),
    };

    let node = SyntaxToken::new(output_kind, &statement);
    Some(node)
}

fn parse_return_statement(statement: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = statement.clone().into_inner();

    let ret_expr = match subtokens.nth(0) {
        Some(e) => parse(Pairs::single(e), errors),
        None => return None,
    };

    let ret_expr = match ret_expr {
        Some(e) => e,
        None => return None,
    };

    let kind = SyntaxKind::ReturnStatement {
        expr: Box::new(ret_expr),
    };

    let node = SyntaxToken::new(kind, &statement);
    Some(node)
}

fn parse_function_declaration(
    declaration: Pair<Rule>,
    errors: &mut ErrorBag,
) -> Option<SyntaxToken> {
    let mut subtokens = declaration.clone().into_inner();

    let identifier = match subtokens.nth(0) {
        Some(i) => parse(Pairs::single(i), errors),
        None => return None,
    };

    let identifier = match identifier {
        Some(i) => i,
        None => return None,
    };

    let params = match subtokens.nth(0) {
        Some(p) => parse(Pairs::single(p), errors),
        None => return None,
    };

    let params = match params {
        Some(p) => p,
        None => return None,
    };

    let block_or_type = match subtokens.nth(0) {
        Some(f) => parse(Pairs::single(f), errors),
        None => return None,
    };

    let block_or_type = match block_or_type {
        Some(f) => f,
        None => return None,
    };

    let (type_annotation, block) = match block_or_type.kind {
        SyntaxKind::FunctionTypeAnnotation(id) => {
            let block = match subtokens.nth(0) {
                Some(b) => parse(Pairs::single(b), errors),
                None => return None,
            };

            let block = match block {
                Some(b) => b,
                None => return None,
            };

            (id, block)
        }
        _ => {
            // must be a block
            ("Void".to_string(), block_or_type)
        }
    };

    let kind = SyntaxKind::FunctionDeclaration {
        identifier: Box::new(identifier),
        params: Box::new(params),
        ret_type: type_annotation,
        block: Box::new(block),
    };

    let node = SyntaxToken::new(kind, &declaration);
    Some(node)
}

fn parse_func_type_annotation(annot: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = annot.clone().into_inner();
    let identifier = match subtokens.nth(0) {
        Some(i) => parse(Pairs::single(i), errors),
        None => return None,
    };

    let identifier = match identifier {
        Some(i) => i,
        None => return None,
    };

    let SyntaxKind::IdentifierToken(id) = identifier.kind else {
        return None;
    };

    let kind = SyntaxKind::FunctionTypeAnnotation(id);
    let node = SyntaxToken::new(kind, &annot);
    Some(node)
}

fn parse_parameter_list(params: Pair<Rule>, _errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = params.clone().into_inner();
    let num_subtokens = subtokens.len();
    let num_params = num_subtokens / 2;

    let mut parameters: Vec<ParameterSyntax> = Vec::new();
    for _ in 0..num_params {
        let identifier = match subtokens.nth(0) {
            Some(i) => i,
            None => return None,
        };

        let type_annotation = match subtokens.nth(0) {
            Some(i) => i,
            None => return None,
        };

        let loc = CodeLocation::from_pair(&identifier);
        let param = ParameterSyntax {
            identifier: String::from(identifier.as_str()),
            type_annotation: String::from(type_annotation.as_str()),
            location: loc,
        };

        parameters.push(param);
    }

    let kind = SyntaxKind::ParameterList { params: parameters };
    let node = SyntaxToken::new(kind, &params);
    Some(node)
}

fn parse_assignment_expression(expr: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = expr.clone().into_inner();
    let reference = match subtokens.nth(0) {
        Some(i) => parse_reference_expression(i, errors),
        None => return None,
    };

    let reference = match reference {
        Some(t) => t,
        None => return None,
    };

    let _assignment = subtokens.nth(0);

    let value = match subtokens.nth(0) {
        Some(v) => parse(Pairs::single(v), errors),
        None => return None,
    };

    let value = match value {
        Some(t) => t,
        None => return None,
    };

    let assignment_kind = SyntaxKind::AssignmentExpression {
        reference: Box::new(reference),
        value: Box::new(value),
    };

    let node = SyntaxToken::new(assignment_kind, &expr);
    Some(node)
}

fn parse_call_expression(expr: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let mut subtokens = expr.clone().into_inner();
    let identifier = match subtokens.nth(0) {
        Some(i) => parse(Pairs::single(i), errors),
        None => return None,
    };

    let identifier = match identifier {
        Some(i) => i,
        None => return None,
    };

    let args = match subtokens.nth(0) {
        Some(a) => a,
        None => return None,
    };

    let args = match parse_args_list(args, errors) {
        Some(a) => a,
        None => return None,
    };

    let kind = SyntaxKind::CallExpression {
        identifier: Box::new(identifier),
        args: Box::new(args),
    };

    let node = SyntaxToken::new(kind, &expr);
    Some(node)
}

fn parse_args_list(args: Pair<Rule>, errors: &mut ErrorBag) -> Option<Vec<SyntaxToken>> {
    let subtokens = args.clone().into_inner();

    let mut tokens: Vec<SyntaxToken> = Vec::new();
    for subtoken in subtokens {
        let parsed = parse(Pairs::single(subtoken), errors);
        match parsed {
            Some(t) => tokens.push(t),
            None => return None,
        };
    }

    Some(tokens)
}

fn parse_reference_expression(
    reference: Pair<Rule>,
    _errors: &mut ErrorBag,
) -> Option<SyntaxToken> {
    let identifier = match reference.clone().into_inner().nth(0) {
        Some(i) => String::from(i.as_str()),
        None => return None,
    };

    let reference_kind = SyntaxKind::ReferenceExpression(identifier);
    let node = SyntaxToken::new(reference_kind, &reference);
    Some(node)
}

fn parse_identifier_token(identifier: Pair<Rule>, _errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let identifier_token_kind = SyntaxKind::IdentifierToken(String::from(identifier.as_str()));
    let node = SyntaxToken::new(identifier_token_kind, &identifier);
    Some(node)
}

fn parse_literal_expression(literal: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let inner = match literal.clone().into_inner().nth(0) {
        Some(i) => parse(Pairs::single(i), errors),
        None => return None,
    };

    let inner = match inner {
        Some(t) => t,
        None => return None,
    };

    let literal_expr_kind = SyntaxKind::LiteralExpression(Box::new(inner));
    let node = SyntaxToken::new(literal_expr_kind, &literal);
    Some(node)
}

fn parse_number_token(pairs: Pair<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let (line, col) = pairs.line_col();
    let num = match pairs.as_str().parse::<i32>() {
        Ok(n) => n,
        Err(_) => {
            errors.add(ErrorKind::NumberParsing, line, col);
            return None;
        }
    };

    let number_token_kind = SyntaxKind::NumberToken(num);
    let node = SyntaxToken::new(number_token_kind, &pairs);
    Some(node)
}

fn parse_boolean_token(pairs: Pair<Rule>, _errors: &mut ErrorBag) -> Option<SyntaxToken> {
    let inner_text = pairs.as_str().to_lowercase();
    let value = if inner_text == "true" {
        true
    } else if inner_text == "false" {
        false
    } else {
        unreachable!()
    };

    let boolean_token_kind = SyntaxKind::BooleanToken(value);
    let node = SyntaxToken::new(boolean_token_kind, &pairs);
    Some(node)
}

fn parse(pairs: Pairs<Rule>, errors: &mut ErrorBag) -> Option<SyntaxToken> {
    PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::module => parse_module(primary, errors),
            Rule::block => parse_block(primary, errors),
            Rule::expression_statement => parse(primary.into_inner(), errors),
            Rule::if_statement => parse_if_statement(primary, errors),
            Rule::output_statement => parse_output_statement(primary, errors),
            Rule::return_statement => parse_return_statement(primary, errors),
            Rule::function_declaration_statement => parse_function_declaration(primary, errors),
            Rule::function_type_annotation => parse_func_type_annotation(primary, errors),
            Rule::parameter_list => parse_parameter_list(primary, errors),
            Rule::expression => parse(primary.into_inner(), errors),
            Rule::assignment_expression => parse_assignment_expression(primary, errors),
            Rule::call_expression => parse_call_expression(primary, errors),
            Rule::reference_expression => parse_reference_expression(primary, errors),
            Rule::literal_expression => parse_literal_expression(primary, errors),
            Rule::number_token => parse_number_token(primary, errors),
            Rule::boolean_token => parse_boolean_token(primary, errors),
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

            let lhs_token = match lhs {
                Some(t) => t,
                None => return None,
            };

            let rhs_token = match rhs {
                Some(t) => t,
                None => return None,
            };

            let loc = lhs_token.loc.clone();
            let expr_kind = SyntaxKind::BinaryExpression {
                lhs: Box::new(lhs_token),
                op: operator,
                rhs: Box::new(rhs_token),
            };

            let node = SyntaxToken {
                kind: expr_kind,
                loc: loc,
            };
            Some(node)
        })
        .map_prefix(|op, rhs| {
            let operator = match op.as_rule() {
                Rule::not => Operator::Not,
                _ => unreachable!(),
            };

            let rhs = match rhs {
                Some(t) => t,
                None => return None,
            };

            let expr_kind = SyntaxKind::UnaryExpression {
                op: operator,
                rhs: Box::new(rhs),
            };

            let node = SyntaxToken::new(expr_kind, &op);
            Some(node)
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
        Ok(mut pairs) => {
            println!("{:#?}", pairs.clone());
            parse(Pairs::single(pairs.next().unwrap()), errors)
        }
        Err(_) => None,
    }
}
