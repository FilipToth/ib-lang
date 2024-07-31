use std::{cell::RefCell, rc::Rc};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    parser::{ParameterSyntax, SyntaxKind, SyntaxToken},
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
    ret_expr: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let ret_expr = match bind(ret_expr, scope.clone(), errors) {
        Some(e) => e,
        None => return None,
    };

    let ret_type = ret_expr.node_type.clone();
    let kind = BoundNodeKind::ReturnStatement {
        expr: Box::new(ret_expr),
    };

    let node = BoundNode::new(kind, ret_type, loc);
    Some(node)
}

fn bind_if_statement(
    condition: &SyntaxToken,
    next: &SyntaxToken,
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

    let block = match bind(next, scope, errors) {
        Some(n) => n,
        None => return None,
    };

    let kind = BoundNodeKind::IfStatement {
        condition: Box::new(condition),
        block: Box::new(block),
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_function_declaration(
    identifier: &SyntaxToken,
    params: &SyntaxToken,
    ret_type: String,
    block: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let identifier = match identifier.kind {
        SyntaxKind::IdentifierToken(ref i) => i.clone(),
        _ => return None,
    };

    let func_scope = BoundScope::new(scope.clone());
    let func_scope_ref = Rc::new(RefCell::new(func_scope));

    let params = match params.kind {
        SyntaxKind::ParameterList { ref params } => {
            bind_params(params, func_scope_ref.clone(), errors)
        }
        _ => return None,
    };

    let params = match params {
        Some(p) => p,
        None => return None,
    };

    let ret_type = match get_type(ret_type, &loc, errors) {
        Some(t) => t,
        None => return None,
    };

    let block_loc = block.loc.clone();
    let SyntaxKind::Block { children } = &block.kind else {
        return None;
    };

    let block = match bind_block(&children, func_scope_ref, false, errors, block_loc) {
        Some(b) => b,
        None => return None,
    };

    let success = scope
        .borrow_mut()
        .declare_function(identifier.clone(), params.clone());

    if !success {
        let kind = ErrorKind::CannotDeclareFunction(identifier.clone());
        errors.add(kind, loc.line, loc.col);
        return None;
    }

    let kind = BoundNodeKind::FunctionDeclaration {
        identifier: identifier,
        params: params,
        ret_type,
        block: Rc::new(block),
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_params(
    params: &Vec<ParameterSyntax>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<Vec<BoundParameter>> {
    let mut parameters: Vec<BoundParameter> = Vec::new();
    for param in params {
        let identifier = param.identifier.clone();
        let type_identifier = param.type_annotation.clone();
        let loc = param.location.clone();

        let param_type = match get_type(type_identifier, &loc, errors) {
            Some(t) => t,
            None => return None,
        };

        let bound_param = BoundParameter {
            identifier: identifier.clone(),
            param_type: param_type.clone(),
        };

        // declare in scope
        let success = scope
            .borrow_mut()
            .assign_variable(identifier.clone(), param_type);

        if !success {
            let kind = ErrorKind::ParamMismatchedTypes(identifier);
            errors.add(kind, loc.line, loc.col);
            return None;
        }

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

fn bind_literal_expression(
    subtoken: &SyntaxToken,
    _errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    match subtoken.kind {
        SyntaxKind::NumberToken(number) => {
            let kind = BoundNodeKind::NumberLiteral(number.clone());
            let node = BoundNode::new(kind, TypeKind::Int, loc);
            Some(node)
        }
        SyntaxKind::BooleanToken(value) => {
            let kind = BoundNodeKind::BooleanLiteral(value.clone());
            let node = BoundNode::new(kind, TypeKind::Boolean, loc);
            Some(node)
        }
        _ => unreachable!(),
    }
}

fn bind_assignment_expression(
    reference: &SyntaxToken,
    value: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let identifier = match &reference.kind {
        SyntaxKind::ReferenceExpression(i) => i.clone(),
        _ => unreachable!(),
    };

    let value = match bind(value, scope.clone(), errors) {
        Some(v) => v,
        None => return None,
    };

    let node_type = value.node_type.clone();
    let success = scope
        .borrow_mut()
        .assign_variable(identifier.clone(), node_type.clone());

    if !success {
        errors.add(ErrorKind::AssignMismatchedTypes, loc.line, loc.col);
        return None;
    }

    let kind = BoundNodeKind::AssignmentExpression {
        identifier: identifier,
        value: Box::new(value),
    };

    let node = BoundNode::new(kind, node_type, loc);
    Some(node)
}

fn bind_call_expression(
    identifier: &SyntaxToken,
    args: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let SyntaxKind::IdentifierToken(id) = &identifier.kind else {
        return None;
    };

    let identifier = id.clone();
    let params = match scope.borrow().get_function(identifier.clone()) {
        Some(def) => def.parameters,
        None => {
            let kind = ErrorKind::CannotFindFunction(identifier);
            errors.add(kind, loc.line, loc.col);
            return None;
        }
    };

    // check if params match args
    let num_params = params.len();
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

    let kind = BoundNodeKind::BoundCallExpression {
        identifier: identifier,
        args: Box::new(bound_args),
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_reference_expression(
    identifier: String,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let ref_type = match scope.borrow().get_variable(identifier.clone()) {
        Some(def) => def.var_type,
        None => {
            errors.add(ErrorKind::CannotFindValue(identifier), loc.line, loc.col);
            return None;
        }
    };

    let kind = BoundNodeKind::ReferenceExpression(identifier);
    let node = BoundNode::new(kind, ref_type, loc);
    Some(node)
}

pub fn bind(
    token: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<BoundNode> {
    let loc = token.loc.clone();
    match &token.kind {
        SyntaxKind::Module { block } => bind_module(&block, scope, errors, loc),
        SyntaxKind::Block { children } => bind_block(&children, scope, true, errors, loc),
        SyntaxKind::OutputStatement { expr } => bind_output_statement(&expr, scope, errors, loc),
        SyntaxKind::ReturnStatement { expr } => bind_return_statement(&expr, scope, errors, loc),
        SyntaxKind::IfStatement { condition, block } => {
            bind_if_statement(&condition, &block, scope, errors, loc)
        }
        SyntaxKind::FunctionDeclaration {
            identifier,
            params,
            ret_type,
            block,
        } => bind_function_declaration(
            &identifier,
            &params,
            ret_type.to_string(),
            &block,
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
        SyntaxKind::LiteralExpression(subtoken) => bind_literal_expression(&subtoken, errors, loc),
        SyntaxKind::AssignmentExpression { reference, value } => {
            bind_assignment_expression(reference, value, scope, errors, loc)
        }
        SyntaxKind::CallExpression { identifier, args } => {
            bind_call_expression(&identifier, &args, scope, errors, loc)
        }
        SyntaxKind::ReferenceExpression(identifier) => {
            bind_reference_expression(identifier.clone(), scope, errors, loc)
        }
        _ => unreachable!(),
    }
}
