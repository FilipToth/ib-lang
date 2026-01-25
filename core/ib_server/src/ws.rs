use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    http::{header::SEC_WEBSOCKET_PROTOCOL, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use futures_util::stream::SplitSink;
use futures_util::{lock::Mutex, SinkExt, StreamExt};
use ibc::eval::{
    evaluator::{self, CancelToken, EvalLimits, RuntimeError},
    EvalIO,
};
use tokio::sync::{mpsc, OwnedSemaphorePermit};
use tokio::task::JoinHandle;
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
    /// Asks for the running program to be stopped, and confirms that it was.
    Stop = 5,
    /// What the run that just ended spent of its budget.
    Usage = 6,
}

impl<'de> Deserialize<'de> for WebsocketMessageKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(WebsocketMessageKind::Execute),
            1 => Ok(WebsocketMessageKind::Output),
            2 => Ok(WebsocketMessageKind::Input),
            3 => Ok(WebsocketMessageKind::RuntimeError),
            4 => Ok(WebsocketMessageKind::AnalysisError),
            5 => Ok(WebsocketMessageKind::Stop),
            6 => Ok(WebsocketMessageKind::Usage),
            _ => Err(serde::de::Error::custom(format!(
                "{} is an invalid value for WebSocketMessageKind",
                value
            ))),
        }
    }
}

impl Serialize for WebsocketMessageKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
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
    /// What the run spent. Only set on Usage messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    steps: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    elements: Option<u64>,
}

/// The half of the socket that is written to. The other half stays with the
/// reader loop, so the client can still be heard while a program runs.
type Sender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

async fn send(sender: &Sender, msg: WebsocketMessage) {
    let Ok(raw) = serde_json::to_string(&msg) else {
        return;
    };

    let mut sender = sender.lock().await;
    let _ = sender.send(Message::Text(raw)).await;
}

async fn close(sender: &Sender) {
    let mut sender = sender.lock().await;
    let _ = sender.send(Message::Close(None)).await;
}

fn notice(kind: WebsocketMessageKind, payload: String) -> WebsocketMessage {
    WebsocketMessage {
        kind: kind,
        payload: payload,
        file_id: None,
        offset_start: None,
        offset_end: None,
        steps: None,
        elements: None,
    }
}

struct WebSocketEvaluator {
    sender: Sender,
    /// Replies to input prompts, handed over by the reader loop.
    input: Arc<Mutex<mpsc::Receiver<String>>>,
}

#[async_trait]
impl EvalIO for WebSocketEvaluator {
    async fn output(&self, output_msg: String) {
        let msg = notice(WebsocketMessageKind::Output, output_msg);
        send(&self.sender, msg).await;
    }

    async fn input(&self) -> String {
        let msg = notice(WebsocketMessageKind::Input, "".to_string());
        send(&self.sender, msg).await;

        // the reader loop forwards the reply. nothing arrives if the client
        // has gone, and the program is cancelled in that case anyway
        let mut input = self.input.lock().await;
        input.recv().await.unwrap_or_default()
    }

    async fn runtime_error(&self, error: RuntimeError) {
        let msg = WebsocketMessage {
            kind: WebsocketMessageKind::RuntimeError,
            payload: error.to_string(),
            file_id: None,
            offset_start: Some(error.span.start.char_offset),
            offset_end: Some(error.span.end.char_offset),
            steps: None,
            elements: None,
        };

        send(&self.sender, msg).await;
    }
}

/// Sends a RuntimeError frame to the client. Used for refusals that are the
/// caller's fault, so the IDE can show a message instead of the socket just
/// dropping.
async fn refuse(sender: &Sender, reason: &str) {
    let msg = notice(WebsocketMessageKind::RuntimeError, reason.to_string());
    send(sender, msg).await;
}

async fn execute(
    body: String,
    sender: Sender,
    input: Arc<Mutex<mpsc::Receiver<String>>>,
    cancel: CancelToken,
    throttle: Throttle,
    // held for the whole run; releases the slot on drop, including if this
    // task panics
    _run_slot: OwnedSemaphorePermit,
) {
    // scoped so the analysis slot is back before evaluation starts. holding it
    // across the run would mean a program waiting on input also holds up
    // everyone else's diagnostics
    let result = {
        let _slot = throttle.analysis_slot().await;
        ibc::analysis::analyze(body)
    };

    // a program with errors is not run. the editor already underlines them
    // through the diagnostics route, so this only has to say why nothing ran
    let Some(root) = result.runnable() else {
        let message = match result.errors.errors.first() {
            Some(error) => error.kind.format(),
            None => "the program could not be analyzed".to_string(),
        };

        send(&sender, notice(WebsocketMessageKind::AnalysisError, message)).await;
        close(&sender).await;
        return;
    };

    let mut io = WebSocketEvaluator {
        sender: sender.clone(),
        input: input,
    };

    let usage = evaluator::eval(root, &mut io, cancel.clone(), EvalLimits::default()).await;

    // sent however the run ended, including when a budget is what ended it, so
    // the editor can show the spend against the cap rather than only the error
    let msg = WebsocketMessage {
        kind: WebsocketMessageKind::Usage,
        payload: String::new(),
        file_id: None,
        offset_start: None,
        offset_end: None,
        steps: Some(usage.steps),
        elements: Some(usage.elements),
    };

    send(&sender, msg).await;

    // say so rather than just falling quiet, so the client knows the run ended
    // because it was asked to
    if cancel.is_cancelled() {
        let msg = notice(
            WebsocketMessageKind::Stop,
            "Execution stopped".to_string(),
        );

        send(&sender, msg).await;
    }

    // the run owns the connection: closing is how the client learns it ended
    close(&sender).await;
}

/// True while `running` is a program that has not finished.
fn is_running(running: &Option<JoinHandle<()>>) -> bool {
    match running {
        Some(task) => !task.is_finished(),
        None => false,
    }
}

async fn handle_execute(
    msg: WebsocketMessage,
    sender: &Sender,
    input: &Arc<Mutex<mpsc::Receiver<String>>>,
    cancel: &CancelToken,
    uid: &str,
    throttle: &Throttle,
) -> Option<JoinHandle<()>> {
    let Some(file_id) = msg.file_id else {
        refuse(sender, "Execute request is missing a file id").await;
        return None;
    };

    // the caller must own the file it is asking us to run
    let Some(file) = get_filename_uid(file_id) else {
        refuse(sender, "No such file").await;
        return None;
    };

    if file.uid != uid {
        refuse(sender, "No such file").await;
        return None;
    }

    if !throttle.try_execute(uid) {
        refuse(sender, "Too many runs, please wait a moment").await;
        return None;
    }

    let Some(run_slot) = throttle.try_run_slot() else {
        refuse(
            sender,
            "The server is busy running other programs, please try again in a moment",
        )
        .await;
        return None;
    };

    // a token is reused across runs on one connection, so an earlier stop
    // must not stop this one before it begins
    cancel.reset();

    let task = tokio::spawn(execute(
        msg.payload,
        sender.clone(),
        input.clone(),
        cancel.clone(),
        throttle.clone(),
        run_slot,
    ));

    Some(task)
}

async fn handle_ws_socket(
    socket: WebSocket,
    uid: String,
    throttle: Throttle,
    // held for the lifetime of the connection; releases the slot on drop
    _guard: ConnectionGuard,
) {
    // the halves are split so the client stays heard while a program runs. the
    // program writes through the sink; this loop keeps reading, which is what
    // lets a stop, or the socket closing, actually reach the evaluator
    let (sink, mut stream) = socket.split();
    let sender: Sender = Arc::new(Mutex::new(sink));

    // one reply at a time is all a program can be waiting for
    let (input_sender, input_receiver) = mpsc::channel::<String>(1);
    let input_receiver = Arc::new(Mutex::new(input_receiver));

    let cancel = CancelToken::new();
    let mut running: Option<JoinHandle<()>> = None;

    while let Some(msg) = stream.next().await {
        let text = match msg {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_reason)) => break,
            Err(e) => {
                println!("ws error: {}", e);
                break;
            }
            _ => continue,
        };

        let msg: WebsocketMessage = match serde_json::from_str(&text) {
            Ok(msg) => msg,
            Err(_) => {
                refuse(&sender, "Malformed message").await;
                continue;
            }
        };

        match msg.kind {
            WebsocketMessageKind::Execute => {
                if is_running(&running) {
                    refuse(&sender, "A program is already running").await;
                    continue;
                }

                running =
                    handle_execute(msg, &sender, &input_receiver, &cancel, &uid, &throttle).await;
            }
            WebsocketMessageKind::Stop => cancel.cancel(),
            WebsocketMessageKind::Input => {
                // nobody is waiting if the program is not asking for input, and
                // a reply nobody wants is dropped rather than held
                let _ = input_sender.try_send(msg.payload);
            }
            // server only accepts execute, stop and input
            WebsocketMessageKind::Output => {}
            WebsocketMessageKind::AnalysisError
            | WebsocketMessageKind::RuntimeError
            | WebsocketMessageKind::Usage => {}
        };
    }

    // the peer is gone: stop the program rather than leaving it running, and
    // wait for it, so the connection slot is only released once it is over
    cancel.cancel();

    // dropping this frees a program that is waiting on input it will never get
    drop(input_sender);

    if let Some(task) = running {
        let _ = task.await;
    }
}

/// Names the scheme the client is authenticating with, so the token that
/// follows it is not mistaken for a subprotocol the server should speak.
///
/// Versioned because the server echoes it back in the handshake: a later
/// scheme can be offered alongside this one and the client learns from the
/// echo which of them it got.
const AUTH_SUBPROTOCOL: &str = "ib-auth-v1";

/// The bearer token offered in `Sec-WebSocket-Protocol`: the marker, then the
/// token, and nothing else.
///
/// The browser WebSocket API cannot set request headers, and a subprotocol is
/// the one part of the handshake it does control. The token travels here
/// rather than in the query string because a URL reaches access logs, proxy
/// logs and history, none of which are treated as holding credentials.
///
/// Trimming has to agree with what axum does when it decides whether to echo
/// the marker, or this could read a token out of a handshake axum then
/// declines, leaving the browser to fail a connection the server thought it
/// had accepted.
fn offered_token(header: &str) -> Option<&str> {
    let mut offered = header.split(',').map(str::trim);

    let marker = offered.next()?;
    let token = offered.next()?;

    if marker != AUTH_SUBPROTOCOL || token.is_empty() || offered.next().is_some() {
        return None;
    }

    Some(token)
}

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Extension(throttle): Extension<Throttle>,
) -> Response {
    // one header line is all the one client sends. a proxy that split the
    // offer across two would need `get_all` joined back together, but nothing
    // sits in front of this server
    let offered = headers
        .get(SEC_WEBSOCKET_PROTOCOL)
        .and_then(|value| value.to_str().ok());

    let Some(token) = offered.and_then(offered_token) else {
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

    // the handshake has to name the subprotocol it accepted: a browser fails a
    // connection where it offered some and the answer chose none
    ws.protocols([AUTH_SUBPROTOCOL])
        .on_upgrade(move |socket| handle_ws_socket(socket, uid, throttle, guard))
}

#[cfg(test)]
mod tests;
