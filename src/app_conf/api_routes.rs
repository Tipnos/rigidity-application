use axum::{routing::{get, post, put}, Router};
use crate::handlers::{custom_room, auth};
use crate::AppState;

pub fn get_all() -> Router<AppState> {
    Router::new().nest("/api", Router::new()
        .route("/logout", post(auth::logout))
        .route("/refresh-cookie", get(auth::refresh_cookie))
        .route("/matchmaking/custom-room",
            get(custom_room::get_all)
                .post(custom_room::create)
                .put(custom_room::update)
                .delete(custom_room::delete))
        .route("/matchmaking/custom-room/{id}/join", put(custom_room::join))
        .route("/matchmaking/custom-room/{id}/quit", put(custom_room::quit))
        .route("/matchmaking/custom-room/{id}/slot", put(custom_room::switch_slot))
        .route("/matchmaking/custom-room/{id}/select-archetype/{archetype}",
            put(custom_room::switch_archetype))
        .route("/matchmaking/custom-room/{id}/kick/{user_id}", put(custom_room::kick))
        .route("/matchmaking/custom-room/{id}/start-matchmaking",
            put(custom_room::start_matchmaking))
        .route("/matchmaking/custom-room/{id}/stop-matchmaking",
            put(custom_room::stop_matchmaking)))
}
