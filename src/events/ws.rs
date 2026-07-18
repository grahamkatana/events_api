use super::models::Event;
use crate::common::state::SharedState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;

// The shape of every message we push to connected clients. `#[serde(tag
// = "type")]` makes each variant serialize with a "type" field naming
// itself — e.g. {"type":"created","event":{...}} — so JS clients can
// switch on `msg.type` easily.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsEventMessage<'a> {
    Created { event: &'a Event },
    Updated { event: &'a Event },
    Deleted { id: i32 },
}

// Called from the events handlers after a successful write — publishes
// the change to every currently-connected WebSocket client. Ignoring
// the Result: `send` only errors when there are zero subscribers
// right now, which is completely fine, not a real failure.
pub fn broadcast_created(state: &SharedState, event: &Event) {
    if let Ok(json) = serde_json::to_string(&WsEventMessage::Created { event }) {
        let _ = state.ws_tx.send(json);
    }
}

pub fn broadcast_updated(state: &SharedState, event: &Event) {
    if let Ok(json) = serde_json::to_string(&WsEventMessage::Updated { event }) {
        let _ = state.ws_tx.send(json);
    }
}

pub fn broadcast_deleted(state: &SharedState, id: i32) {
    if let Ok(json) = serde_json::to_string(&WsEventMessage::Deleted { id }) {
        let _ = state.ws_tx.send(json);
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    // `on_upgrade` runs AFTER the HTTP handshake completes and the
    // connection has switched protocols from HTTP to WebSocket.
    // Everything from here on is a persistent, two-way connection —
    // not a single request/response like every other handler so far.
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SharedState) {
    // Splits the connection into a write half (`sender`) and a read
    // half (`receiver`) so we can use both concurrently below.
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // Task 1: whenever ANY event changes anywhere in the app, forward
    // that notification out to THIS particular connected client.
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break; // client disconnected
            }
        }
    });

    // Task 2: we don't expect meaningful messages FROM the client in
    // this app, but we still need to keep reading from the socket —
    // this is how we detect the client closing the connection.
    let mut recv_task = tokio::spawn(async move {
        while receiver.next().await.is_some() {
            // Intentionally ignoring whatever the client sends.
        }
    });

    // Whichever task finishes first (usually because the client
    // disconnected) — cancel the other, so we don't leak it running
    // forever in the background.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}