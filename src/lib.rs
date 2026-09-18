#[macro_use]
extern crate diesel;
extern crate chrono;

use diesel::{r2d2::ConnectionManager, PgConnection};
use actix::Addr;
use actix::Actor;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub mod app_conf;
pub mod database;
pub mod enums;
pub mod services;
pub mod cmd;
mod handlers;
mod models;
mod errors;
mod schema;

pub fn new_websocket_lobby(pool: Pool) -> Addr<services::websocket::WebsocketLobby> {
    services::websocket::WebsocketLobby::new(pool).start()
}