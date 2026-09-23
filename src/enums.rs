use serde::{Deserialize, Serialize};
use std::fmt::{Formatter, Result, Display};

#[derive(Eq, Hash, Deserialize, PartialEq, Serialize, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "enum_archetypes", rename_all = "snake_case")]
pub enum Archetypes {
    Leader,
    Spiker,
    Healer,
    Assassin,
}

impl Archetypes {
    pub fn from_u32(value: u32) -> Option<Archetypes> {
        match value {
            0 => Some(Archetypes::Leader),
            1 => Some(Archetypes::Spiker),
            2 => Some(Archetypes::Healer),
            3 => Some(Archetypes::Assassin),
            _ => None,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            Archetypes::Leader => 0,
            Archetypes::Spiker => 1,
            Archetypes::Healer => 2,
            Archetypes::Assassin => 3,
        }
    }
}

impl Display for Archetypes {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Eq, Hash, Deserialize, PartialEq, Serialize, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "enum_game_modes", rename_all = "snake_case")]
pub enum GameModes {
    Deathmatch,
    KingOfTheHill
}

impl Display for GameModes {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Eq, Hash, Deserialize, PartialEq, Serialize, Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "enum_maps", rename_all = "snake_case")]
pub enum Maps {
    Heaven,
    Ascent,
    Inferno,
    Colosseum,
    PlayGround,
}

impl Display for Maps {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}
