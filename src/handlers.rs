use axum::{extract::{State, WebSocketUpgrade}, response::Response};
use crate::services::websocket::{self, WebsocketLobby};

pub mod auth;
pub mod custom_room;
pub mod user;
pub mod identity;

pub use identity::Identity;

pub async fn new_websocket(
    Identity(user_id): Identity,
    State(srv): State<WebsocketLobby>,
    ws: WebSocketUpgrade,
) -> Response {
    websocket::new_connection(ws, user_id, srv)
}
