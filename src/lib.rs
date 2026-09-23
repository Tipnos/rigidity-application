extern crate chrono;

use actix::Addr;
use actix::Actor;

pub mod app_conf;
pub mod database;
pub mod enums;
pub mod services;
pub mod cmd;
mod handlers;
mod errors;

pub fn new_websocket_lobby(pool: database::DbPool) -> Addr<services::websocket::WebsocketLobby> {
    services::websocket::WebsocketLobby::new(pool).start()
}