use axum::{routing::post, Router};
use crate::handlers::auth;
use crate::AppState;

// Only merged when `--dev-login` is set, and kept out of the OpenAPI spec
pub fn get_all() -> Router<AppState> {
    Router::new().nest("/api-open", Router::new()
        .route("/dev-login", post(auth::dev_login)))
}
