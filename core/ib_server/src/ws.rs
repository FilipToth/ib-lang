use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    Extension,
};
use futures_util::{lock::Mutex, StreamExt};
use ibc::eval::{evaluator, EvalIO};
use serde::{Deserialize, Serialize, Serializer};

use crate::{Broadcaster, Diagnostic};

#[derive(Debug, Clone, Copy)]
enum WebsocketMessageKind {
    Execute = 0,
    Output = 1,
    Input = 2,
}

impl<'de> Deserialize<'de> for WebsocketMessageKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(WebsocketMessageKind::Execute),
            1 => Ok(WebsocketMessageKind::Output),
            2 => Ok(WebsocketMessageKind::Input),
            _ => Err(serde::de::Error::custom(format!(
                "{} is an invalid value for WebSocketMessageKind",
                value
            )))
        }
    }
}

impl Serialize for WebsocketMessageKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer {
        serializer.serialize_u8(*self as u8)
    }
}



#[derive(Serialize, Deserialize)]
struct WebsocketMessage {
    kind: WebsocketMessageKind,
    payload: String,
}

struct WebSocketEvaluator {
    socket: Arc<Mutex<WebSocket>>,
}

#[async_trait]
impl EvalIO for WebSocketEvaluator {
    async fn output(&self, msg: String) {
        let msg = WebsocketMessage {
            kind: WebsocketMessageKind::Output,
            payload: msg,
        };

        let msg_raw = serde_json::to_string(&msg).unwrap();
        let msg = Message::Text(msg_raw);

        let mut socket = self.socket.lock().await;
        let _ = socket.send(msg).await;
    }

    async fn input(&self) -> String {
        let msg = WebsocketMessage {
            kind: WebsocketMessageKind::Input,
            payload: "".to_string(),
        };

        let mut socket = self.socket.lock().await;
        let msg_raw = serde_json::to_string(&msg).unwrap();
        match send_await_resp(&mut socket, msg_raw).await {
            Ok(msg) => match msg {
                Some(msg) => msg,
                None => unreachable!(),
            },
            Err(_) => unreachable!(),
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

    let mut io = WebSocketEvaluator { socket: socket };
    evaluator::eval(&root, &mut io).await;
}

async fn handle_message(msg: String, socket: Arc<Mutex<WebSocket>>) {
    println!("{}", msg);
    let msg: WebsocketMessage = match serde_json::from_str(&msg) {
        Ok(msg) => msg,
        Err(_) => {
            unreachable!()
        },
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

async fn handle_ws_socket(socket: WebSocket, tx: Broadcaster) {
    let socket = Arc::new(Mutex::new(socket));
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            // Wait for message received
            Some(msg) = async {
                let mut socket_lock = socket.lock().await;
                socket_lock.next().await
            } => {
                match msg {
                    Ok(Message::Text(text)) => {
                        println!("recv: {}", &text);
                        handle_message(text.clone(), Arc::clone(&socket)).await;

                        if tx.send(text).is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_reason)) => {
                        println!("ws closed");
                        break;
                    }
                    Err(e) => {
                        println!("ws error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            Ok(msg) = rx.recv() => {
                let msg = Message::Text(msg);
                if socket.lock().await.send(msg).await.is_err() {
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
