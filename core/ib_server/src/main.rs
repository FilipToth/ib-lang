use std::{collections::HashMap, fs, path::Path};

use auth::auth_middleware;
use axum::{
    extract::{DefaultBodyLimit, Query},
    routing::{get, post},
    Extension, Json, Router,
};
use dotenv::dotenv;
use rusqlite::Connection;
use serde::Serialize;
use db::count_files;
use ibc::eval::evaluator::EvalLimits;
use sync::{
    bytes_used, create_file, delete_file, get_files, oversized, rename_file, sync_file,
    MAX_FILE_BYTES, MAX_FILES_PER_USER,
};
use throttle::{Throttle, EXECUTE_WINDOW, MAX_EXECUTES_PER_WINDOW};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use ws::handle_ws;

extern crate dotenv;
extern crate ibc;

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
    /// Why it failed, worded for the person using the editor.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl RouteSuccess {
    fn ok(success: bool) -> Json<RouteSuccess> {
        Json(RouteSuccess {
            success,
            error: None,
        })
    }

    fn from(result: Result<(), String>) -> Json<RouteSuccess> {
        Json(RouteSuccess {
            success: result.is_ok(),
            error: result.err(),
        })
    }
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

    // the routes that take a program as their body. axum would otherwise
    // buffer up to its own 2MB default, and analysis is CPU bound, so the size
    // of the body is what decides how long a worker is held.
    let source_limit = DefaultBodyLimit::max(MAX_FILE_BYTES);

    let protected_router = Router::new()
        .route("/diagnostics", post(diagnostics).layer(source_limit.clone()))
        .route("/control-flow", post(control_flow).layer(source_limit))
        .route("/files", get(files))
        .route("/limits", get(limits))
        .route("/create", post(create_file_route))
        .route("/delete", post(delete_file_route))
        // twice the cap, so an oversized save is refused by the handler with a
        // message rather than by the layer with a bare 413
        .route(
            "/save",
            post(save_file_route).layer(DefaultBodyLimit::max(2 * MAX_FILE_BYTES)),
        )
        .route("/rename", post(rename_file_route))
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

async fn diagnostics(
    Extension(throttle): Extension<Throttle>,
    body: String,
) -> Json<Vec<Diagnostic>> {
    // the editor posts on every keystroke, so this is the route that decides
    // how much analysis the box is doing at any one moment
    let _slot = throttle.analysis_slot().await;
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
    Extension(throttle): Extension<Throttle>,
    body: String,
) -> Json<ControlFlowGraph> {
    let _slot = throttle.analysis_slot().await;
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

/// One thing the caller is held to: how much of it they have spent, and how
/// much they are allowed.
#[derive(Serialize)]
struct Allowance {
    used: u64,
    allowed: u64,
}

/// Everything the editor needs to show what a user may use and what they have
/// used. Without it a refusal has to explain itself from nothing, and the caps
/// a program runs under are invisible until one is hit.
#[derive(Serialize)]
struct Limits {
    files: Allowance,
    bytes: Allowance,
    /// The largest a single file may be, which `bytes.allowed` is a multiple of.
    bytes_per_file: u64,
    runs: Allowance,
    /// How long the run allowance takes to refill.
    run_window_seconds: u64,
    /// Per run rather than per account, so they are a ceiling, not a balance.
    steps_per_run: u64,
    elements_per_run: u64,
}

async fn limits(
    Extension(uid): Extension<String>,
    Extension(throttle): Extension<Throttle>,
) -> Json<Limits> {
    let per_run = EvalLimits::default();

    let limits = Limits {
        files: Allowance {
            // a count that cannot be read is shown as none used rather than
            // guessed at; the cap itself still refuses on the same failure
            used: count_files(&uid).unwrap_or(0) as u64,
            allowed: MAX_FILES_PER_USER as u64,
        },
        bytes: Allowance {
            used: bytes_used(&uid),
            allowed: (MAX_FILES_PER_USER * MAX_FILE_BYTES) as u64,
        },
        bytes_per_file: MAX_FILE_BYTES as u64,
        runs: Allowance {
            used: throttle.executes_used(&uid) as u64,
            allowed: MAX_EXECUTES_PER_WINDOW as u64,
        },
        run_window_seconds: EXECUTE_WINDOW.as_secs(),
        steps_per_run: per_run.steps,
        elements_per_run: per_run.elements,
    };

    Json(limits)
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
    let (Some(id), Some(filename)) = (query.0.get("id"), query.0.get("filename")) else {
        return RouteSuccess::ok(false);
    };

    RouteSuccess::from(create_file(uid, id.clone(), filename.clone()))
}

async fn rename_file_route(
    Extension(uid): Extension<String>,
    query: Query<HashMap<String, String>>,
) -> Json<RouteSuccess> {
    let (Some(id), Some(filename)) = (query.0.get("id"), query.0.get("filename")) else {
        return RouteSuccess::ok(false);
    };

    RouteSuccess::from(rename_file(uid, id.clone(), filename.clone()))
}

async fn delete_file_route(
    Extension(uid): Extension<String>,
    query: Query<HashMap<String, String>>,
) -> Json<RouteSuccess> {
    let Some(id) = query.0.get("id") else {
        return RouteSuccess::ok(false);
    };

    RouteSuccess::ok(delete_file(uid, id.clone()))
}

/// Stores the posted source as the contents of file `id`.
async fn save_file_route(
    Extension(uid): Extension<String>,
    query: Query<HashMap<String, String>>,
    body: String,
) -> Json<RouteSuccess> {
    let id = match query.0.get("id") {
        Some(id) => id,
        None => return RouteSuccess::ok(false),
    };

    if let Some(reason) = oversized(body.len()) {
        return RouteSuccess::from(Err(reason));
    }

    // missing or malformed, the save is written unguarded
    let seq = query.0.get("seq").and_then(|s| s.parse::<u64>().ok());

    RouteSuccess::ok(sync_file(uid, id.clone(), body, seq))
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
