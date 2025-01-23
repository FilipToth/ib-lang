use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    Extension,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{Broadcaster, Diagnostic, RunResult};

#[derive(Serialize, Deserialize)]
enum WebsocketMessageKind {
    Execute,
    Output,
    Input,
}

#[derive(Serialize, Deserialize)]
struct WebsocketMessage {
    kind: WebsocketMessageKind,
    payload: String,
}

async fn send_await_resp(
    socket: &mut WebSocket,
    msg: String,
) -> Result<Option<String>, axum::Error> {
    let msg = Message::Text(msg);
    let _ = socket.send(msg).await;

    // await for a response
    match socket.next().await {
        Some(Ok(Message::Text(resp))) => Ok(Some(resp)),
        Some(Err(e)) => Err(e),
        Some(_) => Ok(None),
        None => Ok(None),
    }
}

async fn execute(body: String, socket: &mut WebSocket) {
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
        return;
    };

    let input_closure = || async {
        let input_request = WebsocketMessage {
            kind: WebsocketMessageKind::Input,
            payload: "".to_string()
        };

        let raw = serde_json::to_string(&input_request).unwrap();
        send_await_resp(socket, raw).await;
        "".to_string()
    };

    // let output = ibc::eval::eval(&root, input_closure).await;
    // let result = RunResult::new(diagnostics, output);
}

async fn handle_message(msg: String, socket: &mut WebSocket) {
    let msg: WebsocketMessage = match serde_json::from_str(&msg) {
        Ok(msg) => msg,
        Err(_) => unreachable!(),
    };

    match msg.kind {
        WebsocketMessageKind::Execute => {
            // start execution
            execute(msg.payload, socket).await;
        }
        WebsocketMessageKind::Input => {
            // input
        }
        WebsocketMessageKind::Output => {}
    };
}

async fn handle_ws_socket(mut socket: WebSocket, tx: Broadcaster) {
    let mut rx = tx.subscribe();
    loop {
        tokio::select! {
            // wait for message received
            Some(msg) = socket.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("recv: {}", &text);
                        handle_message(text.clone(), &mut socket).await;

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

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    Extension(tx): Extension<Broadcaster>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_ws_socket(socket, tx))
}
