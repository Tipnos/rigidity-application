extern crate chrono;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;

pub mod app_conf;
pub mod database;
pub mod enums;
pub mod services;
pub mod cmd;
mod dto;
mod handlers;
mod errors;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: database::DbPool,
    pub ws: services::websocket::WebsocketLobby,
    // TODO: the state used to hold an AWS GameLift client, which the handlers
    // pulled out to start and stop FlexMatch matchmaking.
    pub cookie_key: Key,
}

pub fn new_websocket_lobby(pool: database::DbPool) -> services::websocket::WebsocketLobby {
    services::websocket::WebsocketLobby::new(pool)
}
