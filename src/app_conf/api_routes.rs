use utoipa_axum::{router::OpenApiRouter, routes};
use crate::handlers::{custom_room, auth};
use crate::AppState;

pub fn get_all() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().nest("/api", OpenApiRouter::new()
        .routes(routes!(auth::logout))
        .routes(routes!(auth::refresh_cookie))
        .routes(routes!(
            custom_room::get_all,
            custom_room::create,
            custom_room::update,
            custom_room::delete))
        .routes(routes!(custom_room::join))
        .routes(routes!(custom_room::quit))
        .routes(routes!(custom_room::switch_slot))
        .routes(routes!(custom_room::switch_archetype))
        .routes(routes!(custom_room::kick))
        .routes(routes!(custom_room::start_matchmaking))
        .routes(routes!(custom_room::stop_matchmaking)))
}
