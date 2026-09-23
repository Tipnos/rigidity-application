use crate::{enums::Archetypes, errors::{AppResult, AppError}};
use actix_web::{HttpResponse, web, web::Path};
use crate::enums::{Maps, GameModes};
use serde::{Serialize, Deserialize};
use crate::database;
use actix_identity::Identity;
use crate::services::{custom_room as service, websocket::WebsocketLobby};
use actix::{Addr};
use rusoto_gamelift::GameLiftClient;

pub mod dtos;

pub async fn get_all(
    _: Identity,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let custom_rooms = service::get_all(&pool).await?;

    Ok(HttpResponse::Ok().json(custom_rooms))
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
    create_data: web::Json<CustomRoomData>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    let custom_room = service::create(
        create_data.into_inner(),
        user_id.parse::<i32>().unwrap(),
        ws.get_ref().to_owned(),
        &pool).await?;

    Ok(HttpResponse::Ok().json(custom_room))
}

pub async fn update(
    update_data: web::Json<CustomRoomData>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    let custom_room = service::update(
        update_data.into_inner(),
        user_id.parse::<i32>().unwrap(),
        ws.get_ref().to_owned(),
        &pool).await?;

    Ok(HttpResponse::Ok().json(custom_room))
}

pub async fn join(
   custom_room_id: Path<i32>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    let custom_room = service::join(
        custom_room_id.into_inner(),
        user_id.parse::<i32>().unwrap(),
        ws.get_ref().to_owned(),
        &pool).await?;

    Ok(HttpResponse::Ok().json(custom_room))
}

pub async fn quit(
    custom_room_id: Path<i32>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    let custom_room = service::quit(
        custom_room_id.into_inner(),
        user_id.parse::<i32>().unwrap(),
        ws.get_ref().to_owned(),
        &pool).await?;

    Ok(HttpResponse::Ok().json(custom_room))
}

pub async fn delete(
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    service::delete(
        user_id.parse::<i32>().unwrap(),
        ws.get_ref().to_owned(),
        &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize)]
pub struct SwitchSlotData {
    pub team: i32,
    pub team_position: i32,
}

pub async fn switch_slot(
    custom_room_id: Path<i32>,
    id: Identity,
    position: web::Json<SwitchSlotData>,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    let custom_room = service::switch_slot(
        custom_room_id.into_inner(),
        user_id.parse::<i32>().unwrap(),
        position.into_inner(),
        ws.get_ref().to_owned(),
        &pool).await?;

    Ok(HttpResponse::Ok().json(custom_room))
}

pub async fn switch_archetype(
    param: Path<(i32, u32)>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    let (custom_room_id, archetype_id) = param.into_inner();

    match Archetypes::from_u32(archetype_id) {
        Some(archetype) => {
            let custom_room = service::switch_archetype(
                custom_room_id,
                archetype,
                user_id.parse::<i32>().unwrap(),
                ws.get_ref().to_owned(),
                &pool).await?;

            Ok(HttpResponse::Ok().json(custom_room))
        },
        None => Err(AppError::BadRequest(format!("Unknown archetype id: {}", archetype_id)))
    }
}

pub async fn kick(
    param: Path<(i32, i32)>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    let (custom_room_id, user_id_to_kick) = param.into_inner();
    let custom_room = service::kick(
        custom_room_id,
        user_id_to_kick,
        Some(user_id.parse::<i32>().unwrap()),
        ws.get_ref().to_owned(),
        &pool).await?;

    Ok(HttpResponse::Ok().json(custom_room))
}

pub async fn start_matchmaking(
    custom_room_id: Path<i32>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    gamelift: web::Data<GameLiftClient>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
       match service::start_matchmaking(
            custom_room_id.into_inner(),
            user_id.parse::<i32>().unwrap(),
            ws.get_ref().to_owned(),
            gamelift.get_ref(),
            &pool).await {
        Ok(_custom_room) => {
            Ok(HttpResponse::Ok().finish())
        }
        Err(err) => {
            Err(err)
        }
    }
}

pub async fn stop_matchmaking(
    custom_room_id: Path<i32>,
    id: Identity,
    ws: web::Data<Addr<WebsocketLobby>>,
    gamelift: web::Data<GameLiftClient>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    match service::stop_matchmaking(
            custom_room_id.into_inner(),
            user_id.parse::<i32>().unwrap(),
            ws.get_ref().to_owned(),
            gamelift.get_ref(),
            &pool).await {
        Ok(_custom_room) => {
            Ok(HttpResponse::Ok().finish())
        }
        Err(err) => {
            Err(err)
        }
    }
}
