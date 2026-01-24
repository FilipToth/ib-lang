use std::{
    collections::HashMap,
    fmt, mem,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
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

/// How deep calls may nest before the program is stopped. Without a limit a
/// runaway recursion takes the whole process down with the native stack, since
/// evaluating a call means recursing in rust too.
///
/// A call costs native stack in proportion to how involved its body is, so this
/// only holds together alongside `EVAL_STACK_SIZE`: the hosts run evaluation on
/// a stack big enough for this many calls of a heavy function.
pub const MAX_CALL_DEPTH: usize = 500;

/// The native stack evaluation needs to reach `MAX_CALL_DEPTH`. Thread stacks
/// are reserved address space, not memory in use, so asking for room is cheap.
pub const EVAL_STACK_SIZE: usize = 256 * 1024 * 1024;

/// What one run of a program may spend before it is stopped.
///
/// Neither of these is a limit anything written by hand comes near. They are
/// here so a runaway program -- an accidental infinite loop is the usual one --
/// stops itself rather than taking the host down with it, which on a small box
/// means every other program running on it as well.
#[derive(Clone, Copy)]
pub struct EvalLimits {
    /// Statements and turns of a loop the run may execute. This measures work
    /// done rather than time passed, so a program is not charged for the time
    /// it sits waiting for somebody to answer an `input()`.
    pub steps: u64,
    /// Elements the run may add to collections.
    ///
    /// Counted over the whole run rather than at any one moment: nothing
    /// tracks a collection being dropped, so removing an element does not give
    /// the budget back. That makes the count an over-estimate of what is held
    /// at once, which is the safe direction to be wrong in.
    pub elements: u64,
}

impl Default for EvalLimits {
    fn default() -> Self {
        EvalLimits {
            // measured at about eight seconds of solid work
            steps: 50_000_000,
            // measured at a peak of about 60MB held
            elements: 2_000_000,
        }
    }
}

/// Lets whoever started a program stop it.
///
/// The evaluator checks this between statements and on every turn of a loop,
/// so a program stops between steps rather than being torn down part way
/// through one. Cloning shares the same flag.
#[derive(Clone, Default)]
pub struct CancelToken {
    stopped: Arc<AtomicBool>,
}

impl CancelToken {
    pub fn new() -> CancelToken {
        CancelToken::default()
    }

    pub fn cancel(&self) {
        self.stopped.store(true, Ordering::Relaxed);
    }

    /// Clears the flag, so the token can start another program.
    pub fn reset(&self) {
        self.stopped.store(false, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }
}

pub struct EvalInfo {
    pub heap: EvalHeap,
    depth: usize,
    cancel: CancelToken,
    limits: EvalLimits,
    steps: u64,
    elements: u64,
}

/// A declared function: its body, and the variables one run of it owns.
#[derive(Clone)]
pub struct FunctionBody {
    block: Arc<BoundNode>,
    locals: Arc<Vec<u64>>,
}

/// The values a call displaced, put back when it returns. `None` is a variable
/// that had no value yet, so leaving the call removes it again rather than
/// leaving the callee's behind.
type SavedLocals = Vec<(u64, Option<EvalValue>)>;

pub struct EvalHeap {
    // just use rust's heap to manage
    // memory, no need for us to make
    // our own heap
    variables: HashMap<u64, EvalValue>,
    functions: HashMap<u64, FunctionBody>,
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

    pub fn declare_func(
        &mut self,
        symbol: &FunctionSymbol,
        block: Arc<BoundNode>,
        locals: Arc<Vec<u64>>,
    ) {
        let id = symbol.symbol_id;
        let body = FunctionBody {
            block: block,
            locals: locals,
        };

        self.functions.insert(id, body);
    }

    /// The body of `symbol`, or `None` for a builtin, which has none.
    pub fn get_func(&self, symbol: &FunctionSymbol) -> Option<FunctionBody> {
        self.functions.get(&symbol.symbol_id).cloned()
    }

    /// Takes the values of a function's own variables out of the way, so the
    /// run that is starting cannot overwrite the one that is waiting.
    fn enter_call(&mut self, locals: &Vec<u64>) -> SavedLocals {
        let mut saved: SavedLocals = Vec::with_capacity(locals.len());

        for id in locals {
            saved.push((*id, self.variables.remove(id)));
        }

        saved
    }

    fn leave_call(&mut self, saved: SavedLocals) {
        for (id, value) in saved {
            match value {
                Some(value) => self.variables.insert(id, value),
                None => self.variables.remove(&id),
            };
        }
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
    /// Whoever started the program asked for it to stop.
    Cancelled,
}

pub type EvalResult<T = EvalValue> = Result<T, Signal>;

/// Charges one step against the run's budget, and stops the program if it has
/// been asked to stop or has done too much.
///
/// Called once for each statement and once for each turn of a loop, which is
/// what makes a loop with an empty body countable too.
fn check_step(info: &Arc<Mutex<EvalInfo>>, span: Span) -> EvalResult<()> {
    let mut lock = info.lock().unwrap();

    if lock.cancel.is_cancelled() {
        return Err(Signal::Cancelled);
    }

    lock.steps += 1;
    if lock.steps > lock.limits.steps {
        return runtime_error("The program ran for too long and was stopped", span);
    }

    Ok(())
}

/// Charges one element against the run's allocation budget. Called wherever a
/// collection grows.
pub fn charge_element(info: &Arc<Mutex<EvalInfo>>, span: Span) -> EvalResult<()> {
    let mut lock = info.lock().unwrap();

    lock.elements += 1;
    if lock.elements > lock.limits.elements {
        return runtime_error("The program stored too much data and was stopped", span);
    }

    Ok(())
}

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
            // collections print through `display`, which has to lock them
            EvalValue::Object(_) => "object".to_string(),
        }
    }

    /// How a value is shown to the user. Primitives print as themselves, and a
    /// collection prints its elements in the order they went in, `[1, 2, 3]`.
    /// Elements print the same way, so nested collections nest.
    #[async_recursion]
    pub async fn display(&self) -> String {
        let EvalValue::Object(state) = self else {
            return self.to_string();
        };

        // the elements are copied out so the collection is not held locked
        // while they are printed, which matters once they are collections too
        let elements = {
            let state = state.lock().await;
            match &*state {
                ObjectState::Array(state) => state.internal.clone(),
                ObjectState::Collection(state) => state.internal.clone(),
                ObjectState::Stack(state) => state.internal.clone(),
                // a queue keeps its front at the end of the vector, so reading
                // it backwards gives the order items were enqueued in
                ObjectState::Queue(state) => state.internal.iter().rev().cloned().collect(),
            }
        };

        let mut parts: Vec<String> = Vec::new();
        for element in elements {
            parts.push(element.display().await);
        }

        format!("[{}]", parts.join(", "))
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

async fn eval_binary_expr(
    lhs: EvalValue,
    op: &Operator,
    rhs: EvalValue,
    span: Span,
) -> EvalResult {
    let value = match op {
        Operator::Addition => {
            if let EvalValue::String(lhs_val) = &lhs {
                let rhs_val = rhs.display().await;
                let val = format!("{}{}", lhs_val, rhs_val);
                EvalValue::String(val)
            } else if let EvalValue::String(rhs_val) = &rhs {
                let lhs_val = lhs.display().await;
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

/// Evaluates a call's arguments, which are the caller's expressions and so read
/// the caller's variables. They are assigned to the parameters only once the
/// callee's frame is in place, by `assign_args`.
async fn eval_call_args(
    args: &Box<Vec<BoundNode>>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalResult<Vec<EvalValue>> {
    let mut values: Vec<EvalValue> = Vec::with_capacity(args.len());

    for arg in args.iter() {
        values.push(eval_rec(arg, info.clone(), io).await?);
    }

    Ok(values)
}

fn assign_args(symbol: &FunctionSymbol, values: Vec<EvalValue>, info: Arc<Mutex<EvalInfo>>) {
    let mut lock = info.lock().unwrap();

    for (param, value) in symbol.parameters.iter().zip(values) {
        lock.heap.assign_var(&param.symbol, value);
    }
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
        // a loop whose body is empty never reaches the block's own check
        check_step(&info, body.span)?;

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
        check_step(&info, expr.span)?;

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
        check_step(&info, expr.span)?;

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
    info: &Arc<Mutex<EvalInfo>>,
    span: Span,
) -> EvalResult {
    let (state, index) = eval_index_operands(base, index, span)?;

    let mut state = state.lock().await;
    match &mut *state {
        ObjectState::Array(array) => set_element(array, index, value, info, span),
        _ => not_indexable(span),
    }
}

/// Registers every function a block declares before any of its statements run.
///
/// This mirrors the binder, which lets a function be called from above the
/// line that declares it. Registering at the declaration statement instead
/// would leave such a call finding no body, and a missing body is otherwise
/// how a builtin is recognised.
fn declare_block_functions(children: &Vec<BoundNode>, info: &Arc<Mutex<EvalInfo>>) {
    let mut lock = info.lock().unwrap();

    for child in children {
        let BoundNodeKind::FunctionDeclaration {
            symbol,
            block,
            locals,
        } = &child.kind
        else {
            continue;
        };

        lock.heap
            .declare_func(symbol, block.clone(), Arc::new(locals.clone()));
    }
}

#[async_recursion]
async fn eval_rec(node: &BoundNode, info: Arc<Mutex<EvalInfo>>, io: &mut impl EvalIO) -> EvalResult {
    let val = match &node.kind {
        BoundNodeKind::Module { block } => eval_rec(&block, info, io).await?,
        BoundNodeKind::Block { children } => {
            declare_block_functions(children, &info);

            for child in children.iter() {
                check_step(&info, child.span)?;
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
            eval_binary_expr(lhs_val, op, rhs_val, node.span).await?
        }
        BoundNodeKind::UnaryExpression { op, rhs } => {
            let rhs_val = eval_rec(&rhs, info, io).await?;
            eval_unary_expr(rhs_val, op, node.span)?
        }
        BoundNodeKind::NumberLiteral(num) => EvalValue::int(*num),
        BoundNodeKind::BooleanLiteral(val) => EvalValue::bool(*val),
        BoundNodeKind::StringLiteral(val) => EvalValue::string(val.clone()),
        BoundNodeKind::OutputStatement { exprs } => {
            // the spec's commas join their values with nothing between them:
            // every space in its examples is written inside a string literal
            let mut line = String::new();
            for expr in exprs.iter() {
                let value = eval_rec(expr, info.clone(), io).await?;
                line.push_str(&value.display().await);
            }

            line.push('\n');
            io.output(line).await;

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
        // already registered by the block holding it, before any of that
        // block's statements ran
        BoundNodeKind::FunctionDeclaration { .. } => EvalValue::void(),
        BoundNodeKind::BoundCallExpression { symbol, args } => {
            let values = eval_call_args(args, info.clone(), io).await?;
            let body = info.lock().unwrap().heap.get_func(symbol);

            let Some(body) = body else {
                // a builtin has no body of its own to run, and nothing to save
                assign_args(symbol, values, info.clone());

                let builtin = eval_builtin::try_eval_builtin(symbol, info.clone(), io).await;
                let Some(value) = builtin else {
                    unreachable!()
                };

                return Ok(value);
            };

            // the run that is starting gets its own copy of the function's
            // variables, so a call to itself cannot overwrite the one waiting
            let saved = {
                let mut lock = info.lock().unwrap();

                lock.depth += 1;
                if lock.depth > MAX_CALL_DEPTH {
                    lock.depth -= 1;
                    return runtime_error("Recursion too deep", node.span);
                }

                lock.heap.enter_call(&body.locals)
            };

            assign_args(symbol, values, info.clone());

            // the call is where a return stops. an error keeps going
            let result = eval_rec(&body.block, info.clone(), io).await;

            {
                let mut lock = info.lock().unwrap();
                lock.heap.leave_call(saved);
                lock.depth -= 1;
            }

            match result {
                Ok(_) => EvalValue::void(),
                Err(Signal::Return(ret_value)) => ret_value,
                Err(error) => return Err(error),
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
            let value = eval_rec(&value, info.clone(), io).await?;
            eval_index_assignment_expr(base_value, index_value, value, &info, node.span).await?
        }
        BoundNodeKind::ObjectMemberExpression { base, next } => {
            let base_value = eval_rec(&base, info.clone(), io).await?;

            // next should either be a reference or a call ;D
            // values are also objects, but they don't hold state?
            match &next.kind {
                BoundNodeKind::BoundCallExpression { symbol, args } => {
                    // a type method has no frame: its parameters are declared
                    // where the call is written
                    let values = eval_call_args(&args, info.clone(), io).await?;
                    assign_args(symbol, values, info.clone());

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

/// Runs a program. `cancel` stops it early, between steps, and `limits` caps
/// what it may spend before it is stopped for running away.
pub async fn eval(
    root: &BoundNode,
    io: &mut impl EvalIO,
    cancel: CancelToken,
    limits: EvalLimits,
) {
    let heap = EvalHeap::new();
    let info = EvalInfo {
        heap: heap,
        depth: 0,
        cancel: cancel,
        limits: limits,
        steps: 0,
        elements: 0,
    };

    // an error stops the program and is reported here, once. a return outside
    // any function also stops it, the same as reaching the end, and so does
    // being cancelled -- the caller asked for that, so there is nothing to say.
    if let Err(Signal::Error(error)) = eval_rec(root, Arc::new(Mutex::new(info)), io).await {
        io.runtime_error(error).await;
    }
}
