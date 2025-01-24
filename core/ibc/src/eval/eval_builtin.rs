use std::sync::{Arc, Mutex};

use crate::analysis::binding::symbols::FunctionSymbol;
use super::{evaluator::{EvalInfo, EvalValue}, IBEval};

pub async fn try_eval_builtin(symbol: &FunctionSymbol, _info: Arc<Mutex<EvalInfo>>, ev: &mut impl IBEval) -> Option<EvalValue> {
    match symbol.identifier.as_str() {
        "input" => {
            let input = ev.input().await;
            let val = EvalValue::String(input);
            Some(val)
        },
        _ => None
    }
}