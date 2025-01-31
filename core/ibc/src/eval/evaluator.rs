use std::{
    collections::HashMap,
    mem,
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
};

use super::{eval_builtin, object_methods::eval_type_method, EvalIO};

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
    Object(Arc<Mutex<ObjectState>>),
    // used to return in
    // the eval rec function
    Return(Box<EvalValue>),
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
            EvalValue::Return(_) => unreachable!(),
        }
    }

    fn force_get_int(&self) -> i64 {
        let EvalValue::Int(val) = self else {
            unreachable!()
        };

        val.clone()
    }

    fn force_get_bool(&self) -> bool {
        let EvalValue::Bool(val) = self else {
            unreachable!()
        };

        val.clone()
    }

    fn force_get_string(&self) -> String {
        let EvalValue::String(val) = self else {
            unreachable!()
        };

        val.clone()
    }
}

fn eval_binary_expr(lhs: EvalValue, op: &Operator, rhs: EvalValue) -> EvalValue {
    match op {
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
                // binder should enforce this
                let lhs = lhs.force_get_int();
                let rhs = rhs.force_get_int();
                EvalValue::int(lhs + rhs)
            }
        }
        Operator::Subtraction => {
            // subtraction is only defined on integers
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::int(lhs - rhs)
        }
        Operator::Multiplication => {
            // multiplication is only defined on integers
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::int(lhs * rhs)
        }
        Operator::Division => {
            // division is only defined on integers
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::int(lhs / rhs)
        }
        Operator::Equality => {
            // check if same variant
            if mem::discriminant(&lhs) != mem::discriminant(&rhs) {
                unreachable!();
            }

            match lhs {
                EvalValue::Void => unreachable!(),
                EvalValue::Int(lhs) => {
                    let rhs = rhs.force_get_int();
                    EvalValue::Bool(rhs == lhs)
                }
                EvalValue::Bool(lhs) => {
                    let rhs = rhs.force_get_bool();
                    EvalValue::Bool(rhs == lhs)
                }
                EvalValue::String(lhs) => {
                    let rhs = rhs.force_get_string();
                    EvalValue::Bool(rhs == lhs)
                }
                EvalValue::Object(_) => unreachable!(),
                EvalValue::Return(_) => unreachable!(),
            }
        }
        Operator::LesserThan => {
            // rhs and lhs are ints
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::bool(lhs < rhs)
        }
        Operator::GreaterThan => {
            // rhs and lhs are ints
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::bool(lhs > rhs)
        }
        _ => {
            unreachable!("Not a binary operator")
        }
    }
}

fn eval_unary_expr(rhs_val: EvalValue, op: &Operator) -> EvalValue {
    match op {
        Operator::Not => {
            // only defined on bools
            let rhs = rhs_val.force_get_bool();
            EvalValue::Bool(!rhs)
        }
        Operator::Subtraction => {
            // only defined on ints
            let rhs = rhs_val.force_get_int();
            EvalValue::Int(-rhs)
        }
        _ => {
            unreachable!("Not a unary operator")
        }
    }
}

async fn eval_call_args(
    symbol: &FunctionSymbol,
    args: &Box<Vec<BoundNode>>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) {
    let num_params = symbol.parameters.len();
    for index in 0..num_params {
        let param = &symbol.parameters[index];
        let arg = &args[index];

        let symbol = &param.symbol;
        let value = eval_rec(arg, info.clone(), io).await;
        info.lock().unwrap().heap.assign_var(symbol, value);
    }
}

async fn eval_for_loop(
    iterator: &VariableSymbol,
    lower_bound: usize,
    upper_bound: usize,
    body: Arc<BoundNode>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    for index in lower_bound..upper_bound {
        let index_val = EvalValue::Int(index as i64);
        info.lock().unwrap().heap.assign_var(iterator, index_val);
        eval_rec(&body, info.clone(), io).await;
    }

    EvalValue::void()
}

async fn eval_while_loop(
    expr: &BoundNode,
    body: Arc<BoundNode>,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    loop {
        let expr_eval = eval_rec(expr, info.clone(), io).await;
        let EvalValue::Bool(expr_eval) = expr_eval else {
            unreachable!()
        };

        if !expr_eval {
            break;
        }

        eval_rec(&body, info.clone(), io).await;
    }

    EvalValue::void()
}

#[async_recursion]
async fn eval_rec(node: &BoundNode, info: Arc<Mutex<EvalInfo>>, io: &mut impl EvalIO) -> EvalValue {
    let val = match &node.kind {
        BoundNodeKind::Module { block } => eval_rec(&block, info, io).await,
        BoundNodeKind::Block { children } => {
            for child in children.iter() {
                let val = eval_rec(child, info.clone(), io).await;
                if let EvalValue::Return(_) = &val {
                    return val;
                }
            }

            EvalValue::void()
        }
        BoundNodeKind::AssignmentExpression { symbol, value } => {
            let value = eval_rec(&value, info.clone(), io).await;
            info.lock().unwrap().heap.assign_var(symbol, value.clone());

            value
        }
        BoundNodeKind::ReferenceExpression(reference) => {
            info.lock().unwrap().heap.get_var(&reference)
        }
        BoundNodeKind::BinaryExpression { lhs, op, rhs } => {
            let lhs_val = eval_rec(&lhs, info.clone(), io).await;
            let rhs_val = eval_rec(&rhs, info, io).await;
            eval_binary_expr(lhs_val, op, rhs_val)
        }
        BoundNodeKind::UnaryExpression { op, rhs } => {
            let rhs_val = eval_rec(&rhs, info, io).await;
            eval_unary_expr(rhs_val, op)
        }
        BoundNodeKind::NumberLiteral(num) => EvalValue::int(*num),
        BoundNodeKind::BooleanLiteral(val) => EvalValue::bool(*val),
        BoundNodeKind::StringLiteral(val) => EvalValue::string(val.clone()),
        BoundNodeKind::OutputStatement { expr } => {
            let value = eval_rec(&expr, info, io).await;

            let value = format!("{}\n", value.to_string());
            io.output(value).await;

            EvalValue::void()
        }
        BoundNodeKind::ReturnStatement { expr } => {
            // create special return value
            let val = if let Some(expr) = expr {
                eval_rec(&expr, info, io).await
            } else {
                EvalValue::void()
            };

            EvalValue::Return(Box::new(val))
        }
        BoundNodeKind::IfStatement {
            condition,
            block,
            else_block,
        } => {
            let cond_value = eval_rec(&condition, info.clone(), io)
                .await
                .force_get_bool();
            let value = if cond_value {
                eval_rec(&block, info, io).await
            } else if let Some(else_block) = else_block {
                eval_rec(else_block, info, io).await
            } else {
                EvalValue::void()
            };

            match &value {
                EvalValue::Return(_) => value,
                _ => EvalValue::void(),
            }
        }
        BoundNodeKind::FunctionDeclaration { symbol, block } => {
            info.lock()
                .unwrap()
                .heap
                .declare_func(symbol, block.clone());

            EvalValue::void()
        }
        BoundNodeKind::BoundCallExpression { symbol, args } => {
            eval_call_args(symbol, args, info.clone(), io).await;

            let builtin_eval = eval_builtin::try_eval_builtin(symbol, info.clone(), io).await;
            match builtin_eval {
                Some(val) => val,
                None => {
                    // no need to clear arguments after executing the block
                    let body = info.lock().unwrap().heap.get_func(symbol);
                    let ret_value = eval_rec(&body, info.clone(), io).await;

                    match ret_value {
                        EvalValue::Void => EvalValue::void(),
                        EvalValue::Return(ret_value) => ret_value.as_ref().clone(),
                        _ => unreachable!(),
                    }
                }
            }
        }
        BoundNodeKind::ObjectExpression => {
            let node_type = node.node_type.clone();
            let object = get_object_state(node_type);

            EvalValue::Object(Arc::new(Mutex::new(object)))
        }
        BoundNodeKind::ObjectMemberExpression { base, next } => {
            let base_value = eval_rec(&base, info.clone(), io).await;

            // next should either be a reference or a call ;D
            // values are also objects, but they don't hold state?
            match &next.kind {
                BoundNodeKind::BoundCallExpression { symbol, args } => {
                    eval_call_args(&symbol, &args, info.clone(), io).await;
                    eval_type_method(base_value, symbol, info)
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
                lower_bound.clone(),
                upper_bound.clone(),
                block.clone(),
                info,
                io,
            )
            .await
        }
        BoundNodeKind::WhileLoop { expr, block } => {
            eval_while_loop(expr, block.clone(), info, io).await
        }
    };

    val
}

pub async fn eval(root: &BoundNode, io: &mut impl EvalIO) {
    let heap = EvalHeap::new();
    let info = EvalInfo { heap: heap };

    eval_rec(root, Arc::new(Mutex::new(info)), io).await;
}
