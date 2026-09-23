use axum::{extract::WebSocketUpgrade, response::Response};
use serde::{Serialize};

mod ws;
mod lobby;

pub type WebsocketLobby = lobby::Lobby;

#[derive(Serialize)]
pub struct ServerMessage<'a, T: Serialize> {
    route: String,
    message: String,
    data: &'a T
}

impl<'a, T: Serialize> ServerMessage<'a, T> {
    pub fn new(route: String, message: String, data: &'a T) -> Self {
        ServerMessage {
            route,
            message,
            data
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
} 

pub struct ForwardMessage {
    id: i32,
    message: String,
}

impl ForwardMessage {
    pub fn new<T: Serialize>(id: &i32, srv_message: ServerMessage<T>) -> Self {
        ForwardMessage {
            id: *id,
            message: srv_message.to_string()
        }
    }

    pub fn get_id(&self) -> &i32 {
        &self.id
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

pub struct MultiForwardMessage {
    ids: Vec<i32>,
    message: String,
}

impl MultiForwardMessage {
    pub fn new<T: Serialize>(ids: &Vec<i32>, srv_message: ServerMessage<T>) -> Self {
        MultiForwardMessage {
            ids: ids.clone(),
            message: srv_message.to_string()
        }
    }

    pub fn get_ids(&self) -> &Vec<i32> {
        &self.ids
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

pub struct BroadcastExceptMessage {
    ids_to_except: Vec<i32>,
    message: String,
}

impl BroadcastExceptMessage {
    pub fn new<T: Serialize>(ids_to_except: &Vec<i32>, srv_message: ServerMessage<T>) -> Self {
        BroadcastExceptMessage {
            ids_to_except: ids_to_except.clone(),
            message: srv_message.to_string()
        }
    }

    pub fn get_ids_to_except(&self) -> &Vec<i32> {
        &self.ids_to_except
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

pub fn new_connection(
    upgrade: WebSocketUpgrade,
    user_id: i32,
    lobby: WebsocketLobby
) -> Response {
    upgrade.on_upgrade(move |socket| ws::run(socket, user_id, lobby))
}
