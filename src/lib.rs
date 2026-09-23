extern crate chrono;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use rusoto_gamelift::GameLiftClient;

pub mod app_conf;
pub mod database;
pub mod enums;
pub mod services;
pub mod cmd;
mod handlers;
mod errors;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: database::DbPool,
    pub ws: services::websocket::WebsocketLobby,
    pub gamelift: GameLiftClient,
    pub cookie_key: Key,
}

pub fn new_websocket_lobby(pool: database::DbPool) -> services::websocket::WebsocketLobby {
    services::websocket::WebsocketLobby::new(pool)
}
