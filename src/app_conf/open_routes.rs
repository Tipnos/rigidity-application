use utoipa_axum::{router::OpenApiRouter, routes};
use crate::handlers::{auth, user};
use crate::AppState;

pub fn get_all() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().nest("/api-open", OpenApiRouter::new()
        .routes(routes!(auth::ask_password_reset, auth::reset_password))
        .routes(routes!(auth::login))
        .routes(routes!(auth::login_steam))
        .routes(routes!(user::create))
        .routes(routes!(auth::email_confirmation, auth::update_email_confirmation)))
}
