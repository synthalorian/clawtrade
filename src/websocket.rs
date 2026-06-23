use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use once_cell::sync::Lazy;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardEvent {
    ServiceCreated { service_id: String, name: String, agent_name: String },
    PurchaseInitiated { tx_id: String, service_name: String, buyer_id: String },
    PaymentConfirmed { tx_id: String, service_name: String, amount_cents: i64 },
    DeliveryCompleted { tx_id: String, service_type: String },
    AgentConnected { agent_id: String, agent_name: String },
}

pub type EventTx = broadcast::Sender<DashboardEvent>;
#[allow(dead_code)]
pub type EventRx = broadcast::Receiver<DashboardEvent>;

static EVENTS: Lazy<EventTx> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(100);
    tx
});

#[allow(dead_code)]
pub fn get_events() -> &'static EventTx {
    &EVENTS
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_pool): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut rx = EVENTS.subscribe();

    loop {
        tokio::select! {
            Ok(event) = rx.recv() => {
                let msg = match serde_json::to_string(&event) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Text(text))) => {
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Broadcast an event to all connected dashboard clients
pub fn broadcast_event(event: DashboardEvent) {
    let _ = EVENTS.send(event);
}
