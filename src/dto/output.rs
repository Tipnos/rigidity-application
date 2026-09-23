use serde::Serialize;
use utoipa::ToSchema;
use chrono::NaiveDateTime;
use uuid::Uuid;
use crate::database::{
    custom_rooms::{CustomRoomDAO, CustomRoomSettings},
    custom_room_slots::CustomRoomSlotDAO,
    users::UserDAO,
};
use super::{GameModesDTO, MapsDTO};

#[derive(Debug, Serialize, ToSchema)]
pub struct UserDTO {
    pub id: i32,
    pub nickname: String,
    pub steam_id: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: NaiveDateTime,
}

impl From<UserDAO> for UserDTO {
    fn from(user: UserDAO) -> Self {
        UserDTO {
            id: user.id,
            nickname: user.nickname,
            steam_id: user.steam_id,
            first_name: user.first_name,
            last_name: user.last_name,
            birth_date: user.birth_date,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct CustomRoomDTO {
    pub id: i32,
    pub label: String,
    pub user_id: i32,
    pub nb_teams: i32,
    pub max_player_per_team: i32,
    pub game_mode: GameModesDTO,
    pub map: MapsDTO,
    pub matchmaking_ticket: Option<Uuid>,
    pub slots: Vec<CustomRoomSlotDTO>
}

impl From<(CustomRoomDAO, Vec<(CustomRoomSlotDAO, UserDAO)>)> for CustomRoomDTO {
    fn from((custom_room, slots): (CustomRoomDAO, Vec<(CustomRoomSlotDAO, UserDAO)>)) -> Self {
        CustomRoomDTO {
            id: custom_room.id,
            label: custom_room.label,
            user_id: custom_room.user_id,
            nb_teams: custom_room.nb_teams,
            max_player_per_team: custom_room.max_player_per_team,
            game_mode: custom_room.current_game_mode.into(),
            map: custom_room.current_map.into(),
            matchmaking_ticket: custom_room.matchmaking_ticket,
            slots: slots.into_iter().map(CustomRoomSlotDTO::from).collect(),
        }
    }
}

impl CustomRoomDTO {
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

#[derive(Serialize, ToSchema)]
pub struct CustomRoomSlotDTO {
    pub id: i32,
    pub custom_room_id: i32,
    pub team: i32,
    pub team_position: i32,
    pub user_id: i32,
    pub nickname: String,
    pub archetype: u32,
}

impl From<(CustomRoomSlotDAO, UserDAO)> for CustomRoomSlotDTO {
    fn from((slot, user): (CustomRoomSlotDAO, UserDAO)) -> Self {
        CustomRoomSlotDTO {
            id: slot.id,
            custom_room_id: slot.custom_room_id,
            team: slot.team,
            team_position: slot.team_position,
            user_id: slot.user_id,
            nickname: user.nickname,
            archetype: slot.current_archetype.to_u32(),
        }
    }
}

// Websocket envelope pushed to clients: `{route, message, data}`.
#[derive(Serialize)]
pub struct ServerMessageDTO<'a, T: Serialize> {
    route: String,
    message: String,
    data: &'a T
}

impl<'a, T: Serialize> ServerMessageDTO<'a, T> {
    pub fn new(route: String, message: String, data: &'a T) -> Self {
        ServerMessageDTO {
            route,
            message,
            data
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

// Websocket payloads (the `data` of a `ServerMessageDTO`).

#[derive(Serialize)]
pub struct UserIdDTO {
    pub user_id: i32,
}

#[derive(Serialize)]
pub struct SlotSwitchedDTO {
    pub user_id: i32,
    pub nickname: String,
    pub team: i32,
    pub team_position: i32,
}

#[derive(Serialize)]
pub struct ArchetypeSwitchedDTO {
    pub user_id: i32,
    pub archetype: u32,
}

#[derive(Serialize)]
pub struct MatchmakingSucceededDTO {
    pub ip_address: String,
    pub port: i32,
    pub player_id: String,
    pub player_session_id: String,
}

#[derive(Serialize)]
pub struct MatchmakingFailedDTO {
    pub reason: String,
}

#[derive(Serialize)]
pub struct EmptyDTO {}

#[derive(Serialize)]
pub struct CustomRoomUpdatedDTO {
    pub label: String,
    pub nb_teams: i32,
    pub max_players_per_team: i32,
    pub game_mode: Option<GameModesDTO>,
    pub map: Option<MapsDTO>,
}

impl From<CustomRoomSettings> for CustomRoomUpdatedDTO {
    fn from(settings: CustomRoomSettings) -> Self {
        CustomRoomUpdatedDTO {
            label: settings.label,
            nb_teams: settings.nb_teams,
            max_players_per_team: settings.max_player_per_team,
            game_mode: settings.game_mode.map(Into::into),
            map: settings.map.map(Into::into),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_dto_only_exposes_public_fields() {
        let now = chrono::Utc::now().naive_utc();
        let user = UserDAO {
            id: 1,
            nickname: String::from("nick"),
            created_at: now,
            steam_id: String::from("42"),
            first_name: String::from("first"),
            last_name: String::from("last"),
            birth_date: now,
        };

        let json = serde_json::to_value(UserDTO::from(user)).unwrap();
        let object = json.as_object().unwrap();
        assert!(!object.contains_key("created_at"));
        assert_eq!(object["steam_id"], "42");
    }

    #[test]
    fn empty_dto_serializes_as_empty_object() {
        assert_eq!(serde_json::to_string(&EmptyDTO {}).unwrap(), "{}");
    }
}
