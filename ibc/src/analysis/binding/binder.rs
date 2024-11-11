use std::{cell::RefCell, rc::Rc};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    syntax::syntax_token::{SyntaxKind, SyntaxToken},
    CodeLocation,
};

use super::{
    bound_node::{BoundNode, BoundNodeKind, BoundParameter},
    bound_scope::BoundScope,
    types::{get_type, TypeKind},
};

fn bind_module(
    block: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let block = match bind(block, scope, errors) {
        Some(b) => b,
        None => return None,
    };

    let node_type = block.node_type.clone();
    let kind = BoundNodeKind::Module {
        block: Box::new(block),
    };

    let node = BoundNode::new(kind, node_type, loc);
    Some(node)
}

fn bind_block(
    children: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    create_child_scope: bool,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let scope_ref = if create_child_scope {
        let child_scope = BoundScope::new(scope);
        Rc::new(RefCell::new(child_scope))
    } else {
        scope
    };

    let mut bound = Vec::<BoundNode>::new();
    for child in children {
        let bound_child = match bind(child, scope_ref.clone(), errors) {
            Some(n) => n,
            None => return None,
        };

        bound.push(bound_child);
    }

    let kind = BoundNodeKind::Block {
        children: Box::new(bound),
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_output_statement(
    expr: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let expr = match bind(expr, scope, errors) {
        Some(expr) => expr,
        None => return None,
    };

    let kind = BoundNodeKind::OutputStatement {
        expr: Box::new(expr),
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_return_statement(
    ret_expr: &Option<Box<SyntaxToken>>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let (ret_type, expr) = match ret_expr {
        Some(ret_expr) => {
            let ret_expr = match bind(ret_expr, scope.clone(), errors) {
                Some(e) => e,
                None => return None,
            };

            let ret_type = ret_expr.node_type.clone();
            let ret_expr = Some(Box::new(ret_expr));
            (ret_type, ret_expr)
        }
        None => (TypeKind::Void, None),
    };

    let kind = BoundNodeKind::ReturnStatement { expr: expr };
    let node = BoundNode::new(kind, ret_type, loc);
    Some(node)
}

fn bind_if_statement(
    condition: &SyntaxToken,
    next: &SyntaxToken,
    else_next: Option<&SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let condition = match bind(condition, scope.clone(), errors) {
        Some(cond) => cond,
        None => return None,
    };

    if condition.node_type != TypeKind::Boolean {
        errors.add(
            ErrorKind::ConditionMustBeBoolean(condition.node_type),
            loc.line,
            loc.col,
        );
        return None;
    }

    let block = match bind(next, scope.clone(), errors) {
        Some(n) => n,
        None => return None,
    };

    let else_block = match else_next {
        Some(e) => match bind(e, scope, errors) {
            Some(e) => Some(Box::new(e)),
            None => return None,
        },
        None => None,
    };

    let kind = BoundNodeKind::IfStatement {
        condition: Box::new(condition),
        block: Box::new(block),
        else_block: else_block,
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_function_declaration(
    identifier: String,
    params: &Vec<SyntaxToken>,
    ret_type: &Option<String>,
    block: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let func_scope = BoundScope::new(scope.clone());
    let func_scope_ref = Rc::new(RefCell::new(func_scope));

    let params = bind_params(params, func_scope_ref.clone(), errors);
    let params = match params {
        Some(p) => p,
        None => return None,
    };

    let ret_type = match ret_type {
        Some(t) => t,
        None => "Void",
    }
    .to_string();

    let ret_type = match get_type(ret_type, &loc, errors) {
        Some(t) => t,
        None => return None,
    };

    let block_loc = block.loc.clone();
    let SyntaxKind::Scope { subtokens } = &block.kind else {
        return None;
    };

    let block = match bind_block(&subtokens, func_scope_ref, false, errors, block_loc) {
        Some(b) => b,
        None => return None,
    };

    let symbol =
        scope
            .borrow_mut()
            .declare_function(identifier.clone(), params.clone(), ret_type.clone());

    let kind = match symbol {
        Some(s) => BoundNodeKind::FunctionDeclaration {
            symbol: s,
            block: Rc::new(block),
        },
        None => {
            let kind = ErrorKind::CannotDeclareFunction(identifier.clone());
            errors.add(kind, loc.line, loc.col);
            return None;
        }
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_params(
    params: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<Vec<BoundParameter>> {
    let mut parameters: Vec<BoundParameter> = Vec::new();
    for param in params {
        let loc = param.loc.clone();
        let SyntaxKind::Parameter {
            identifier,
            type_annotation,
        } = &param.kind
        else {
            return None;
        };

        let param_type = match get_type(type_annotation.clone(), &loc, errors) {
            Some(t) => t,
            None => return None,
        };

        // declare in scope
        let symbol = scope
            .borrow_mut()
            .assign_variable(identifier.clone(), param_type.clone());

        if symbol.is_none() {
            let kind = ErrorKind::ParamMismatchedTypes(identifier.clone());
            errors.add(kind, loc.line, loc.col);
            return None;
        }

        let bound_param = BoundParameter {
            symbol: symbol.unwrap(),
            param_type: param_type.clone(),
        };

        parameters.push(bound_param);
    }

    Some(parameters)
}

fn bind_binary_expression(
    lhs: &SyntaxToken,
    op: &Operator,
    rhs: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let lhs = match bind(lhs, scope.clone(), errors) {
        Some(n) => n,
        None => return None,
    };

    let rhs = match bind(rhs, scope, errors) {
        Some(n) => n,
        None => return None,
    };

    let op_type = match op.return_type_binary(&lhs, &rhs, errors) {
        Some(t) => t,
        None => return None,
    };

    let kind = BoundNodeKind::BinaryExpression {
        lhs: Box::new(lhs),
        op: op.clone(),
        rhs: Box::new(rhs),
    };

    let node = BoundNode::new(kind, op_type, loc);
    Some(node)
}

fn bind_unary_expression(
    op: &Operator,
    rhs: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let rhs = match bind(rhs, scope, errors) {
        Some(n) => n,
        None => return None,
    };

    let op_type = match op.return_type_unary(&rhs, errors) {
        Some(t) => t,
        None => return None,
    };

    let kind = BoundNodeKind::UnaryExpression {
        op: op.clone(),
        rhs: Box::new(rhs),
    };

    let node = BoundNode::new(kind, op_type, loc);
    Some(node)
}

fn bind_integer_literal(
    value: i64,
    _errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let kind = BoundNodeKind::NumberLiteral(value);
    let node = BoundNode::new(kind, TypeKind::Int, loc);
    Some(node)
}

fn bind_boolean_literal(
    value: bool,
    _errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let kind = BoundNodeKind::BooleanLiteral(value);
    let node = BoundNode::new(kind, TypeKind::Boolean, loc);
    Some(node)
}

fn bind_assignment_expression(
    identifier: String,
    value: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let value = match bind(value, scope.clone(), errors) {
        Some(v) => v,
        None => return None,
    };

    let node_type = value.node_type.clone();
    let symbol = scope
        .borrow_mut()
        .assign_variable(identifier.clone(), node_type.clone());

    let kind = match symbol {
        Some(s) => BoundNodeKind::AssignmentExpression {
            symbol: s,
            value: Box::new(value),
        },
        None => {
            errors.add(ErrorKind::AssignMismatchedTypes, loc.line, loc.col);
            return None;
        }
    };

    let node = BoundNode::new(kind, node_type, loc);
    Some(node)
}

fn bind_call_expression(
    identifier: String,
    args: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let symbol = match scope.borrow().get_function(identifier.clone()) {
        Some(sym) => sym,
        None => {
            let kind = ErrorKind::CannotFindFunction(identifier);
            errors.add(kind, loc.line, loc.col);
            return None;
        }
    };

    let params = &symbol.parameters;
    let num_params = params.len();

    // check if params match args
    if num_params != args.len() {
        let kind = ErrorKind::MismatchedNumberOfArgs {
            id: identifier.clone(),
            expected: num_params,
            found: args.len(),
        };

        errors.add(kind, loc.line, loc.col);
        return None;
    }

    let mut bound_args: Vec<BoundNode> = Vec::new();
    for index in 0..num_params {
        let param = &params[index];
        let arg = &args[index];

        let bound_arg = match bind(arg, scope.clone(), errors) {
            Some(a) => a,
            None => return None,
        };

        if param.param_type != bound_arg.node_type {
            let kind = ErrorKind::MismatchedArgTypes {
                id: identifier.clone(),
                expected: param.param_type.clone(),
                found: bound_arg.node_type,
            };

            errors.add(kind, loc.line, loc.col);
            return None;
        }

        bound_args.push(bound_arg);
    }

    let ret_type = symbol.ret_type.clone();
    let kind = BoundNodeKind::BoundCallExpression {
        symbol: symbol,
        args: Box::new(bound_args),
    };

    let node = BoundNode::new(kind, ret_type, loc);
    Some(node)
}

fn bind_reference_expression(
    identifier: String,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let symbol = match scope.borrow().get_variable(identifier.clone()) {
        Some(def) => def,
        None => {
            errors.add(ErrorKind::CannotFindValue(identifier), loc.line, loc.col);
            return None;
        }
    };

    let var_type = symbol.var_type.clone();
    let kind = BoundNodeKind::ReferenceExpression(symbol);
    let node = BoundNode::new(kind, var_type, loc);
    Some(node)
}

pub fn bind(
    token: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<BoundNode> {
    let loc = token.loc.clone();
    match &token.kind {
        SyntaxKind::Scope { subtokens } => bind_block(&subtokens, scope, true, errors, loc),
        SyntaxKind::OutputStatement { expr } => bind_output_statement(&expr, scope, errors, loc),
        SyntaxKind::ReturnStatement { expr } => bind_return_statement(&expr, scope, errors, loc),
        SyntaxKind::IfStatement { condition, body } => {
            bind_if_statement(&condition, &body, None, scope, errors, loc)
        }
        SyntaxKind::FunctionDeclaration {
            identifier,
            parameters,
            return_type,
            body,
        } => bind_function_declaration(
            identifier.clone(),
            parameters,
            return_type,
            &body,
            scope,
            errors,
            loc,
        ),
        SyntaxKind::BinaryExpression { lhs, op, rhs } => {
            bind_binary_expression(&lhs, &op, &rhs, scope, errors, loc)
        }
        SyntaxKind::UnaryExpression { op, rhs } => {
            bind_unary_expression(&op, &rhs, scope, errors, loc)
        }
        SyntaxKind::IntegerLiteralExpression(value) => {
            bind_integer_literal(value.clone(), errors, loc)
        }
        SyntaxKind::BooleanLiteralExpression(value) => {
            bind_boolean_literal(value.clone(), errors, loc)
        }
        SyntaxKind::AssignmentExpression { identifier, value } => {
            bind_assignment_expression(identifier.clone(), value, scope, errors, loc)
        }
        SyntaxKind::CallExpression { identifier, args } => {
            bind_call_expression(identifier.clone(), &args, scope, errors, loc)
        }
        SyntaxKind::ReferenceExpression(identifier) => {
            bind_reference_expression(identifier.clone(), scope, errors, loc)
        }
        SyntaxKind::ParenthesizedExpression { inner } => bind(&inner, scope, errors),
        _ => {
            println!("unknown: {:?}", token.kind);
            unreachable!()
        }
    }
}
