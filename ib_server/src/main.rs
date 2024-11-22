use std::collections::HashMap;

use axum::{extract::Query, routing::post, Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

extern crate ibc;

pub mod sync;

#[derive(Serialize)]
struct Diagnostic {
    message: String,
    line: usize,
    col: usize
}

#[derive(Serialize)]
struct RunResult {
    diagnostics: Vec<Diagnostic>,
    output: String,
}

impl RunResult {
    fn new(diagnostics: Vec<Diagnostic>, output: String) -> RunResult {
        RunResult {
            diagnostics: diagnostics,
            output: output,
        }
    }
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let app = Router::new()
        .route("/execute", post(execute))
        .route("/diagnostics", post(diagnostics))
        .layer(ServiceBuilder::new().layer(cors));

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn execute(body: String) -> Json<RunResult> {
    let result = ibc::analysis::analyze(body);

    let mut diagnostics: Vec<Diagnostic> = vec![];
    let errors = result.errors.errors;

    for error in errors {
        let diagnostic = Diagnostic {
            message: error.kind.format(),
            line: error.line,
            col: error.column
        };

        diagnostics.push(diagnostic)
    }

    let Some(root) = result.root else {
        let result = RunResult::new(diagnostics, "".to_string());
        return Json(result);
    };

    let output = ibc::eval::eval(&root);
    let result = RunResult::new(diagnostics, output);
    Json(result)
}

async fn diagnostics(query: Query<HashMap<String, String>>, body: String) -> Json<Vec<Diagnostic>> {
    let result = ibc::analysis::analyze(body.clone());

    // TODO: JWT verification
    let file = match query.0.get("file") {
        Some(f) => f.clone(),
        None => return Json(Vec::new())
    };

    let uid = match query.0.get("uid") {
        Some(u) => u.clone(),
        None => return Json(Vec::new())
    };

    sync::sync_file(uid, file, body);

    let mut diagnostics: Vec<Diagnostic> = vec![];
    let errors = result.errors.errors;

    for error in errors {
        let diagnostic = Diagnostic {
            message: error.kind.format(),
            line: error.line + 1,
            col: error.column + 1
        };

        diagnostics.push(diagnostic)
    }

    Json(diagnostics)
}
