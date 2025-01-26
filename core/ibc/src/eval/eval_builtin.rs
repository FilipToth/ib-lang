use std::sync::{Arc, Mutex};

use super::{
    evaluator::{EvalInfo, EvalValue},
    EvalIO,
};
use crate::analysis::binding::symbols::FunctionSymbol;

pub async fn try_eval_builtin(
    symbol: &FunctionSymbol,
    _info: Arc<Mutex<EvalInfo>>,
    io: &mut impl EvalIO,
) -> Option<EvalValue> {
    match symbol.identifier.as_str() {
        "input" => {
            let input = io.input().await;
            let val = EvalValue::String(input);
            Some(val)
        }
        _ => None,
    }
}
