use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    span::Span,
    syntax::syntax_token::{SyntaxKind, SyntaxToken},
};

use super::{
    bound_node::{BoundNode, BoundNodeKind, BoundParameter},
    bound_scope::BoundScope,
    symbols::VariableSymbol,
    types::{get_type, TypeKind},
};

fn bind_module(
    block: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let block = match bind(block, scope, errors) {
        Some(b) => b,
        None => return None,
    };

    let node_type = block.node_type.clone();
    let kind = BoundNodeKind::Module {
        block: Box::new(block),
    };

    let node = BoundNode::new(kind, node_type, span);
    Some(node)
}

fn bind_block(
    children: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    create_child_scope: bool,
    errors: &mut ErrorBag,
    span: Span,
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

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

fn bind_output_statement(
    expr: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let expr = match bind(expr, scope, errors) {
        Some(expr) => expr,
        None => return None,
    };

    let kind = BoundNodeKind::OutputStatement {
        expr: Box::new(expr),
    };

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

fn bind_return_statement(
    ret_expr: &Option<Box<SyntaxToken>>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
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
    let node = BoundNode::new(kind, ret_type, span);
    Some(node)
}

fn bind_if_statement(
    condition: &SyntaxToken,
    next: &SyntaxToken,
    else_next: Option<&SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let condition = match bind(condition, scope.clone(), errors) {
        Some(cond) => cond,
        None => return None,
    };

    if condition.node_type != TypeKind::Boolean {
        errors.add(
            ErrorKind::ConditionMustBeBoolean(condition.node_type),
            condition.span,
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

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

fn bind_function_declaration(
    identifier: String,
    params: &Vec<SyntaxToken>,
    ret_type: &Option<String>,
    block: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
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

    let ret_type = match get_type(ret_type, None, &span, errors) {
        Some(t) => t,
        None => return None,
    };

    let block_span = block.span.clone();
    let SyntaxKind::Scope { subtokens } = &block.kind else {
        return None;
    };

    let block = match bind_block(&subtokens, func_scope_ref, false, errors, block_span) {
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
            block: Arc::new(block),
        },
        None => {
            let kind = ErrorKind::CannotDeclareFunction(identifier.clone());
            errors.add(kind, span);
            return None;
        }
    };

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

fn bind_for_statement(
    identifier: String,
    lower_bound: usize,
    upper_bound: usize,
    body: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let mut loop_scope = BoundScope::new(scope);
    let iterator = match loop_scope.assign_variable(identifier, TypeKind::Int) {
        Some(s) => s,
        None => return None,
    };

    let loop_scope = Rc::new(RefCell::new(loop_scope));
    let body = match bind(body, loop_scope, errors) {
        Some(b) => b,
        None => return None,
    };

    let kind = BoundNodeKind::ForLoop {
        iterator: iterator,
        lower_bound: lower_bound,
        upper_bound: upper_bound,
        block: Arc::new(body),
    };

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

fn bind_while_statement(
    expr: &SyntaxToken,
    body: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let expr = match bind(expr, scope.clone(), errors) {
        Some(e) => e,
        None => return None,
    };

    let expr_type = expr.node_type.clone();
    if expr_type != TypeKind::Boolean {
        let kind = ErrorKind::ConditionMustBeBoolean(expr_type);
        errors.add(kind, expr.span);
        return None;
    }

    let body = match bind(body, scope, errors) {
        Some(b) => b,
        None => return None,
    };

    let kind = BoundNodeKind::WhileLoop {
        expr: Box::new(expr),
        block: Arc::new(body),
    };

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

fn bind_params(
    params: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<Vec<BoundParameter>> {
    let mut parameters: Vec<BoundParameter> = Vec::new();
    for param in params {
        let span = param.span.clone();
        let SyntaxKind::Parameter {
            identifier,
            type_annotation,
        } = &param.kind
        else {
            return None;
        };

        let param_type = match get_type(type_annotation.clone(), None, &span, errors) {
            Some(t) => t,
            None => return None,
        };

        // declare in scope
        let symbol = scope
            .borrow_mut()
            .assign_variable(identifier.clone(), param_type.clone());

        if symbol.is_none() {
            let kind = ErrorKind::ParamMismatchedTypes(identifier.clone());
            errors.add(kind, span);
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
    span: Span,
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

    let node = BoundNode::new(kind, op_type, span);
    Some(node)
}

fn bind_unary_expression(
    op: &Operator,
    rhs: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
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

    let node = BoundNode::new(kind, op_type, span);
    Some(node)
}

fn bind_integer_literal(value: i64, _errors: &mut ErrorBag, span: Span) -> Option<BoundNode> {
    let kind = BoundNodeKind::NumberLiteral(value);
    let node = BoundNode::new(kind, TypeKind::Int, span);
    Some(node)
}

fn bind_boolean_literal(value: bool, _errors: &mut ErrorBag, span: Span) -> Option<BoundNode> {
    let kind = BoundNodeKind::BooleanLiteral(value);
    let node = BoundNode::new(kind, TypeKind::Boolean, span);
    Some(node)
}

fn bind_string_literal(value: String, _errors: &mut ErrorBag, span: Span) -> Option<BoundNode> {
    let kind = BoundNodeKind::StringLiteral(value);
    let node = BoundNode::new(kind, TypeKind::String, span);
    Some(node)
}

fn bind_assignment_expression(
    identifier: String,
    value: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
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
            errors.add(ErrorKind::AssignMismatchedTypes, span);
            return None;
        }
    };

    let node = BoundNode::new(kind, node_type, span);
    Some(node)
}

fn bind_call_expression(
    identifier: String,
    args: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let symbol = match scope.borrow().get_function(identifier.clone()) {
        Some(sym) => sym,
        None => {
            let kind = ErrorKind::CannotFindFunction(identifier);
            errors.add(kind, span);
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

        errors.add(kind, span);
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

            errors.add(kind, span);
            return None;
        }

        bound_args.push(bound_arg);
    }

    let ret_type = symbol.ret_type.clone();
    let kind = BoundNodeKind::BoundCallExpression {
        symbol: symbol,
        args: Box::new(bound_args),
    };

    let node = BoundNode::new(kind, ret_type, span);
    Some(node)
}

fn bind_reference_expression(
    identifier: String,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let symbol = match scope.borrow().get_variable(identifier.clone()) {
        Some(def) => def,
        None => {
            errors.add(ErrorKind::CannotFindValue(identifier), span);
            return None;
        }
    };

    let var_type = symbol.var_type.clone();
    let kind = BoundNodeKind::ReferenceExpression(symbol);
    let node = BoundNode::new(kind, var_type, span);
    Some(node)
}

fn bind_object_member_expression(
    base: &SyntaxToken,
    next: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let base_node = match bind(base, scope.clone(), errors) {
        Some(n) => n,
        None => return None,
    };

    // create a scope with all object member methods
    // and the run regular binding with that scope
    let mut object_scope = BoundScope::new_root();
    let type_methods = base_node.node_type.reflection_methods();

    for method in type_methods {
        let mut params = Vec::<BoundParameter>::new();
        let mut scope_mut = scope.borrow_mut();

        for param in method.params {
            let param = match scope_mut.assign_variable(param.identifier, param.param_type) {
                Some(p) => p,
                None => return None,
            };

            let param_type = param.var_type.clone();
            let bound_parameter = BoundParameter {
                symbol: param,
                param_type: param_type,
            };

            params.push(bound_parameter);
        }

        object_scope.declare_function(method.identifier, params, method.ret_type);
    }

    let next = match bind(next, Rc::new(RefCell::new(object_scope)), errors) {
        Some(n) => n,
        None => return None,
    };

    let node_type = next.node_type.clone();
    let kind = BoundNodeKind::ObjectMemberExpression {
        base: Box::new(base_node),
        next: Box::new(next),
    };

    let node = BoundNode::new(kind, node_type, span);
    Some(node)
}

fn bind_instantiation_expression(
    type_name: String,
    type_param: Option<String>,
    args: &Vec<SyntaxToken>,
    _scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let instantiation_type = match get_type(type_name.clone(), type_param, &span, errors) {
        Some(t) => t,
        None => return None,
    };

    if args.len() != 0 {
        // we don't support constructors with arguments yet
        let ctor = format!("{}.constructor()", type_name);
        let kind = ErrorKind::MismatchedNumberOfArgs {
            id: ctor,
            expected: 0,
            found: args.len(),
        };

        errors.add(kind, span);
        return None;
    }

    let kind = BoundNodeKind::ObjectExpression;
    let node = BoundNode::new(kind, instantiation_type, span);
    Some(node)
}

pub fn bind(
    token: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<BoundNode> {
    let span = token.span.clone();
    match &token.kind {
        SyntaxKind::Scope { subtokens } => bind_block(&subtokens, scope, true, errors, span),
        SyntaxKind::OutputStatement { expr } => bind_output_statement(&expr, scope, errors, span),
        SyntaxKind::ReturnStatement { expr } => bind_return_statement(&expr, scope, errors, span),
        SyntaxKind::IfStatement { condition, body } => {
            bind_if_statement(&condition, &body, None, scope, errors, span)
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
            span,
        ),
        SyntaxKind::ForLoop {
            identifier,
            lower_bound,
            upper_bound,
            body,
        } => bind_for_statement(
            identifier.clone(),
            lower_bound.clone(),
            upper_bound.clone(),
            &body,
            scope,
            errors,
            span,
        ),
        SyntaxKind::WhileLoop { expr, body } => {
            bind_while_statement(&expr, &body, scope, errors, span)
        }
        SyntaxKind::BinaryExpression { lhs, op, rhs } => {
            bind_binary_expression(&lhs, &op, &rhs, scope, errors, span)
        }
        SyntaxKind::UnaryExpression { op, rhs } => {
            bind_unary_expression(&op, &rhs, scope, errors, span)
        }
        SyntaxKind::IntegerLiteralExpression(value) => {
            bind_integer_literal(value.clone(), errors, span)
        }
        SyntaxKind::BooleanLiteralExpression(value) => {
            bind_boolean_literal(value.clone(), errors, span)
        }
        SyntaxKind::StringLiteralExpression(value) => {
            bind_string_literal(value.clone(), errors, span)
        }
        SyntaxKind::AssignmentExpression { identifier, value } => {
            bind_assignment_expression(identifier.clone(), value, scope, errors, span)
        }
        SyntaxKind::CallExpression { identifier, args } => {
            bind_call_expression(identifier.clone(), &args, scope, errors, span)
        }
        SyntaxKind::ReferenceExpression(identifier) => {
            bind_reference_expression(identifier.clone(), scope, errors, span)
        }
        SyntaxKind::ObjectMemberExpression { base, next } => {
            bind_object_member_expression(&base, &next, scope, errors, span)
        }
        SyntaxKind::InstantiationExpression {
            type_name,
            type_param,
            args,
        } => bind_instantiation_expression(
            type_name.clone(),
            type_param.clone(),
            &args,
            scope,
            errors,
            span,
        ),
        SyntaxKind::ParenthesizedExpression { inner } => bind(&inner, scope, errors),
        _ => {
            println!("unknown: {:?}", token.kind);
            unreachable!()
        }
    }
}
