use tokio::{net::TcpStream, sync::broadcast};
use tokio_tungstenite::WebSocketStream;
use axum::{extract::ws::{Message, WebSocket, WebSocketUpgrade}, Extension};
use futures_util::StreamExt;

use crate::Broadcaster;

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    Extension(tx): Extension<Broadcaster>
) -> impl axum::response::IntoResponse {
    // need query params...
    println!("upgrading ws request...");
    ws.on_upgrade(|socket| handle_ws_socket(socket, tx))
}

async fn handle_ws_socket(
    mut socket: WebSocket,
    tx: Broadcaster,
) {
    let mut rx = tx.subscribe();
    loop {
        tokio::select! {
            // wait for message received
            Some(msg) = socket.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("recv: {}", text);
                        if tx.send(text).is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_reason)) => {
                        println!("ws closed");
                    }
                    Err(e) => {
                        println!("ws error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // send messages
            Ok(msg) = rx.recv() => {
                let msg = Message::Text(msg);
                if socket.send(msg).await.is_err() {
                    println!("client disconnected");
                    break;
                }
            }
        }
    }
}