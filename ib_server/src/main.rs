use axum::{routing::post, Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;

extern crate ibc;

#[derive(Serialize)]
struct RunResult {
    diagnostics: Vec<String>,
    output: String,
}

impl RunResult {
    fn new(diagnostics: Vec<String>, output: String) -> RunResult {
        RunResult {
            diagnostics: diagnostics,
            output: output,
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", post(root));

    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(body: String) -> Json<RunResult> {
    let result = ibc::analysis::analyze(body);

    let mut diagnostics: Vec<String> = vec![];
    let Some(root) = result.root else {
        let errors = result.errors.errors;
        for error in errors {
            diagnostics.push(error.format())
        }

        let result = RunResult::new(diagnostics, "".to_string());
        return Json(result);
    };

    let output = ibc::eval::eval(&root);
    let result = RunResult::new(diagnostics, output);
    Json(result)
}
