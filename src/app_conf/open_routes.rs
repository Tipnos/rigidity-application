use axum::{routing::post, Router};
use crate::handlers::{auth, user};
use crate::AppState;

pub fn get_all() -> Router<AppState> {
    Router::new().nest("/api-open", Router::new()
        .route("/password",
            post(auth::ask_password_reset)
                .put(auth::reset_password))
        .route("/login", post(auth::login))
        .route("/login-steam", post(auth::login_steam))
        .route("/user/create", post(user::create))
        .route("/email-confirmation",
            post(auth::email_confirmation)
                .put(auth::update_email_confirmation)))
}
