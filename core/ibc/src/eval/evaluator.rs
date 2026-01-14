use std::{
    collections::HashMap,
    fmt, mem,
    sync::{Arc, Mutex},
};

use async_recursion::async_recursion;

use crate::analysis::{
    binding::{
        bound_node::{BoundNode, BoundNodeKind},
        symbols::{FunctionSymbol, VariableSymbol},
        types::{get_object_state, ObjectState},
    },
    operator::Operator,
    span::Span,
};

use super::{
    eval_builtin,
    object_methods::{eval_type_method, get_element, set_element},
    EvalIO,
};

pub struct EvalInfo {
    pub heap: EvalHeap,
}

pub struct EvalHeap {
    // just use rust's heap to manage
    // memory, no need for us to make
    // our own heap
    variables: HashMap<u64, EvalValue>,
    functions: HashMap<u64, Arc<BoundNode>>,
}

impl EvalHeap {
    fn new() -> EvalHeap {
        EvalHeap {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn assign_var(&mut self, symbol: &VariableSymbol, val: EvalValue) {
        let id = symbol.symbol_id;
        self.variables.insert(id, val);
    }

    pub fn get_var(&self, symbol: &VariableSymbol) -> EvalValue {
        let id = &symbol.symbol_id;
        let value = &self.variables[id];
        value.clone()
    }

    pub fn declare_func(&mut self, symbol: &FunctionSymbol, body: Arc<BoundNode>) {
        let id = symbol.symbol_id;
        self.functions.insert(id, body);
    }

    pub fn get_func(&self, symbol: &FunctionSymbol) -> Arc<BoundNode> {
        let id = &symbol.symbol_id;
        let body = &self.functions[id];
        body.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EvalValue {
    Void,
    Int(i64),
    Bool(bool),
    String(String),
    // used for non-primitive types
    Object(Arc<tokio::sync::Mutex<ObjectState>>),
}

/// A failure while running a program, such as dividing by zero. It carries the
/// span of the node that raised it, so it can be reported with a location.
#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // spans count lines from 0, editors count them from 1
        write!(f, "{} on line {}", self.message, self.span.start.line + 1)
    }
}

/// Anything that stops a program carrying on to its next statement. It travels
/// as the `Err` side of an `EvalResult`, so `?` carries it up through every
/// construct until something catches it: a function call catches `Return`, and
/// only the top level catches `Error`.
#[derive(Debug)]
pub enum Signal {
    Return(EvalValue),
    Error(RuntimeError),
}

pub type EvalResult<T = EvalValue> = Result<T, Signal>;

/// Raises a runtime error at `span`.
pub fn runtime_error<T>(message: impl Into<String>, span: Span) -> EvalResult<T> {
    let error = RuntimeError {
        message: message.into(),
        span: span,
    };

    Err(Signal::Error(error))
}

impl EvalValue {
    fn void() -> EvalValue {
        EvalValue::Void
    }

    fn int(val: i64) -> EvalValue {
        EvalValue::Int(val)
    }

    fn bool(val: bool) -> EvalValue {
        EvalValue::Bool(val)
    }

    fn string(val: String) -> EvalValue {
        EvalValue::String(val)
    }

    pub fn to_string(&self) -> String {
        match self {
            EvalValue::Void => "void".to_string(),
            EvalValue::Int(val) => val.to_string(),
            EvalValue::Bool(val) => val.to_string(),
            EvalValue::String(val) => val.clone(),
            EvalValue::Object(_) => unreachable!(),
        }
    }

    fn get_int(&self, span: Span) -> EvalResult<i64> {
        let EvalValue::Int(val) = self else {
            let msg = format!("Expected integer, got {}", self.to_string());
            return runtime_error(msg, span);
        };

        Ok(val.clone())
    }

    fn get_bool(&self, span: Span) -> EvalResult<bool> {
        let EvalValue::Bool(val) = self else {
            let msg = format!("Expected bool, got {}", self.to_string());
            return runtime_error(msg, span);
        };

        Ok(val.clone())
    }

    fn get_string(&self, span: Span) -> EvalResult<String> {
        let EvalValue::String(val) = self else {
            let msg = format!("Expected string, got {}", self.to_string());
            return runtime_error(msg, span);
        };

        Ok(val.clone())
    }
}

fn eval_int_only_binexpr(lhs: EvalValue, op: &Operator, rhs: EvalValue, span: Span) -> EvalResult {
    let lhs = lhs.get_int(span)?;
    let rhs = rhs.get_int(span)?;

    // division and both integer-remainder forms trap on a zero divisor
    // rather than panicking the whole evaluator
    if rhs == 0 {
        let divides = matches!(
            op,
            Operator::Division | Operator::Modulo | Operator::IntDivision
        );

        if divides {
            return runtime_error("Division by zero", span);
        }
    }

    let value = match op {
        Operator::Subtraction => EvalValue::int(lhs - rhs),
        Operator::Multiplication => EvalValue::int(lhs * rhs),
        Operator::Division => EvalValue::int(lhs / rhs),
        Operator::Modulo => EvalValue::int(lhs % rhs),
        Operator::IntDivision => EvalValue::int(lhs / rhs),
        Operator::LesserThan => EvalValue::bool(lhs < rhs),
        Operator::LesserThanEquals => EvalValue::bool(lhs <= rhs),
        Operator::GreaterThan => EvalValue::bool(lhs > rhs),
        Operator::GreaterThanEquals => EvalValue::bool(lhs >= rhs),
        _ => unreachable!(),
    };

    Ok(value)
}

fn eval_binary_expr(lhs: EvalValue, op: &Operator, rhs: EvalValue, span: Span) -> EvalResult {
    let value = match op {
        Operator::Addition => {
            if let EvalValue::String(lhs_val) = &lhs {
                let rhs_val = rhs.to_string();
                let val = format!("{}{}", lhs_val, rhs_val);
                EvalValue::String(val)
            } else if let EvalValue::String(rhs_val) = &rhs {
                let lhs_val = lhs.to_string();
                let val = format!("{}{}", lhs_val, rhs_val);
                EvalValue::String(val)
            } else {
                // addition on integers,
                // binder should enforce
                // this, or push runtime
                // error on mismatched
                // any type
                let lhs = lhs.get_int(span)?;
                let rhs = rhs.get_int(span)?;

                EvalValue::int(lhs + rhs)
            }
        }
        Operator::Subtraction
        | Operator::Multiplication
        | Operator::Division
        | Operator::Modulo
        | Operator::IntDivision
        | Operator::LesserThan
        | Operator::LesserThanEquals
        | Operator::GreaterThan
        | Operator::GreaterThanEquals => return eval_int_only_binexpr(lhs, op, rhs, span),
        Operator::Equality | Operator::Inequality => {
            // check if same variant
            if mem::discriminant(&lhs) != mem::discriminant(&rhs) {
                unreachable!();
            }

            // != is == negated, so compute equality and flip at the end
            let negate = matches!(op, Operator::Inequality);

            let equal = match lhs {
                EvalValue::Void => unreachable!(),
                EvalValue::Int(lhs) => rhs.get_int(span)? == lhs,
                EvalValue::Bool(lhs) => rhs.get_bool(span)? == lhs,
                EvalValue::String(lhs) => rhs.get_string(span)? == lhs,
                EvalValue::Object(_) => unreachable!(),
            };

            if negate {
                EvalValue::Bool(!equal)
            } else {
                EvalValue::Bool(equal)
            }
        }
        Operator::And | Operator::Or => {
            let lhs = lhs.get_bool(span)?;
            let rhs = rhs.get_bool(span)?;

            match op {
                Operator::And => EvalValue::bool(lhs && rhs),
                _ => EvalValue::bool(lhs || rhs),
            }
        }
        _ => {
            unreachable!("Not a binary operator")
        }
    };

    Ok(value)
}

fn eval_unary_expr(rhs_val: EvalValue, op: &Operator, span: Span) -> EvalResult {
    let value = match op {
        Operator::Not => {
            // only defined on bools
            EvalValue::Bool(!rhs_val.get_bool(span)?)
        }
        Operator::Subtraction => {
            // only defined on ints
            EvalValue::Int(-rhs_val.get_int(span)?)
        }
        _ => {
            unreachable!("Not a unary operator")
        }
    };

    Ok(value)
}

async fn eval_call_args(
    symbol: &FunctionSymbol,
    args: &Box<Vec<BoundNode>>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalResult<()> {
    let num_params = symbol.parameters.len();
    for index in 0..num_params {
        let param = &symbol.parameters[index];
        let arg = &args[index];

        let symbol = &param.symbol;
        let value = eval_rec(arg, info.clone(), io).await?;
        info.lock().unwrap().heap.assign_var(symbol, value);
    }

    Ok(())
}

async fn eval_for_loop(
    iterator: &VariableSymbol,
    lower_bound: &BoundNode,
    upper_bound: &BoundNode,
    body: Arc<BoundNode>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalResult {
    // both bounds are evaluated once, before the first iteration, so mutating
    // a variable used in a bound inside the body cannot change the trip count
    let lower_value = eval_rec(lower_bound, info.clone(), io).await?;
    let lower_value = lower_value.get_int(lower_bound.span)?;

    let upper_value = eval_rec(upper_bound, info.clone(), io).await?;
    let upper_value = upper_value.get_int(upper_bound.span)?;

    // the spec's from/to loop includes its upper bound: `loop COUNT from 0 to 5`
    // runs for 0..=5, which is what makes `from 0 to COUNT-1` visit COUNT items
    for index in lower_value..=upper_value {
        let index_val = EvalValue::Int(index);
        info.lock().unwrap().heap.assign_var(iterator, index_val);

        // a return or an error in the body leaves the loop as well
        eval_rec(&body, info.clone(), io).await?;
    }

    Ok(EvalValue::void())
}

/// `loop until X` is the inverse of `loop while X`: the condition is tested
/// before each iteration and the loop runs while it is false.
async fn eval_until_loop(
    expr: &BoundNode,
    body: Arc<BoundNode>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalResult {
    loop {
        let condition = eval_rec(expr, info.clone(), io).await?;
        if condition.get_bool(expr.span)? {
            break;
        }

        eval_rec(&body, info.clone(), io).await?;
    }

    Ok(EvalValue::void())
}

async fn eval_while_loop(
    expr: &BoundNode,
    body: Arc<BoundNode>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalResult {
    loop {
        let condition = eval_rec(expr, info.clone(), io).await?;
        if !condition.get_bool(expr.span)? {
            break;
        }

        eval_rec(&body, info.clone(), io).await?;
    }

    Ok(EvalValue::void())
}

fn not_indexable<T>(span: Span) -> EvalResult<T> {
    runtime_error("Only arrays can be indexed", span)
}

/// Resolves the `base[index]` operands shared by reads and writes. The binder
/// only lets arrays through, or `Any`, which has to be checked here instead.
fn eval_index_operands(
    base: EvalValue,
    index: EvalValue,
    span: Span,
) -> EvalResult<(Arc<tokio::sync::Mutex<ObjectState>>, i64)> {
    let index = index.get_int(span)?;

    let EvalValue::Object(state) = base else {
        return not_indexable(span);
    };

    Ok((state, index))
}

async fn eval_index_expr(base: EvalValue, index: EvalValue, span: Span) -> EvalResult {
    let (state, index) = eval_index_operands(base, index, span)?;

    let state = state.lock().await;
    match &*state {
        ObjectState::Array(array) => get_element(array, index, span),
        _ => not_indexable(span),
    }
}

async fn eval_index_assignment_expr(
    base: EvalValue,
    index: EvalValue,
    value: EvalValue,
    span: Span,
) -> EvalResult {
    let (state, index) = eval_index_operands(base, index, span)?;

    let mut state = state.lock().await;
    match &mut *state {
        ObjectState::Array(array) => set_element(array, index, value, span),
        _ => not_indexable(span),
    }
}

#[async_recursion]
async fn eval_rec(node: &BoundNode, info: Arc<Mutex<EvalInfo>>, io: &mut impl EvalIO) -> EvalResult {
    let val = match &node.kind {
        BoundNodeKind::Module { block } => eval_rec(&block, info, io).await?,
        BoundNodeKind::Block { children } => {
            for child in children.iter() {
                eval_rec(child, info.clone(), io).await?;
            }

            EvalValue::void()
        }
        BoundNodeKind::AssignmentExpression { symbol, value } => {
            let value = eval_rec(&value, info.clone(), io).await?;
            info.lock().unwrap().heap.assign_var(symbol, value.clone());

            value
        }
        BoundNodeKind::ReferenceExpression(reference) => {
            info.lock().unwrap().heap.get_var(&reference)
        }
        BoundNodeKind::BinaryExpression { lhs, op, rhs } => {
            let lhs_val = eval_rec(&lhs, info.clone(), io).await?;
            let rhs_val = eval_rec(&rhs, info, io).await?;
            eval_binary_expr(lhs_val, op, rhs_val, node.span)?
        }
        BoundNodeKind::UnaryExpression { op, rhs } => {
            let rhs_val = eval_rec(&rhs, info, io).await?;
            eval_unary_expr(rhs_val, op, node.span)?
        }
        BoundNodeKind::NumberLiteral(num) => EvalValue::int(*num),
        BoundNodeKind::BooleanLiteral(val) => EvalValue::bool(*val),
        BoundNodeKind::StringLiteral(val) => EvalValue::string(val.clone()),
        BoundNodeKind::OutputStatement { expr } => {
            let value = eval_rec(&expr, info, io).await?;

            let value = format!("{}\n", value.to_string());
            io.output(value).await;

            EvalValue::void()
        }
        BoundNodeKind::ReturnStatement { expr } => {
            let val = if let Some(expr) = expr {
                eval_rec(&expr, info, io).await?
            } else {
                EvalValue::void()
            };

            // not a value but a signal: it unwinds out of every block and loop
            // until the call running this function catches it
            return Err(Signal::Return(val));
        }
        BoundNodeKind::IfStatement {
            condition,
            block,
            else_block,
        } => {
            let cond_value = eval_rec(&condition, info.clone(), io).await?;

            if cond_value.get_bool(condition.span)? {
                eval_rec(&block, info, io).await?;
            } else if let Some(else_block) = else_block {
                eval_rec(else_block, info, io).await?;
            }

            EvalValue::void()
        }
        BoundNodeKind::FunctionDeclaration { symbol, block } => {
            info.lock()
                .unwrap()
                .heap
                .declare_func(symbol, block.clone());

            EvalValue::void()
        }
        BoundNodeKind::BoundCallExpression { symbol, args } => {
            eval_call_args(symbol, args, info.clone(), io).await?;

            let builtin_eval = eval_builtin::try_eval_builtin(symbol, info.clone(), io).await;
            match builtin_eval {
                Some(val) => val,
                None => {
                    // no need to clear arguments after executing the block
                    let body = info.lock().unwrap().heap.get_func(symbol);

                    // the call is where a return stops. an error keeps going
                    match eval_rec(&body, info.clone(), io).await {
                        Ok(_) => EvalValue::void(),
                        Err(Signal::Return(ret_value)) => ret_value,
                        Err(error) => return Err(error),
                    }
                }
            }
        }
        BoundNodeKind::ObjectExpression => {
            let node_type = node.node_type.clone();
            let object = get_object_state(node_type);

            EvalValue::Object(Arc::new(tokio::sync::Mutex::new(object)))
        }
        BoundNodeKind::IndexExpression { base, index } => {
            let base_value = eval_rec(&base, info.clone(), io).await?;
            let index_value = eval_rec(&index, info, io).await?;
            eval_index_expr(base_value, index_value, node.span).await?
        }
        BoundNodeKind::IndexAssignmentExpression { base, index, value } => {
            let base_value = eval_rec(&base, info.clone(), io).await?;
            let index_value = eval_rec(&index, info.clone(), io).await?;
            let value = eval_rec(&value, info, io).await?;
            eval_index_assignment_expr(base_value, index_value, value, node.span).await?
        }
        BoundNodeKind::ObjectMemberExpression { base, next } => {
            let base_value = eval_rec(&base, info.clone(), io).await?;

            // next should either be a reference or a call ;D
            // values are also objects, but they don't hold state?
            match &next.kind {
                BoundNodeKind::BoundCallExpression { symbol, args } => {
                    eval_call_args(&symbol, &args, info.clone(), io).await?;
                    eval_type_method(base_value, symbol, info, node.span).await?
                }
                _ => unreachable!(),
            }
        }
        BoundNodeKind::ForLoop {
            iterator,
            lower_bound,
            upper_bound,
            block,
        } => {
            eval_for_loop(
                iterator,
                lower_bound,
                upper_bound,
                block.clone(),
                info,
                io,
            )
            .await?
        }
        BoundNodeKind::UntilLoop { expr, block } => {
            eval_until_loop(expr, block.clone(), info, io).await?
        }
        BoundNodeKind::WhileLoop { expr, block } => {
            eval_while_loop(expr, block.clone(), info, io).await?
        }
    };

    Ok(val)
}

pub async fn eval(root: &BoundNode, io: &mut impl EvalIO) {
    let heap = EvalHeap::new();
    let info = EvalInfo { heap: heap };

    // an error stops the program and is reported here, once. a return outside
    // any function also stops it, the same as reaching the end.
    if let Err(Signal::Error(error)) = eval_rec(root, Arc::new(Mutex::new(info)), io).await {
        io.runtime_error(error).await;
    }
}
