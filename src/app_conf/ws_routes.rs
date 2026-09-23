use axum::routing::{get as get_method, MethodRouter};
use crate::handlers;
use crate::AppState;

pub fn get() -> MethodRouter<AppState> {
    get_method(handlers::new_websocket)
}
