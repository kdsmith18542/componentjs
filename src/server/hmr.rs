//! Hot Module Replacement (HMR) support

use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::ServerState;

/// HMR message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum HmrMessage {
    /// Connection established
    Connected,
    
    /// Full page reload required
    FullReload {
        reason: String,
    },
    
    /// CSS file updated (can be hot-reloaded)
    CssUpdate {
        path: String,
    },
    
    /// JavaScript module updated
    JsUpdate {
        path: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        accepted: Option<bool>,
    },
    
    /// Error during compilation
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        file: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        line: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        column: Option<u32>,
    },
}

/// Handle WebSocket upgrade for HMR
pub async fn hmr_websocket(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_hmr_socket(socket, state))
}

/// Handle HMR WebSocket connection
async fn handle_hmr_socket(socket: WebSocket, state: Arc<ServerState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to HMR messages
    let mut hmr_rx = state.hmr_tx.subscribe();
    
    // Send connected message
    let connected = HmrMessage::Connected;
    if let Ok(json) = serde_json::to_string(&connected) {
        let _ = sender.send(Message::Text(json)).await;
    }
    
    debug!("HMR client connected");
    
    // Spawn task to forward HMR messages to client
    let send_task = tokio::spawn(async move {
        while let Ok(message) = hmr_rx.recv().await {
            if let Ok(json) = serde_json::to_string(&message) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Handle incoming messages from client (for future use)
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            match message {
                Message::Text(text) => {
                    debug!("Received HMR message: {}", text);
                    // Handle client messages if needed
                }
                Message::Close(_) => {
                    debug!("HMR client disconnected");
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
    
    debug!("HMR connection closed");
}
