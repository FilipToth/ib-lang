use std::sync::Arc;

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    Extension,
};
use futures_util::{lock::Mutex, StreamExt};
use ibc::eval::{evaluator, IBEval};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

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

struct WebSocketEvaluator {
    socket: Arc<Mutex<WebSocket>>
}

#[async_trait]
impl IBEval for WebSocketEvaluator {
    fn output(&self, msg: String) {
        // send message via WS... this should be async...
    }

    async fn input(&self) -> String {
        let msg = WebsocketMessage {
            kind: WebsocketMessageKind::Input,
            payload: "".to_string()
        };

        let mut socket = self.socket.lock().await;
        let msg_raw = serde_json::to_string(&msg).unwrap();
        match send_await_resp(&mut socket, msg_raw).await {
            Ok(msg) => {
                match msg {
                    Some(msg) => msg,
                    None => unreachable!()
                }
            },
            Err(_) => unreachable!()
        }
    }
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

async fn execute(body: String, socket: Arc<Mutex<WebSocket>>) {
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
        // TODO: Report Error        
        unreachable!()
    };

    let mut ev = WebSocketEvaluator {
        socket: socket
    };

    evaluator::eval(&root, &mut ev).await;

    // let output = ibc::eval::eval(&root, input_closure).await;
    // let result = RunResult::new(diagnostics, output);
}

async fn handle_message(msg: String, socket: Arc<Mutex<WebSocket>>) {
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
                        let socket = Arc::new(Mutex::new(socket));
                        handle_message(text.clone(), socket).await;

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
