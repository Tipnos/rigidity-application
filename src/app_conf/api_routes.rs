use actix_web::{web, Scope};
use crate::handlers::{custom_room, auth};

pub fn get_all() -> Scope {
    web::scope("/api")
        .service(
            web::resource("/logout")
                .route(web::post().to(auth::logout)))
        .service(
            web::resource("/refresh-cookie")
                .route(web::get().to(auth::refresh_cookie)))
        .service(
            web::resource("/matchmaking/custom-room")
                .route(web::get().to(custom_room::get_all))
                .route(web::post().to(custom_room::create))
                .route(web::put().to(custom_room::update))
                .route(web::delete().to(custom_room::delete)))
        .service(
            web::resource("/matchmaking/custom-room/{id}/join")
                .route(web::put().to(custom_room::join)))
        .service(
            web::resource("/matchmaking/custom-room/{id}/quit")
                .route(web::put().to(custom_room::quit)))
        .service(
            web::resource("/matchmaking/custom-room/{id}/slot")
                .route(web::put().to(custom_room::switch_slot)))
        .service(
            web::resource("/matchmaking/custom-room/{id}/select-archetype/{archetype}")
                .route(web::put().to(custom_room::switch_archetype)))
        .service(
            web::resource("/matchmaking/custom-room/{id}/kick/{user_id}")
                .route(web::put().to(custom_room::kick)))
        .service(
            web::resource("/matchmaking/custom-room/{id}/start-matchmaking")
                .route(web::put().to(custom_room::start_matchmaking)))
        .service(
            web::resource("/matchmaking/custom-room/{id}/stop-matchmaking")
                .route(web::put().to(custom_room::stop_matchmaking)))
}