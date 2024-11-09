use axum::{routing::post, Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

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
    let cors = CorsLayer::new().allow_methods(Any).allow_origin(Any);

    let app = Router::new()
        .route("/", post(root))
        .layer(ServiceBuilder::new().layer(cors));

    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root(body: String) -> Json<RunResult> {
    let result = ibc::analysis::analyze(body);

    let mut diagnostics: Vec<String> = vec![];
    let errors = result.errors.errors;

    for error in errors {
        println!("err: {}", error.format());
        diagnostics.push(error.format())
    }

    let Some(root) = result.root else {
        let result = RunResult::new(diagnostics, "".to_string());
        return Json(result);
    };

    let output = ibc::eval::eval(&root);
    let result = RunResult::new(diagnostics, output);
    Json(result)
}
