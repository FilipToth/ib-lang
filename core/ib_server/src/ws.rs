use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use futures_util::{lock::Mutex, StreamExt};
use ibc::eval::{
    evaluator::{self, RuntimeError},
    EvalIO,
};
use serde::{Deserialize, Serialize, Serializer};

use crate::auth::verify_jwt;
use crate::db::get_filename_uid;
use crate::throttle::{ConnectionGuard, Throttle};

#[derive(Debug, Clone, Copy)]
enum WebsocketMessageKind {
    Execute = 0,
    Output = 1,
    Input = 2,
    RuntimeError = 3,
    /// The program did not pass analysis, so it was not run at all.
    AnalysisError = 4,
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
            3 => Ok(WebsocketMessageKind::RuntimeError),
            4 => Ok(WebsocketMessageKind::AnalysisError),
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
    /// Set on Execute requests so the server can verify the caller owns the
    /// file. Absent on the messages the server sends back.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file_id: Option<String>,
    /// Where a runtime error happened, as character offsets into the source,
    /// so the editor can highlight it. Only set on RuntimeError messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    offset_start: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    offset_end: Option<usize>,
}

struct WebSocketEvaluator {
    socket: Arc<Mutex<WebSocket>>,
}

#[async_trait]
impl EvalIO for WebSocketEvaluator {
    async fn output(&self, output_msg: String) {
        let msg = WebsocketMessage {
            kind: WebsocketMessageKind::Output,
            payload: output_msg,
            file_id: None,
            offset_start: None,
            offset_end: None,
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
            file_id: None,
            offset_start: None,
            offset_end: None,
        };

        let mut socket = self.socket.lock().await;
        let msg_raw = serde_json::to_string(&msg).unwrap();
        match send_await_resp(&mut socket, msg_raw).await {
            Ok(msg) => match msg {
                Some(msg) => {
                    let msg: WebsocketMessage = match serde_json::from_str(&msg) {
                        Ok(msg) => msg,
                        Err(_) => unreachable!()
                    };

                    msg.payload
                },
                None => unreachable!(),
            },
            Err(_) => unreachable!(),
        }
    }

    async fn runtime_error(&self, error: RuntimeError) {
        let msg = WebsocketMessage {
            kind: WebsocketMessageKind::RuntimeError,
            payload: error.to_string(),
            file_id: None,
            offset_start: Some(error.span.start.char_offset),
            offset_end: Some(error.span.end.char_offset),
        };

        let msg_raw = serde_json::to_string(&msg).unwrap();
        let msg = Message::Text(msg_raw);

        let mut socket = self.socket.lock().await;
        let _ = socket.send(msg).await;
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

/// Sends a RuntimeError frame to the client. Used for refusals that are the
/// caller's fault, so the IDE can show a message instead of the socket just
/// dropping.
async fn refuse(socket: &Arc<Mutex<WebSocket>>, reason: &str) {
    let msg = WebsocketMessage {
        kind: WebsocketMessageKind::RuntimeError,
        payload: reason.to_string(),
        file_id: None,
        offset_start: None,
        offset_end: None,
    };

    if let Ok(raw) = serde_json::to_string(&msg) {
        let mut socket = socket.lock().await;
        let _ = socket.send(Message::Text(raw)).await;
    }
}

async fn execute(body: String, socket: Arc<Mutex<WebSocket>>) {
    let result = ibc::analysis::analyze(body);

    // a program with errors is not run. the editor already underlines them
    // through the diagnostics route, so this only has to say why nothing ran
    let Some(root) = result.runnable() else {
        let message = match result.errors.errors.first() {
            Some(error) => error.kind.format(),
            None => "the program could not be analyzed".to_string(),
        };

        let msg = WebsocketMessage {
            kind: WebsocketMessageKind::AnalysisError,
            payload: message,
            file_id: None,
            offset_start: None,
            offset_end: None,
        };

        if let Ok(raw) = serde_json::to_string(&msg) {
            let mut socket = socket.lock().await;
            let _ = socket.send(Message::Text(raw)).await;
        }

        let _ = socket.lock().await.send(Message::Close(None)).await;
        return;
    };

    let mut io = WebSocketEvaluator { socket: socket.clone() };
    evaluator::eval(root, &mut io).await;

    let _ = socket.lock().await.send(Message::Close(None)).await;
}

async fn handle_message(
    msg: String,
    socket: Arc<Mutex<WebSocket>>,
    uid: &str,
    throttle: &Throttle,
) {
    let msg: WebsocketMessage = match serde_json::from_str(&msg) {
        Ok(msg) => msg,
        Err(_) => {
            refuse(&socket, "Malformed message").await;
            return;
        }
    };

    match msg.kind {
        WebsocketMessageKind::Execute => {
            let Some(file_id) = msg.file_id else {
                refuse(&socket, "Execute request is missing a file id").await;
                return;
            };

            // the caller must own the file it is asking us to run
            let Some(file) = get_filename_uid(file_id) else {
                refuse(&socket, "No such file").await;
                return;
            };

            if file.uid != uid {
                refuse(&socket, "No such file").await;
                return;
            }

            if !throttle.try_execute(uid) {
                refuse(&socket, "Too many runs, please wait a moment").await;
                return;
            }

            execute(msg.payload, socket).await;
        }
        // server only accepts execute requests
        WebsocketMessageKind::Input => {}
        WebsocketMessageKind::Output => {}
        WebsocketMessageKind::AnalysisError | WebsocketMessageKind::RuntimeError => {}
    };
}

async fn handle_ws_socket(
    socket: WebSocket,
    uid: String,
    throttle: Throttle,
    // held for the lifetime of the connection; releases the slot on drop
    _guard: ConnectionGuard,
) {
    let socket = Arc::new(Mutex::new(socket));

    loop {
        // the lock is scoped so it is released before the message is handled
        let msg = {
            let mut socket_lock = socket.lock().await;
            socket_lock.next().await
        };

        // stream ended, the peer is gone
        let Some(msg) = msg else {
            break;
        };

        match msg {
            Ok(Message::Text(text)) => {
                handle_message(text, Arc::clone(&socket), &uid, &throttle).await;
            }
            Ok(Message::Close(_reason)) => {
                break;
            }
            Err(e) => {
                println!("ws error: {}", e);
                break;
            }
            _ => {}
        }
    }
}

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    Extension(throttle): Extension<Throttle>,
) -> Response {
    // The browser WebSocket API cannot set request headers, so the Firebase
    // ID token travels as a query parameter rather than in Authorization.
    let Some(token) = params.get("token") else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Some(uid) = verify_jwt(token).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // Reserve a slot before upgrading, so a flood is rejected as a cheap HTTP
    // response instead of becoming a live socket.
    let Some(guard) = throttle.try_connect(&uid) else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };

    ws.on_upgrade(move |socket| handle_ws_socket(socket, uid, throttle, guard))
}
