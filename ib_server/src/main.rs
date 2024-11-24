use std::collections::HashMap;

use auth::auth_middleware;
use axum::{extract::Query, routing::{get, post}, Extension, Json, Router};
use serde::Serialize;
use sync::get_files;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

extern crate ibc;

pub mod sync;
pub mod auth;

#[derive(Serialize)]
struct Diagnostic {
    message: String,
    offset_start: usize,
    offset_end: usize
}

#[derive(Serialize)]
struct RunResult {
    diagnostics: Vec<Diagnostic>,
    output: String,
}

#[derive(Serialize, Debug)]
pub struct IbFile {
    pub filename: String,
    pub contents: String
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
    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any).allow_headers(Any);

    let app = Router::new()
        .route("/execute", post(execute))
        .route("/diagnostics", post(diagnostics))
        .route("/files", get(files))
        .layer(axum::middleware::from_fn(auth_middleware))
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
            offset_start: error.span.start.char_offset,
            offset_end: error.span.end.char_offset,
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

async fn diagnostics(Extension(uid): Extension<String>, query: Query<HashMap<String, String>>, body: String) -> Json<Vec<Diagnostic>> {
    let result = ibc::analysis::analyze(body.clone());

    let file = match query.0.get("file") {
        Some(f) => f.clone(),
        None => return Json(Vec::new())
    };

    sync::sync_file(uid, file, body);

    let mut diagnostics: Vec<Diagnostic> = vec![];
    let errors = result.errors.errors;

    for error in errors {
        let diagnostic = Diagnostic {
            message: error.kind.format(),
            offset_start: error.span.start.char_offset,
            offset_end: error.span.end.char_offset,
        };

        diagnostics.push(diagnostic)
    }

    Json(diagnostics)
}

async fn files(Extension(uid): Extension<String>, query: Query<HashMap<String, String>>) -> Json<Vec<IbFile>> {
    let files = get_files(uid);
    println!("{:?}", files);
    Json(files)
}
