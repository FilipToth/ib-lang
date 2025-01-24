use std::{fs, io::{self, BufRead, BufReader}};

use async_trait::async_trait;
use eval::IBEval;

mod analysis;
mod eval;

struct IBEvaluator;

#[async_trait]
impl IBEval for IBEvaluator {
    fn output(&self, msg: String) {
        print!("{}", msg);
    }

    async fn input(&self) -> String {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut buffer = String::new();

        match reader.read_line(&mut buffer) {
            Ok(_) => buffer.trim().to_string(),
            Err(_) => unreachable!()
        }
    }
}

async fn parse_file() {
    let contents = fs::read_to_string("test.ib").unwrap();
    let result = analysis::analyze(contents);
    result.errors.report();

    let Some(root) = &result.root else {
        return;
    };

    // evaluate
    eval::evaluator::eval(root, &mut IBEvaluator).await;
}

#[tokio::main]
async fn main() {
    parse_file().await;
}
