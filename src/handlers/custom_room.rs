use crate::{enums::Archetypes, errors::{AppResult, AppError}};
use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use crate::database;
use crate::dto::input::{CustomRoomSettingsDTO, SwitchSlotDTO};
use crate::dto::output::CustomRoomDTO;
use crate::handlers::Identity;
use crate::services::{custom_room as service, websocket::WebsocketLobby};
use rusoto_gamelift::GameLiftClient;

#[utoipa::path(
    get,
    path = "/matchmaking/custom-room",
    tag = "custom-room",
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "All custom rooms", body = Vec<CustomRoomDTO>),
        (status = 401, description = "Not logged in", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
pub async fn get_all(
    _: Identity,
    State(pool): State<database::DbPool>
) -> AppResult<Json<Vec<CustomRoomDTO>>> {
    let custom_rooms = service::get_all(&pool).await?;

    Ok(Json(custom_rooms))
}

#[utoipa::path(
    post,
    path = "/matchmaking/custom-room",
    tag = "custom-room",
    request_body = CustomRoomSettingsDTO,
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Created custom room", body = CustomRoomDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn create(
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>,
    Json(create_data): Json<CustomRoomSettingsDTO>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::create(
        create_data.into(),
        user_id,
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room",
    tag = "custom-room",
    request_body = CustomRoomSettingsDTO,
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Updated custom room owned by the user", body = CustomRoomDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn update(
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>,
    Json(update_data): Json<CustomRoomSettingsDTO>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::update(
        update_data.into(),
        user_id,
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room/{id}/join",
    tag = "custom-room",
    params(("id" = i32, Path, description = "Custom room id")),
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Updated custom room", body = CustomRoomDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn join(
    Path(custom_room_id): Path<i32>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::join(
        custom_room_id,
        user_id,
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room/{id}/quit",
    tag = "custom-room",
    params(("id" = i32, Path, description = "Custom room id")),
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Updated custom room", body = CustomRoomDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn quit(
    Path(custom_room_id): Path<i32>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::quit(
        custom_room_id,
        user_id,
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

#[utoipa::path(
    delete,
    path = "/matchmaking/custom-room",
    tag = "custom-room",
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Custom room owned by the user deleted"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn delete(
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>
) -> AppResult<impl IntoResponse> {
    service::delete(
        user_id,
        ws,
        &pool).await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room/{id}/slot",
    tag = "custom-room",
    params(("id" = i32, Path, description = "Custom room id")),
    request_body = SwitchSlotDTO,
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Updated custom room", body = CustomRoomDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn switch_slot(
    Path(custom_room_id): Path<i32>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>,
    Json(position): Json<SwitchSlotDTO>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::switch_slot(
        custom_room_id,
        user_id,
        position.into(),
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room/{id}/select-archetype/{archetype}",
    tag = "custom-room",
    params(("id" = i32, Path, description = "Custom room id"),
        ("archetype" = u32, Path, description = "0 = Leader, 1 = Spiker, 2 = Healer, 3 = Assassin")),
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Updated custom room", body = CustomRoomDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn switch_archetype(
    Path((custom_room_id, archetype_id)): Path<(i32, u32)>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>
) -> AppResult<impl IntoResponse> {
    match Archetypes::from_u32(archetype_id) {
        Some(archetype) => {
            let custom_room = service::switch_archetype(
                custom_room_id,
                archetype,
                user_id,
                ws,
                &pool).await?;

            Ok(Json(custom_room))
        },
        None => Err(AppError::BadRequest(format!("Unknown archetype id: {}", archetype_id)))
    }
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room/{id}/kick/{user_id}",
    tag = "custom-room",
    params(("id" = i32, Path, description = "Custom room id"),
        ("user_id" = i32, Path, description = "Id of the user to kick")),
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Updated custom room", body = CustomRoomDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn kick(
    Path((custom_room_id, user_id_to_kick)): Path<(i32, i32)>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::kick(
        custom_room_id,
        user_id_to_kick,
        Some(user_id),
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room/{id}/start-matchmaking",
    tag = "custom-room",
    params(("id" = i32, Path, description = "Custom room id")),
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Matchmaking started"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn start_matchmaking(
    Path(custom_room_id): Path<i32>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(gamelift): State<GameLiftClient>,
    State(pool): State<database::DbPool>
) -> AppResult<impl IntoResponse> {
       match service::start_matchmaking(
            custom_room_id,
            user_id,
            ws,
            &gamelift,
            &pool).await {
        Ok(_custom_room) => {
            Ok(StatusCode::OK)
        }
        Err(err) => {
            Err(err)
        }
    }
}

#[utoipa::path(
    put,
    path = "/matchmaking/custom-room/{id}/stop-matchmaking",
    tag = "custom-room",
    params(("id" = i32, Path, description = "Custom room id")),
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Matchmaking stopped"),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn stop_matchmaking(
    Path(custom_room_id): Path<i32>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(gamelift): State<GameLiftClient>,
    State(pool): State<database::DbPool>
) -> AppResult<impl IntoResponse> {
    match service::stop_matchmaking(
            custom_room_id,
            user_id,
            ws,
            &gamelift,
            &pool).await {
        Ok(_custom_room) => {
            Ok(StatusCode::OK)
        }
        Err(err) => {
            Err(err)
        }
    }
}
