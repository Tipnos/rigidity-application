use crate::{enums::Archetypes, errors::{AppResult, AppError}};
use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use crate::enums::{Maps, GameModes};
use serde::{Serialize, Deserialize};
use crate::database;
use crate::handlers::Identity;
use crate::services::{custom_room as service, websocket::WebsocketLobby};
use dtos::CustomRoomDto;
use rusoto_gamelift::GameLiftClient;

pub mod dtos;

pub async fn get_all(
    _: Identity,
    State(pool): State<database::DbPool>
) -> AppResult<Json<Vec<CustomRoomDto>>> {
    let custom_rooms = service::get_all(&pool).await?;

    Ok(Json(custom_rooms))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomRoomData {
    pub label: String,
    pub nb_teams: i32,
    pub max_players_per_team: i32,
    pub game_mode: Option<GameModes>,
    pub map: Option<Maps>
}

pub async fn create(
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>,
    Json(create_data): Json<CustomRoomData>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::create(
        create_data,
        user_id,
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

pub async fn update(
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>,
    Json(update_data): Json<CustomRoomData>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::update(
        update_data,
        user_id,
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

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

#[derive(Debug, Deserialize)]
pub struct SwitchSlotData {
    pub team: i32,
    pub team_position: i32,
}

pub async fn switch_slot(
    Path(custom_room_id): Path<i32>,
    Identity(user_id): Identity,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>,
    Json(position): Json<SwitchSlotData>
) -> AppResult<impl IntoResponse> {
    let custom_room = service::switch_slot(
        custom_room_id,
        user_id,
        position,
        ws,
        &pool).await?;

    Ok(Json(custom_room))
}

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
