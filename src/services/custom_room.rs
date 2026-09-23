use std::collections::HashMap;
use crate::database::{custom_rooms, custom_room_slots, users, DbPool};
use crate::database::custom_rooms::CustomRoomDAO;
use crate::database::custom_room_slots::CustomRoomSlotDAO;
use rusoto_gamelift::*;
use crate::services::websocket::{ServerMessage, BroadcastExceptMessage, WebsocketLobby, MultiForwardMessage, ForwardMessage};
use serde::{Serialize};
use crate::handlers::custom_room::dtos::CustomRoomDto;
use crate::handlers::custom_room::{CustomRoomData, SwitchSlotData};
use crate::errors::{AppResult, AppError};
use crate::enums::Archetypes;
use uuid::Uuid;
use crate::services::aws::{FlexMatchEvents, FlexMatchData, FlexMatchSucceededDetail};

pub async fn get_all(pool: &DbPool) -> AppResult<Vec<CustomRoomDto>> {
    let tuples = custom_rooms::get_all_with_slots(pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let mut results = Vec::new();
    for tuple in tuples {
        let dto = CustomRoomDto::new(tuple, pool).await
            .map_err(|err| AppError::InternalServerError(err.to_string()))?;
        results.push(dto);
    }

    Ok(results)
}

pub async fn create(
    create_data: CustomRoomData,
    user_id: i32,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<CustomRoomDto> {
    let tuple = custom_rooms::create_with_owner_slot(
        &create_data.label,
        user_id,
        create_data.nb_teams,
        create_data.max_players_per_team,
        create_data.game_mode,
        create_data.map,
        pool
    ).await.map_err(|err| AppError::BadRequest(err.to_string()))?;

    let dto = CustomRoomDto::new(tuple, pool).await
        .map_err(|err| AppError::InternalServerError(err.to_string()))?;

    let msg = BroadcastExceptMessage::new(
        &vec![user_id],
        ServerMessage::new(
            String::from("/matchmaking/custom-room"),
            String::from("new"),
            &dto
        )
    );
    ws.broadcast_except(msg);

    Ok(dto)
}

pub async fn join(
    custom_room_id: i32,
    user_id: i32,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<CustomRoomDto> {
    let (custom_room, slots) = custom_rooms::get_with_slots(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let (team, team_position) = find_open_slot(&custom_room, &slots)?;

    custom_room_slots::create(custom_room_id, team, team_position, user_id, Archetypes::Leader, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let tuple = custom_rooms::get_with_slots(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let dto = CustomRoomDto::new(tuple, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let user_ids = dto.get_all_user_ids_except(&user_id);
    if let Some(slot_dto_index) = dto.get_slot_index_from_user_id(&user_id) {
        let msg = MultiForwardMessage::new(
            &user_ids,
            ServerMessage::new(
                String::from("/matchmaking/custom-room"),
                String::from("join"),
                &dto.slots.get(slot_dto_index))
        );

        ws.multi_forward(msg);
    } else {
        return Err(AppError::InternalServerError(String::from("Error in Custom room dtos.")))
    }

    Ok(dto)
}

pub async fn update(
    update_data: CustomRoomData,
    user_id: i32,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<CustomRoomDto> {
    let (existing_room, _) = custom_rooms::get_by_user_id_with_slots(user_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let updated_room = custom_rooms::update(
        existing_room.id,
        &update_data.label,
        update_data.nb_teams,
        update_data.max_players_per_team,
        update_data.game_mode,
        update_data.map,
        pool
    ).await.map_err(|err| AppError::BadRequest(err.to_string()))?;

    let slots = custom_room_slots::get_by_custom_room_id(updated_room.id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    send_multi_forward_message(
        ws,
        &user_id,
        (updated_room, slots),
        String::from("update"),
        pool,
        &update_data
    ).await
}

pub async fn quit(
    custom_room_id: i32,
    user_id: i32,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<CustomRoomDto> {
    #[derive(Serialize)]
    struct WsData<'a> {
        pub user_id: &'a i32
    }

    custom_room_slots::delete_by_user_id(user_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let tuple = custom_rooms::get_with_slots(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let ws_data = WsData {user_id: &user_id};
    send_multi_forward_message(
        ws,
        &user_id,
        tuple,
        String::from("quit"),
        pool,
        &ws_data
    ).await
}

pub async fn delete(
    user_id: i32,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<()> {
    let tuple = custom_rooms::get_by_user_id_with_slots(user_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    custom_rooms::delete_by_user_id(user_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    #[derive(Serialize)]
    struct Empty{}
    send_multi_forward_message(
        ws,
        &user_id,
        tuple,
        String::from("delete"),
        pool,
        &Empty{}
    ).await?;

    Ok(())
}

pub async fn switch_slot(
    custom_room_id: i32,
    user_id: i32,
    position: SwitchSlotData,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<CustomRoomDto> {
    #[derive(Serialize)]
    struct WsData<'a> {
        pub user_id: &'a i32,
        pub nickname: &'a str,
        pub team: &'a i32,
        pub team_position: &'a i32
    }

    validate_switch_slot(custom_room_id, &position, pool).await?;

    custom_room_slots::update_position(user_id, custom_room_id, position.team, position.team_position, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let tuple = custom_rooms::get_with_slots(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let user = users::get(user_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;
    let ws_data = WsData {
        user_id: &user_id,
        nickname: &user.nickname,
        team: &position.team,
        team_position: &position.team_position
    };

    send_multi_forward_message(
        ws,
        &user_id,
        tuple,
        String::from("slot"),
        pool,
        &ws_data
    ).await
}

pub async fn switch_archetype(
    custom_room_id: i32,
    archetype: Archetypes,
    user_id: i32,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<CustomRoomDto> {
    #[derive(Serialize)]
    struct WsData<'a> {
        pub user_id: &'a i32,
        pub archetype: u32,
    }

    custom_room_slots::update_archetype_by_user_id(user_id, archetype, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let tuple = custom_rooms::get_with_slots(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let ws_data = WsData {
        user_id: &user_id,
        archetype: archetype.to_u32(),
    };
    send_multi_forward_message(
        ws,
        &user_id,
        tuple,
        String::from("select-archetype"),
        pool,
        &ws_data
    ).await
}

pub async fn kick(
    custom_room_id: i32,
    user_id_to_kick: i32,
    o_user_id: Option<i32>,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<CustomRoomDto> {
    custom_room_slots::delete_by_user_id(user_id_to_kick, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let tuple = custom_rooms::get_with_slots(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    #[derive(Serialize)]
    struct WsData {
        pub user_id: i32,
    }
    let data = WsData{user_id: user_id_to_kick};

    let dto = CustomRoomDto::new(tuple, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    let mut user_ids;
    let message;
    if let Some(user_id) = o_user_id {
        user_ids = dto.get_all_user_ids_except(&user_id);
        user_ids.push(user_id_to_kick);
        message = String::from("kick");
    } else { // disconnect
        user_ids = dto.get_all_user_ids();
        message = String::from("disconnect");
    }

    let msg = MultiForwardMessage::new(
        &user_ids,
        ServerMessage::new(
            String::from("/matchmaking/custom-room"),
            message,
            &data)
    );
    ws.multi_forward(msg);

    Ok(dto)
}

pub async fn start_matchmaking(
    custom_room_id: i32,
    user_id: i32,
    ws: WebsocketLobby,
    gamelift: &GameLiftClient,
    pool: &DbPool
) -> AppResult<()> {
    let (custom_room, tuples) = custom_rooms::get_with_users(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if custom_room.user_id != user_id {
        return Err(AppError::BadRequest(String::from("Only the room owner can start matchmaking.")))
    }

    let ticket_id = Uuid::new_v4();
    let start_matchmaking_input = custom_room.get_start_matchmaking_input(&tuples, &ticket_id);
    match gamelift.start_matchmaking(start_matchmaking_input).await {
        Ok(result) => {
            if let Some(_matchmaking_ticket) = result.matchmaking_ticket {
                custom_rooms::update_ticket(custom_room_id, Some(ticket_id), pool).await
                    .map_err(|err| AppError::InternalServerError(err.to_string()))?;

                #[derive(Serialize)]
                struct Empty{}
                let data = &Empty{};
                let mut slots = Vec::new();
                for (slot, _user) in tuples {
                    slots.push(slot);
                }
                send_multi_forward_message(
                    ws,
                    &user_id,
                    (custom_room, slots),
                    String::from("start-matchmaking"),
                    pool,
                    data).await?;

                return Ok(())
            }

            Err(AppError::InternalServerError(String::from("Problem with aws matchmaking.")))
        },
        Err(err) => {
            Err(AppError::BadRequest(err.to_string()))
        }
    }
}

pub async fn stop_matchmaking(
    custom_room_id: i32,
    user_id: i32,
    ws: WebsocketLobby,
    gamelift: &GameLiftClient,
    pool: &DbPool
) -> AppResult<()> {
    let tuple = custom_rooms::get_with_slots(custom_room_id, pool).await
        .map_err(|err| AppError::BadRequest(err.to_string()))?;

    if tuple.0.user_id != user_id {
        return Err(AppError::BadRequest(String::from("Only the room owner can stop matchmaking.")))
    }
    if tuple.0.matchmaking_ticket == None {
        return Err(AppError::BadRequest(String::from("No matchmaking started for this room.")))
    }

    match gamelift.stop_matchmaking(StopMatchmakingInput {
        ticket_id: tuple.0.matchmaking_ticket.unwrap().to_string()
    }).await {
        Ok(_result) => {
            custom_rooms::update_ticket(custom_room_id, None, pool).await
                .map_err(|err| AppError::InternalServerError(err.to_string()))?;

            #[derive(Serialize)]
            struct Empty{}
            let data = &Empty{};
            send_multi_forward_message(
                ws,
                &user_id,
                tuple,
                String::from("stop-matchmaking"),
                pool,
                data).await?;

            Ok(())
        },
        Err(err) => {
            Err(AppError::BadRequest(err.to_string()))
        }
    }
}

pub async fn matchmaking_succeeded(
    data: FlexMatchData<FlexMatchSucceededDetail>,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<()> {
    let ticket_id = Uuid::parse_str(&data.detail.tickets[0].ticket_id).unwrap();

    let (custom_room, slots) = custom_rooms::get_by_ticket_id_with_slots(ticket_id, pool).await
        .map_err(|err| AppError::InternalServerError(err.to_string()))?;

    #[derive(Serialize)]
    struct WsData<'a> {
        pub ip_address: &'a str,
        pub port: &'a i32,
        pub player_id: &'a str,
        pub player_session_id: &'a str
    }

    for slot in slots {
        let str_user_id = slot.user_id.to_string();
        for player in &data.detail.game_session_info.players {
            if player.player_id == str_user_id {
                let ws_data = WsData {
                    ip_address: &data.detail.game_session_info.ip_address,
                    port: &data.detail.game_session_info.port,
                    player_id: &player.player_id,
                    player_session_id: &player.player_session_id
                };

                let msg = ForwardMessage::new(
                    &slot.user_id,
                    ServerMessage::new(
                        String::from("/matchmaking/custom-room"),
                        String::from("matchmaking-succeeded"),
                        &ws_data)
                );
                ws.forward(msg);
                break;
            }
        }
    }

    custom_rooms::delete_by_user_id(custom_room.user_id, pool).await
        .map_err(|err| AppError::InternalServerError(err.to_string()))?;

    Ok(())
}

pub async fn matchmaking_failed(
    reason: FlexMatchEvents,
    ticket_id: &str,
    ws: WebsocketLobby,
    pool: &DbPool
) -> AppResult<()> {
    let uuid_ticket_id = Uuid::parse_str(ticket_id).unwrap();
    let (custom_room, slots) = custom_rooms::get_by_ticket_id_with_slots(uuid_ticket_id, pool).await
        .map_err(|err| AppError::InternalServerError(err.to_string()))?;

    #[derive(Serialize)]
    struct WsData {
        pub reason: String
    }

    let mut user_ids = Vec::new();
    for slot in slots {
        user_ids.push(slot.user_id);
    }

    let msg = MultiForwardMessage::new(
        &user_ids,
        ServerMessage::new(
            String::from("/matchmaking/custom-room"),
            String::from("matchmaking-failed"),
            &WsData {reason: reason.to_string()})
    );
    ws.multi_forward(msg);

    custom_rooms::update_ticket(custom_room.id, None, pool).await
        .map_err(|err| AppError::InternalServerError(err.to_string()))?;

    Ok(())
}

pub async fn handle_websocket_closing(
    user_id: &i32,
    ws: WebsocketLobby,
    pool: &DbPool
) {
    if let Ok(slot) = custom_room_slots::get_by_user_id(*user_id, pool).await {
        match custom_rooms::get_with_slots(slot.custom_room_id, pool).await {
            Ok(tuple) => {
                if tuple.0.user_id == *user_id {
                    if let Err(_err) = delete(*user_id, ws, pool).await {
                        // futur logger service InternalServerError
                    }
                } else {
                    if let Err(_err) = kick(
                        slot.custom_room_id,
                        *user_id,
                        None,
                        ws,
                        pool).await {
                        // futur logger service InternalServerError
                    }
                }
            },
            Err(_) => {
                // futur logger service InternalServerError
            }
        }
    }
}

// Ports CustomRoomSlotForm::new_from_user_join's slot-allocation logic: finds
// the first free (team, team_position) pair, erroring if the room is full.
fn find_open_slot(custom_room: &CustomRoomDAO, slots: &[CustomRoomSlotDAO]) -> AppResult<(i32, i32)> {
    let room_full_error = Err(AppError::BadRequest(String::from("Can't join, room is full.")));

    if slots.len() >= custom_room.get_capacity() {
        return room_full_error;
    }

    let mut empty_slots = HashMap::new();
    for i in 0..custom_room.nb_teams {
        let mut hash = HashMap::new();
        for j in 0..custom_room.max_player_per_team {
            hash.insert(j, j);
        }

        empty_slots.insert(i, hash);
    }

    for slot in slots {
        match empty_slots.get_mut(&slot.team) {
            Some(slots_of_team) => {
                slots_of_team.remove(&slot.team_position);
            }
            None => {
                return Err(AppError::InternalServerError(String::from("Custom room error in slot allocation.")))
            }
        }
    }

    for (team, team_empty_slots) in empty_slots {
        if team_empty_slots.len() > 0 {
            if let Some(team_position) = team_empty_slots.values().next() {
                return Ok((team, *team_position));
            }
        }
    }

    room_full_error
}

// Ports CustomRoomSlotForm::new_from_switch_slot's validation: the target
// position must exist in the room and not already be taken.
async fn validate_switch_slot(custom_room_id: i32, slot_data: &SwitchSlotData, pool: &DbPool) -> AppResult<()> {
    let custom_room = custom_rooms::get_by_id(custom_room_id, pool).await
        .map_err(|_err| AppError::BadRequest(format!("Unknown custom room id: {}", custom_room_id)))?;

    if !custom_room.is_valid_slot(&slot_data.team, &slot_data.team_position) {
        return Err(AppError::BadRequest(String::from("Invalid slot position.")))
    }

    match custom_room_slots::get_by_position(custom_room_id, slot_data.team, slot_data.team_position, pool).await {
        Ok(_slot) => Err(AppError::BadRequest(String::from("Slot already taken by someone else"))),
        Err(_err) => Ok(())
    }
}

async fn send_multi_forward_message<T: Serialize>(
    ws: WebsocketLobby,
    user_id: &i32,
    tuple: (CustomRoomDAO, Vec<CustomRoomSlotDAO>),
    typ: String,
    pool: &DbPool,
    data: &T
) -> AppResult<CustomRoomDto> {
    match CustomRoomDto::new(tuple, pool).await {
        Ok(dto) => {
            let user_ids = dto.get_all_user_ids_except(&user_id);
            let msg = MultiForwardMessage::new(
                &user_ids,
                ServerMessage::new(
                    String::from("/matchmaking/custom-room"),
                    typ,
                    data)
            );
            ws.multi_forward(msg);

            Ok(dto)
        },
        Err(err) => Err(AppError::BadRequest(err.to_string()))
    }
}
