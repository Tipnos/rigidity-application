use serde::{Serialize};
use crate::database::{custom_rooms::CustomRoomDAO, custom_room_slots::CustomRoomSlotDAO, users, DbPool, DbResult};
use crate::enums::{GameModes, Maps};
use uuid::Uuid;

#[derive(Serialize)]
pub struct CustomRoomDto {
    pub id: i32,
    pub label: String,
    pub user_id: i32,
    pub nb_teams: i32,
    pub max_player_per_team: i32,
    pub game_mode: GameModes,
    pub map: Maps,
    pub matchmaking_ticket: Option<Uuid>,
    pub slots: Vec<CustomRoomSlotDto>
}

impl CustomRoomDto {
    pub async fn new(tuple: (CustomRoomDAO, Vec<CustomRoomSlotDAO>), pool: &DbPool) -> DbResult<Self> {
        let (custom_room, room_slots) = tuple;
        let user_ids: Vec<i32> = room_slots.iter().map(|s| s.user_id).collect();
        let fetched_users = users::get_by_ids(&user_ids, pool).await?;

        let mut slots = Vec::with_capacity(room_slots.len());
        for slot in room_slots {
            match fetched_users.iter().find(|u| u.id == slot.user_id) {
                Some(user) => slots.push(CustomRoomSlotDto::new(slot, user.nickname.clone())),
                None => return Err(sqlx::Error::RowNotFound),
            }
        }

        Ok(CustomRoomDto {
            id: custom_room.id,
            label: custom_room.label,
            user_id: custom_room.user_id,
            nb_teams: custom_room.nb_teams,
            max_player_per_team: custom_room.max_player_per_team,
            game_mode: custom_room.current_game_mode,
            map: custom_room.current_map,
            matchmaking_ticket: custom_room.matchmaking_ticket,
            slots: slots,
        })
    }

    pub fn get_all_user_ids_except(&self, except_id: &i32) -> Vec<i32> {
        let mut result = Vec::new();

        for slot in &self.slots {
            if slot.user_id != *except_id {
                result.push(slot.user_id.clone());
            }
        }

        result
    }

    pub fn get_all_user_ids(&self) -> Vec<i32> {
        let mut result = Vec::new();

        for slot in &self.slots {
            result.push(slot.user_id.clone());
        }

        result
    }

    pub fn get_slot_index_from_user_id(&self, user_id: &i32) -> Option<usize> {
        let mut i = 0;
        for slot in &self.slots {
            if slot.user_id == *user_id {
                let j = i as usize;
                return Some(j);
            }
            i += 1;
        }

        None
    }
}

#[derive(Serialize)]
pub struct CustomRoomSlotDto {
    pub id: i32,
    pub custom_room_id: i32,
    pub team: i32,
    pub team_position: i32,
    pub user_id: i32,
    pub nickname: String,
    pub archetype: u32,
}

impl CustomRoomSlotDto {
    pub fn new(slot: CustomRoomSlotDAO, nickname: String) -> Self {
        CustomRoomSlotDto {
            id: slot.id,
            custom_room_id: slot.custom_room_id,
            team: slot.team,
            team_position: slot.team_position,
            user_id: slot.user_id,
            nickname,
            archetype: slot.current_archetype.to_u32(),
        }
    }
}
