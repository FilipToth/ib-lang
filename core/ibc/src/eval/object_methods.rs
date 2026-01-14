use std::sync::{Arc, Mutex};

use crate::analysis::{
    binding::{
        symbols::FunctionSymbol,
        types::{ArrayState, CollectionState, ObjectState},
    },
    span::Span,
};

use super::evaluator::{runtime_error, EvalInfo, EvalResult, EvalValue};

fn out_of_bounds<T>(index: i64, len: usize, span: Span) -> EvalResult<T> {
    let msg = format!(
        "Index {} is out of bounds for an array of length {}",
        index, len
    );

    runtime_error(msg, span)
}

/// Reads `index` out of an array, raising a runtime error instead of panicking
/// when it falls outside. Shared by `A[I]` and `A.get(I)`.
pub fn get_element(state: &ArrayState, index: i64, span: Span) -> EvalResult {
    let element = usize::try_from(index)
        .ok()
        .and_then(|i| state.internal.get(i));

    match element {
        Some(v) => Ok(v.clone()),
        None => out_of_bounds(index, state.internal.len(), span),
    }
}

/// Writes `value` at `index`. Arrays have no declared size and start out
/// empty, so writing one past the end appends -- that is what lets the spec's
/// `LIST[COUNT] = DATA` fill an array. Anything further out is an error.
pub fn set_element(state: &mut ArrayState, index: i64, value: EvalValue, span: Span) -> EvalResult {
    let len = state.internal.len();

    match usize::try_from(index) {
        Ok(i) if i < len => state.internal[i] = value.clone(),
        Ok(i) if i == len => state.internal.push(value.clone()),
        _ => return out_of_bounds(index, len, span),
    }

    Ok(value)
}

fn execute_array_method(
    state: &mut ArrayState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    span: Span,
) -> EvalResult {
    match symbol.identifier.as_str() {
        "push" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.push(item_value);
            Ok(EvalValue::Void)
        }
        "get" => {
            let index = &symbol.parameters[0].symbol;
            let index_value = info.lock().unwrap().heap.get_var(index);

            let index_value = match index_value {
                EvalValue::Int(i) => i,
                _ => {
                    let msg = "Attempting to call Array.get with a non-integer index";
                    return runtime_error(msg, span);
                }
            };

            get_element(state, index_value, span)
        }
        "len" => {
            let length = state.internal.len() as i64;
            Ok(EvalValue::Int(length))
        }
        "isEmpty" => {
            let res = state.internal.len() == 0;
            Ok(EvalValue::Bool(res))
        }
        _ => unimplemented!(),
    }
}

fn execute_collection_method(
    state: &mut CollectionState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    span: Span,
) -> EvalResult {
    match symbol.identifier.as_str() {
        "hasNext" => {
            let index = state.head.clone();
            let res = state.internal.get(index).is_some();
            Ok(EvalValue::Bool(res))
        }
        "getNext" => {
            let index = state.head.clone();
            match state.internal.get(index) {
                Some(v) => {
                    state.head += 1;
                    Ok(v.clone())
                }
                None => runtime_error("Getting item from an empty collection", span),
            }
        }
        "resetNext" => {
            state.head = 0;
            Ok(EvalValue::Void)
        }
        "addItem" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.push(item_value);
            Ok(EvalValue::Void)
        }
        "isEmpty" => {
            let res = state.internal.len() == 0;
            Ok(EvalValue::Bool(res))
        }
        _ => unimplemented!(),
    }
}

fn execute_stack_method(
    state: &mut ArrayState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    span: Span,
) -> EvalResult {
    match symbol.identifier.as_str() {
        "push" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.push(item_value);
            Ok(EvalValue::Void)
        }
        "pop" => match state.internal.pop() {
            Some(v) => Ok(v),
            None => runtime_error("Popping element from an empty stack", span),
        },
        "isEmpty" => {
            let res = state.internal.len() == 0;
            Ok(EvalValue::Bool(res))
        }
        _ => unimplemented!(),
    }
}

fn execute_queue_method(
    state: &mut ArrayState,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    span: Span,
) -> EvalResult {
    match symbol.identifier.as_str() {
        "enqueue" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.lock().unwrap().heap.get_var(item);

            state.internal.insert(0, item_value);
            Ok(EvalValue::Void)
        }
        "dequeue" => match state.internal.pop() {
            Some(v) => Ok(v),
            None => runtime_error("Dequeuing from an empty queue", span),
        },
        "isEmpty" => {
            let res = state.internal.len() == 0;
            Ok(EvalValue::Bool(res))
        }
        _ => unimplemented!(),
    }
}

async fn execute_object_method(
    state: Arc<tokio::sync::Mutex<ObjectState>>,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    span: Span,
) -> EvalResult {
    let mut state = state.lock().await;
    match &mut *state {
        ObjectState::Array(state) => execute_array_method(state, symbol, info, span),
        ObjectState::Collection(state) => execute_collection_method(state, symbol, info, span),
        ObjectState::Stack(state) => execute_stack_method(state, symbol, info, span),
        ObjectState::Queue(state) => execute_queue_method(state, symbol, info, span),
    }
}

pub async fn eval_type_method(
    value: EvalValue,
    symbol: &FunctionSymbol,
    info: Arc<Mutex<EvalInfo>>,
    span: Span,
) -> EvalResult {
    match value {
        EvalValue::Object(state) => execute_object_method(state, symbol, info, span).await,
        _ => unimplemented!(),
    }
}
