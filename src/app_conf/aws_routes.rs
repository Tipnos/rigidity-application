use axum::{routing::post, Router};
use crate::handlers::aws;
use crate::AppState;

pub fn get_all() -> Router<AppState> {
    Router::new().nest("/aws", Router::new()
        .route("/sns", post(aws::sns)))
}
