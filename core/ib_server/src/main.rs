use std::{collections::HashMap, fs, path::Path};

use auth::auth_middleware;
use axum::{
    extract::Query,
    routing::{get, post},
    Extension, Json, Router,
};
use dotenv::dotenv;
use rusqlite::Connection;
use serde::Serialize;
use sync::{create_file, delete_file, get_files, sync_file};
use throttle::Throttle;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use ws::handle_ws;

extern crate ibc;
extern crate dotenv;

pub mod auth;
pub mod db;
pub mod sync;
pub mod throttle;
pub mod ws;

#[derive(Serialize)]
struct Diagnostic {
    message: String,
    offset_start: usize,
    offset_end: usize,
}

#[derive(Serialize)]
struct RouteSuccess {
    success: bool,
}

#[derive(Serialize, Debug)]
pub struct IbFile {
    pub id: String,
    pub filename: String,
    pub contents: String,
}

fn main() {
    // evaluation recurses, so the threads that run request handlers need more
    // stack than tokio's default
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(ibc::eval::evaluator::EVAL_STACK_SIZE)
        .enable_all()
        .build()
        .expect("cannot build the tokio runtime");

    runtime.block_on(serve());
}

async fn serve() {
    dotenv().ok();
    setup_db();

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let throttle = Throttle::new();

    let protected_router = Router::new()
        .route("/diagnostics", post(diagnostics))
        .route("/control-flow", post(control_flow))
        .route("/files", get(files))
        .route("/create", post(create_file_route))
        .route("/delete", post(delete_file_route))
        .route("/save", post(save_file_route))
        .layer(axum::middleware::from_fn(auth_middleware));

    let app = Router::new()
        .route("/wss", get(handle_ws))
        .nest("/api", protected_router)
        .layer(ServiceBuilder::new().layer(cors))
        .layer(Extension(throttle));

    println!("Listening on port 8080...");
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn diagnostics(body: String) -> Json<Vec<Diagnostic>> {
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

    Json(diagnostics)
}

#[derive(Serialize)]
struct ControlFlowGraph {
    /// The graph in Graphviz DOT, or null when the program does not compile.
    dot: Option<String>,
    /// Why it could not be drawn. Empty when `dot` is set.
    diagnostics: Vec<Diagnostic>,
}

/// Draws the control flow graph of the posted source. Unlike `/diagnostics`
/// this does not sync the file: it is a view of code the caller already has.
async fn control_flow(
    Extension(_uid): Extension<String>,
    body: String,
) -> Json<ControlFlowGraph> {
    let (errors, dot) = ibc::analysis::control_flow_graph(body);

    let mut diagnostics: Vec<Diagnostic> = vec![];
    for error in errors.errors {
        let diagnostic = Diagnostic {
            message: error.kind.format(),
            offset_start: error.span.start.char_offset,
            offset_end: error.span.end.char_offset,
        };

        diagnostics.push(diagnostic);
    }

    let graph = ControlFlowGraph {
        dot: dot,
        diagnostics: diagnostics,
    };

    Json(graph)
}

async fn files(
    Extension(uid): Extension<String>,
    _query: Query<HashMap<String, String>>,
) -> Json<Vec<IbFile>> {
    let files = get_files(uid);
    Json(files)
}

async fn create_file_route(
    Extension(uid): Extension<String>,
    query: Query<HashMap<String, String>>,
) -> Json<RouteSuccess> {
    let failed_resp = RouteSuccess { success: false };

    let id = match query.0.get("id") {
        Some(id) => id,
        None => return Json(failed_resp),
    };

    let filename = match query.0.get("filename") {
        Some(f) => f,
        None => return Json(failed_resp),
    };

    let success = create_file(uid.clone(), id.clone(), filename.clone());

    let resp = RouteSuccess { success: success };
    Json(resp)
}

async fn delete_file_route(
    Extension(uid): Extension<String>,
    query: Query<HashMap<String, String>>,
) -> Json<RouteSuccess> {
    let failed = RouteSuccess { success: false };
    let id = match query.0.get("id") {
        Some(id) => id,
        None => return Json(failed),
    };

    let success = delete_file(uid, id.clone());
    Json(RouteSuccess { success: success })
}

/// Stores the posted source as the contents of file `id`.
async fn save_file_route(
    Extension(uid): Extension<String>,
    query: Query<HashMap<String, String>>,
    body: String,
) -> Json<RouteSuccess> {
    let id = match query.0.get("id") {
        Some(id) => id,
        None => return Json(RouteSuccess { success: false }),
    };

    // missing or malformed, the save is written unguarded
    let seq = query.0.get("seq").and_then(|s| s.parse::<u64>().ok());

    let success = sync_file(uid, id.clone(), body, seq);
    Json(RouteSuccess { success: success })
}

fn setup_db() {
    let dir_path = Path::new("./data/");
    if !dir_path.exists() {
        fs::create_dir(dir_path).unwrap();
    }

    let path = dir_path.join("files.db");
    let conn = Connection::open(path).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (id TEXT PRIMARY KEY, uid TEXT, filename TEXT)",
        [],
    )
    .unwrap();

    conn.close().unwrap();
}
