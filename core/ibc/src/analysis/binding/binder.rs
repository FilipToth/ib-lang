use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    span::Span,
    syntax::syntax_token::{SyntaxKind, SyntaxToken, TypeAnnotation},
};

use super::{
    bound_node::{BoundNode, BoundNodeKind, BoundParameter},
    bound_scope::BoundScope,
    symbols::FunctionSymbol,
    types::{get_type, TypeKind},
};

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

    // every function in the block is declared before any statement binds, so
    // a function can call one written after it. mutual recursion needs this:
    // each body refers to the other, so neither order works one at a time.
    let mut signatures: Vec<Option<FunctionSignature>> = Vec::with_capacity(children.len());
    for child in children {
        let signature = match &child.kind {
            SyntaxKind::FunctionDeclaration {
                identifier,
                parameters,
                return_type,
                body: _,
            } => Some(declare_function_signature(
                identifier.clone(),
                parameters,
                return_type,
                scope_ref.clone(),
                errors,
                child.span,
            )?),
            _ => None,
        };

        signatures.push(signature);
    }

    let mut bound = Vec::<BoundNode>::new();
    for (child, signature) in children.iter().zip(signatures) {
        let bound_child = match (signature, &child.kind) {
            (Some(signature), SyntaxKind::FunctionDeclaration { body, .. }) => {
                bind_function_body(signature, body, errors, child.span)?
            }
            _ => bind(child, scope_ref.clone(), errors)?,
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
    exprs: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let mut bound_exprs: Vec<BoundNode> = Vec::new();
    for expr in exprs {
        bound_exprs.push(bind(expr, scope.clone(), errors)?);
    }

    let kind = BoundNodeKind::OutputStatement {
        exprs: bound_exprs,
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
            let ret_expr = bind(ret_expr, scope.clone(), errors)?;

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
    let condition = bind(condition, scope.clone(), errors)?;

    if condition.node_type != TypeKind::Boolean {
        errors.add(
            ErrorKind::ConditionMustBeBoolean(condition.node_type),
            condition.span,
        );
        return None;
    }

    let block = bind(next, scope.clone(), errors)?;

    let else_block = match else_next {
        Some(e) => Some(Box::new(bind(e, scope, errors)?)),
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

/// A function's signature, declared ahead of its body.
///
/// The body binds against the very parameter symbols the signature created,
/// which is why the scope holding them travels with it rather than being built
/// again when the body's turn comes.
struct FunctionSignature {
    symbol: FunctionSymbol,
    params: Vec<BoundParameter>,
    scope: Rc<RefCell<BoundScope>>,
}

fn declare_function_signature(
    identifier: String,
    params: &Vec<SyntaxToken>,
    ret_type: &Option<TypeAnnotation>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<FunctionSignature> {
    let func_scope = BoundScope::new(scope.clone());
    let func_scope_ref = Rc::new(RefCell::new(func_scope));

    let params = bind_params(params, func_scope_ref.clone(), errors)?;

    let ret_type = match ret_type {
        Some(t) => get_type(t, errors)?,
        None => TypeKind::Void,
    };

    let symbol =
        scope
            .borrow_mut()
            .declare_function(identifier.clone(), params.clone(), ret_type.clone());

    let Some(symbol) = symbol else {
        let kind = ErrorKind::CannotDeclareFunction(identifier.clone());
        errors.add(kind, span);
        return None;
    };

    let signature = FunctionSignature {
        symbol: symbol,
        params: params,
        scope: func_scope_ref,
    };

    Some(signature)
}

fn bind_function_body(
    signature: FunctionSignature,
    block: &SyntaxToken,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let FunctionSignature {
        symbol,
        params,
        scope,
    } = signature;

    let block_span = block.span.clone();
    let SyntaxKind::Scope { subtokens } = &block.kind else {
        return None;
    };

    // everything allocated while the body binds belongs to this function, at
    // any depth, so a call can give it storage of its own
    scope.borrow().enter_function(symbol.symbol_id);
    let block = bind_block(&subtokens, scope.clone(), false, errors, block_span);
    scope.borrow().exit_function();

    let block = block?;

    // the parameters were allocated before the function had an id
    let mut locals: Vec<u64> = params.iter().map(|p| p.symbol.symbol_id).collect();
    locals.extend(scope.borrow().locals_of(symbol.symbol_id));

    let kind = BoundNodeKind::FunctionDeclaration {
        symbol: symbol,
        block: Arc::new(block),
        locals: locals,
    };

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

/// Binds one loop bound in the enclosing scope and checks it yields an Int.
fn bind_loop_bound(
    bound: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<BoundNode> {
    let bound = bind(bound, scope, errors)?;

    let bound_type = bound.node_type.clone();
    if bound_type != TypeKind::Int && bound_type != TypeKind::Any {
        let kind = ErrorKind::LoopBoundMustBeInt(bound_type);
        errors.add(kind, bound.span);
        return None;
    }

    Some(bound)
}

fn bind_for_statement(
    identifier: String,
    lower_bound: &SyntaxToken,
    upper_bound: &SyntaxToken,
    body: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    // the bounds are evaluated once, before the loop variable exists, so they
    // bind in the enclosing scope rather than the loop's own
    let lower_bound = bind_loop_bound(lower_bound, scope.clone(), errors)?;
    let upper_bound = bind_loop_bound(upper_bound, scope.clone(), errors)?;

    let mut loop_scope = BoundScope::new(scope);
    let iterator = loop_scope.assign_variable(identifier, TypeKind::Int)?;

    let loop_scope = Rc::new(RefCell::new(loop_scope));
    let body = bind(body, loop_scope, errors)?;

    let kind = BoundNodeKind::ForLoop {
        iterator: iterator,
        lower_bound: Box::new(lower_bound),
        upper_bound: Box::new(upper_bound),
        block: Arc::new(body),
    };

    let node = BoundNode::new(kind, TypeKind::Void, span);
    Some(node)
}

/// `loop until X` runs while X is false. Binding mirrors the while loop; the
/// inversion happens at evaluation.
fn bind_until_statement(
    expr: &SyntaxToken,
    body: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let expr = bind(expr, scope.clone(), errors)?;

    let expr_type = expr.node_type.clone();
    if expr_type != TypeKind::Boolean {
        let kind = ErrorKind::ConditionMustBeBoolean(expr_type);
        errors.add(kind, expr.span);
        return None;
    }

    let body = bind(body, scope, errors)?;

    let kind = BoundNodeKind::UntilLoop {
        expr: Box::new(expr),
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
    let expr = bind(expr, scope.clone(), errors)?;

    let expr_type = expr.node_type.clone();
    if expr_type != TypeKind::Boolean {
        let kind = ErrorKind::ConditionMustBeBoolean(expr_type);
        errors.add(kind, expr.span);
        return None;
    }

    let body = bind(body, scope, errors)?;

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

        let param_type = match type_annotation {
            Some(t) => get_type(t, errors)?,
            None => TypeKind::Any,
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
    let lhs = bind(lhs, scope.clone(), errors)?;

    let rhs = bind(rhs, scope, errors)?;

    let op_type = op.return_type_binary(&lhs, &rhs, errors)?;

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
    let rhs = bind(rhs, scope, errors)?;

    let op_type = op.return_type_unary(&rhs, errors)?;

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
    let value = bind(value, scope.clone(), errors)?;

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

/// `scope` resolves the function and `arg_scope` binds its arguments. For an
/// ordinary call they are the same scope. A method call looks the method up on
/// the object, but its arguments are still expressions from where the call was
/// written.
fn bind_call_expression(
    identifier: String,
    args: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    arg_scope: Rc<RefCell<BoundScope>>,
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

        let bound_arg = bind(arg, arg_scope.clone(), errors)?;

        if param.param_type != bound_arg.node_type
            && param.param_type != TypeKind::Any
            && bound_arg.node_type != TypeKind::Any
        {
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

/// Builds the internal name a type method's parameter is declared under.
///
/// Parameters are declared in the caller's scope so the evaluator can read
/// their values back out of the heap, which is the same convention user
/// functions use. Two things keep that from colliding with anything:
///
/// - the name begins with `$`, which cannot start an identifier, so user code
///   can neither declare nor reference one;
/// - it encodes the receiver type and method, so two methods that happen to
///   share a parameter name -- `Stack<Int>.push(item)` and
///   `Queue<String>.enqueue(item)` -- get distinct names rather than colliding
///   on `item` with incompatible types.
///
/// `$` separates the parts as well as leading them, so `VariableSymbol::name()`
/// can recover the user's name as the segment after the last `$`.
fn type_method_param_name(base_type: &TypeKind, method: &str, param: &str) -> String {
    format!("$param${}${}${}", base_type.to_string(), method, param)
}

/// Binds the `base[index]` shared by reads and writes, returning both along
/// with the array's element type. The spec only gives arrays index notation,
/// so stacks, queues and collections are rejected.
fn bind_index_target(
    base: &SyntaxToken,
    index: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<(BoundNode, BoundNode, TypeKind)> {
    let base = bind(base, scope.clone(), errors)?;
    let index = bind(index, scope, errors)?;

    let element_type = match &base.node_type {
        TypeKind::Array(generic) => *generic.clone(),
        TypeKind::Any => TypeKind::Any,
        base_type => {
            let kind = ErrorKind::CannotIndexType(base_type.clone());
            errors.add(kind, base.span);
            return None;
        }
    };

    let index_type = index.node_type.clone();
    if index_type != TypeKind::Int && index_type != TypeKind::Any {
        let kind = ErrorKind::IndexMustBeInt(index_type);
        errors.add(kind, index.span);
        return None;
    }

    Some((base, index, element_type))
}

fn bind_index_expression(
    base: &SyntaxToken,
    index: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let (base, index, element_type) = bind_index_target(base, index, scope, errors)?;

    let kind = BoundNodeKind::IndexExpression {
        base: Box::new(base),
        index: Box::new(index),
    };

    let node = BoundNode::new(kind, element_type, span);
    Some(node)
}

/// Binds `base[index] = value`. The value has to match the array's element
/// type, the same rule reassigning a variable follows.
fn bind_index_assignment_expression(
    base: &SyntaxToken,
    index: &SyntaxToken,
    value: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let (base, index, element_type) = bind_index_target(base, index, scope.clone(), errors)?;
    let value = bind(value, scope, errors)?;

    let value_type = value.node_type.clone();
    let matches = element_type == value_type
        || element_type == TypeKind::Any
        || value_type == TypeKind::Any;

    if !matches {
        errors.add(ErrorKind::AssignMismatchedTypes, span);
        return None;
    }

    let kind = BoundNodeKind::IndexAssignmentExpression {
        base: Box::new(base),
        index: Box::new(index),
        value: Box::new(value),
    };

    let node = BoundNode::new(kind, value_type, span);
    Some(node)
}

fn bind_object_member_expression(
    base: &SyntaxToken,
    next: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let base_node = bind(base, scope.clone(), errors)?;

    // create a scope with all object member methods
    // and the run regular binding with that scope
    let mut object_scope = BoundScope::new_root();
    let type_methods = base_node.node_type.reflection_methods();

    for method in type_methods {
        let mut params = Vec::<BoundParameter>::new();
        let mut scope_mut = scope.borrow_mut();
        let method_identifier = method.identifier.clone();

        for param in method.params {
            let identifier =
                type_method_param_name(&base_node.node_type, &method_identifier, &param.identifier);

            let param_symbol = match scope_mut.assign_variable(identifier, param.param_type) {
                Some(symbol) => symbol,
                None => {
                    // unreachable while the mangled name encodes the receiver
                    // type: the same name always carries the same type. report
                    // rather than bail silently if that ever stops holding,
                    // using the name the user would know.
                    let kind = ErrorKind::ConflictingDeclaration(param.identifier);
                    errors.add(kind, span);
                    return None;
                }
            };

            let param_type = param_symbol.var_type.clone();
            let bound_parameter = BoundParameter {
                symbol: param_symbol,
                param_type: param_type,
            };

            params.push(bound_parameter);
        }

        object_scope.declare_function(method_identifier, params, method.ret_type);
    }

    let object_scope = Rc::new(RefCell::new(object_scope));
    let next = match &next.kind {
        SyntaxKind::CallExpression { identifier, args } => bind_call_expression(
            identifier.clone(),
            args,
            object_scope,
            scope,
            errors,
            next.span,
        )?,
        _ => bind(next, object_scope, errors)?,
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
    type_annotation: &TypeAnnotation,
    args: &Vec<SyntaxToken>,
    _scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    span: Span,
) -> Option<BoundNode> {
    let instantiation_type = get_type(type_annotation, errors)?;

    if args.len() != 0 {
        // we don't support constructors with arguments yet
        let ctor = format!("{}.constructor()", type_annotation.name);
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
        SyntaxKind::OutputStatement { exprs } => bind_output_statement(&exprs, scope, errors, span),
        SyntaxKind::ReturnStatement { expr } => bind_return_statement(&expr, scope, errors, span),
        SyntaxKind::IfStatement {
            condition,
            body,
            else_body,
        } => bind_if_statement(
            &condition,
            &body,
            else_body.as_deref(),
            scope,
            errors,
            span,
        ),
        SyntaxKind::FunctionDeclaration {
            identifier,
            parameters,
            return_type,
            body,
        } => {
            // unreached in practice: a declaration is always a statement in a
            // block, and `bind_block` binds those itself, signatures first.
            // kept whole so binding one on its own still works.
            let signature = declare_function_signature(
                identifier.clone(),
                parameters,
                return_type,
                scope,
                errors,
                span,
            )?;

            bind_function_body(signature, &body, errors, span)
        }
        SyntaxKind::ForLoop {
            identifier,
            lower_bound,
            upper_bound,
            body,
        } => bind_for_statement(
            identifier.clone(),
            &lower_bound,
            &upper_bound,
            &body,
            scope,
            errors,
            span,
        ),
        SyntaxKind::WhileLoop { expr, body } => {
            bind_while_statement(&expr, &body, scope, errors, span)
        }
        SyntaxKind::UntilLoop { expr, body } => {
            bind_until_statement(&expr, &body, scope, errors, span)
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
            bind_call_expression(identifier.clone(), &args, scope.clone(), scope, errors, span)
        }
        SyntaxKind::ReferenceExpression(identifier) => {
            bind_reference_expression(identifier.clone(), scope, errors, span)
        }
        SyntaxKind::IndexExpression { base, index } => {
            bind_index_expression(&base, &index, scope, errors, span)
        }
        SyntaxKind::IndexAssignmentExpression { base, index, value } => {
            bind_index_assignment_expression(&base, &index, &value, scope, errors, span)
        }
        SyntaxKind::ObjectMemberExpression { base, next } => {
            bind_object_member_expression(&base, &next, scope, errors, span)
        }
        SyntaxKind::InstantiationExpression {
            type_annotation,
            args,
        } => bind_instantiation_expression(&type_annotation, &args, scope, errors, span),
        SyntaxKind::ParenthesizedExpression { inner } => bind(&inner, scope, errors),
        _ => unreachable!("unhandled syntax kind: {:?}", token.kind),
    }
}
