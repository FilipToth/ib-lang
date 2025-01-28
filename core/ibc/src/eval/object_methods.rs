use std::sync::{Arc, Mutex};

use crate::analysis::binding::{
    symbols::FunctionSymbol,
    types::{ArrayState, CollectionState, ObjectState, QueueState, StackState},
};

use super::{
    evaluator::{EvalInfo, EvalValue},
    EvalIO,
};

async fn execute_array_method(
    state: &mut ArrayState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    match symbol.identifier.as_str() {
        "push" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.push(item_value);
            EvalValue::Void
        }
        "get" => {
            let index = &symbol.parameters[0].symbol;
            let index_value = info.lock().unwrap().heap.get_var(index);

            let index_value = match index_value {
                EvalValue::Int(i) => i as usize,
                _ => {
                    let msg = "Attempting to call Array.get with a non-integer index".to_string();
                    io.runtime_error(msg).await;
                    return EvalValue::Error;
                }
            };

            match state.internal.get(index_value) {
                Some(v) => v.clone(),
                None => {
                    let msg = "Getting element from an empty array".to_string();
                    io.runtime_error(msg);
                    return EvalValue::Error;
                }
            }
        }
        "len" => {
            let length = state.internal.len() as i64;
            EvalValue::Int(length)
        }
        _ => unimplemented!(),
    }
}

async fn execute_collection_method(
    state: &mut CollectionState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    match symbol.identifier.as_str() {
        "hasNext" => {
            let index = state.head.clone();
            let res = state.internal.get(index).is_some();
            EvalValue::Bool(res)
        }
        "getItem" => {
            let index = state.head.clone();
            match state.internal.get(index) {
                Some(v) => {
                    state.head += 1;
                    v.clone()
                }
                None => {
                    let msg = "Getting item from an empty collection".to_string();
                    io.runtime_error(msg).await;
                    return EvalValue::Error;
                }
            }
        }
        "resetNext" => {
            state.head = 0;
            EvalValue::Void
        }
        "addItem" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.push(item_value);
            EvalValue::Void
        }
        "isEmpty" => {
            let res = state.internal.len() == 0;
            EvalValue::Bool(res)
        }
        _ => unimplemented!(),
    }
}

async fn execute_stack_method(
    state: &mut StackState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    match symbol.identifier.as_str() {
        "push" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.push(item_value);
            EvalValue::Void
        }
        "pop" => match state.internal.pop() {
            Some(v) => v,
            None => {
                let msg = "Popping element from an empty stack".to_string();
                io.runtime_error(msg).await;
                return EvalValue::Error;
            }
        },
        "isEmpty" => {
            let res = state.internal.len() == 0;
            EvalValue::Bool(res)
        }
        _ => unimplemented!(),
    }
}

async fn execute_queue_method(
    state: &mut QueueState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    match symbol.identifier.as_str() {
        "enqueue" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.insert(0, item_value);
            EvalValue::Void
        }
        "dequeue" => match state.internal.pop() {
            Some(v) => v,
            None => {
                let msg = "Dequeuing from an empty array".to_string();
                io.runtime_error(msg).await;
                return EvalValue::Error;
            }
        },
        "isEmpty" => {
            let res = state.internal.len() == 0;
            EvalValue::Bool(res)
        }
        _ => unimplemented!(),
    }
}

async fn execute_object_method(
    state: Arc<Mutex<ObjectState>>,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    let mut state = state.lock().unwrap();
    match &mut *state {
        ObjectState::Array(state) => execute_array_method(state, symbol, info, io).await,
        ObjectState::Collection(state) => execute_collection_method(state, symbol, info, io).await,
        ObjectState::Stack(state) => execute_stack_method(state, symbol, info, io).await,
        ObjectState::Queue(state) => execute_queue_method(state, symbol, info, io).await,
    }
}

pub async fn eval_type_method(
    mut value: EvalValue,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> EvalValue {
    match &mut value {
        EvalValue::Object(state) => execute_object_method(state.clone(), symbol, info.clone(), io).await,
        _ => unimplemented!(),
    }
}
