use serde::Deserialize;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::database::{custom_rooms::CustomRoomSettings, custom_room_slots::SlotPosition};
use crate::services::steam::SteamAuth;
use super::{GameModesDTO, MapsDTO};

// Body of the `--dev-login` route, which is kept out of the OpenAPI spec.
#[derive(Debug, Deserialize)]
pub struct DevLoginDTO {
    pub user_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SteamAuthDTO {
    pub app_id: u64,
    pub auth_ticket: String,
}

impl From<SteamAuthDTO> for SteamAuth {
    fn from(dto: SteamAuthDTO) -> Self {
        SteamAuth {
            app_id: dto.app_id,
            auth_ticket: dto.auth_ticket,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserDTO {
    pub nickname: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: DateTime<Utc>,
    pub auth: SteamAuthDTO,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CustomRoomSettingsDTO {
    pub label: String,
    pub nb_teams: i32,
    pub max_players_per_team: i32,
    pub game_mode: Option<GameModesDTO>,
    pub map: Option<MapsDTO>,
}

impl From<CustomRoomSettingsDTO> for CustomRoomSettings {
    fn from(dto: CustomRoomSettingsDTO) -> Self {
        CustomRoomSettings {
            label: dto.label,
            nb_teams: dto.nb_teams,
            max_player_per_team: dto.max_players_per_team,
            game_mode: dto.game_mode.map(Into::into),
            map: dto.map.map(Into::into),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SwitchSlotDTO {
    pub team: i32,
    pub team_position: i32,
}

impl From<SwitchSlotDTO> for SlotPosition {
    fn from(dto: SwitchSlotDTO) -> Self {
        SlotPosition {
            team: dto.team,
            team_position: dto.team_position,
        }
    }
}
