use axum::{routing::get, Router};
use tokio::net::TcpListener;

extern crate ibc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello" }));

    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}