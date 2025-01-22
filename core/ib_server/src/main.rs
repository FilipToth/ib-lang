use std::{collections::HashMap, fs, path::Path, sync::Arc};

use auth::auth_middleware;
use axum::{
    extract::Query,
    routing::{get, post},
    Extension, Json, Router,
};
use rusqlite::Connection;
use serde::Serialize;
use sync::{create_file, delete_file, get_files};
use tokio::{net::TcpListener, sync::broadcast};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use ws::handle_ws;

extern crate ibc;

pub mod auth;
pub mod db;
pub mod sync;
pub mod ws;

type Broadcaster = Arc<broadcast::Sender<String>>;

#[derive(Serialize)]
struct Diagnostic {
    message: String,
    offset_start: usize,
    offset_end: usize,
}

#[derive(Serialize)]
struct RunResult {
    diagnostics: Vec<Diagnostic>,
    output: String,
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
    setup_db();

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let (tx, _rx) = broadcast::channel::<String>(100);
    let tx = Arc::new(tx);

    let app = Router::new()
        .route("/execute", post(execute))
        .route("/diagnostics", post(diagnostics))
        .route("/files", get(files))
        .route("/create", post(create_file_route))
        .route("/delete", post(delete_file_route))
        .route("/ws", get(handle_ws))
        .layer(axum::middleware::from_fn(auth_middleware))
        .layer(ServiceBuilder::new().layer(cors))
        .layer(Extension(tx));

    println!("Listening on port 8080...");
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn handle_input() -> String {
    "".to_string()
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

    let output = ibc::eval::eval(&root, handle_input);
    let result = RunResult::new(diagnostics, output);
    Json(result)
}

async fn diagnostics(
    Extension(uid): Extension<String>,
    query: Query<HashMap<String, String>>,
    body: String,
) -> Json<Vec<Diagnostic>> {
    let result = ibc::analysis::analyze(body.clone());

    let id = match query.0.get("id") {
        Some(i) => i.clone(),
        None => return Json(Vec::new()),
    };

    sync::sync_file(uid, id, body);

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

    delete_file(uid, id.clone());
    let resp = RouteSuccess { success: true };
    Json(resp)
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
