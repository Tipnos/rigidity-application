use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::enums::{GameModes, Maps};

pub mod input;
pub mod output;

// API mirrors of the domain enums, shared by `input` and `output`. Variant
// names match the domain ones so the JSON wire format is unchanged.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum GameModesDTO {
    Deathmatch,
    KingOfTheHill,
}

impl From<GameModes> for GameModesDTO {
    fn from(game_mode: GameModes) -> Self {
        match game_mode {
            GameModes::Deathmatch => GameModesDTO::Deathmatch,
            GameModes::KingOfTheHill => GameModesDTO::KingOfTheHill,
        }
    }
}

impl From<GameModesDTO> for GameModes {
    fn from(game_mode: GameModesDTO) -> Self {
        match game_mode {
            GameModesDTO::Deathmatch => GameModes::Deathmatch,
            GameModesDTO::KingOfTheHill => GameModes::KingOfTheHill,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum MapsDTO {
    Heaven,
    Ascent,
    Inferno,
    Colosseum,
    PlayGround,
}

impl From<Maps> for MapsDTO {
    fn from(map: Maps) -> Self {
        match map {
            Maps::Heaven => MapsDTO::Heaven,
            Maps::Ascent => MapsDTO::Ascent,
            Maps::Inferno => MapsDTO::Inferno,
            Maps::Colosseum => MapsDTO::Colosseum,
            Maps::PlayGround => MapsDTO::PlayGround,
        }
    }
}

impl From<MapsDTO> for Maps {
    fn from(map: MapsDTO) -> Self {
        match map {
            MapsDTO::Heaven => Maps::Heaven,
            MapsDTO::Ascent => Maps::Ascent,
            MapsDTO::Inferno => Maps::Inferno,
            MapsDTO::Colosseum => Maps::Colosseum,
            MapsDTO::PlayGround => Maps::PlayGround,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_mode_wire_format_is_unchanged() {
        assert_eq!(serde_json::to_string(&GameModesDTO::KingOfTheHill).unwrap(), "\"KingOfTheHill\"");
        assert_eq!(serde_json::from_str::<GameModesDTO>("\"KingOfTheHill\"").unwrap(), GameModesDTO::KingOfTheHill);
    }

    #[test]
    fn enums_round_trip_through_domain() {
        for dto in [GameModesDTO::Deathmatch, GameModesDTO::KingOfTheHill] {
            assert_eq!(GameModesDTO::from(GameModes::from(dto)), dto);
        }
        for dto in [MapsDTO::Heaven, MapsDTO::Ascent, MapsDTO::Inferno, MapsDTO::Colosseum, MapsDTO::PlayGround] {
            assert_eq!(MapsDTO::from(Maps::from(dto)), dto);
        }
    }
}
