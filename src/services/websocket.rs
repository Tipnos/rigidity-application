use axum::{extract::WebSocketUpgrade, response::Response};
use serde::{Serialize};
use uuid::Uuid;

mod ws;
mod lobby;

pub type WebsocketLobby = lobby::Lobby;

pub use crate::dto::output::ServerMessageDTO as ServerMessage;

pub struct ForwardMessage {
    id: Uuid,
    message: String,
}

impl ForwardMessage {
    pub fn new<T: Serialize>(id: &Uuid, srv_message: ServerMessage<T>) -> Self {
        ForwardMessage {
            id: *id,
            message: srv_message.to_string()
        }
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

pub struct MultiForwardMessage {
    ids: Vec<Uuid>,
    message: String,
}

impl MultiForwardMessage {
    pub fn new<T: Serialize>(ids: &Vec<Uuid>, srv_message: ServerMessage<T>) -> Self {
        MultiForwardMessage {
            ids: ids.clone(),
            message: srv_message.to_string()
        }
    }

    pub fn get_ids(&self) -> &Vec<Uuid> {
        &self.ids
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

pub struct BroadcastExceptMessage {
    ids_to_except: Vec<Uuid>,
    message: String,
}

impl BroadcastExceptMessage {
    pub fn new<T: Serialize>(ids_to_except: &Vec<Uuid>, srv_message: ServerMessage<T>) -> Self {
        BroadcastExceptMessage {
            ids_to_except: ids_to_except.clone(),
            message: srv_message.to_string()
        }
    }

    pub fn get_ids_to_except(&self) -> &Vec<Uuid> {
        &self.ids_to_except
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

pub fn new_connection(
    upgrade: WebSocketUpgrade,
    user_id: Uuid,
    lobby: WebsocketLobby
) -> Response {
    upgrade.on_upgrade(move |socket| ws::run(socket, user_id, lobby))
}
