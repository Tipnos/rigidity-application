use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rusoto_gamelift::{Player, StartMatchmakingInput};

use crate::enums::{Archetypes, GameModes, Maps};

use super::{DbPool, DbResult};
use super::custom_room_slots::{self, CustomRoomSlotDAO};
use super::users::{self, UserDAO};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CustomRoomDAO {
    pub id: i32,
    pub label: String,
    pub user_id: i32,
    pub nb_teams: i32,
    pub max_player_per_team: i32,
    pub current_game_mode: GameModes,
    pub current_map: Maps,
    pub matchmaking_ticket: Option<Uuid>,
}

impl CustomRoomDAO {
    pub fn get_capacity(&self) -> usize {
        (self.nb_teams * self.max_player_per_team) as usize
    }

    pub fn is_valid_slot(&self, team: &i32, team_position: &i32) -> bool {
        *team < self.nb_teams && *team_position < self.max_player_per_team
    }

    pub fn get_start_matchmaking_input(&self, tuples: &Vec<(CustomRoomSlotDAO, UserDAO)>, ticket_id: &Uuid) -> StartMatchmakingInput {
        let mut players = Vec::new();

        for (slot, user) in tuples {
            if user.id == slot.user_id {
                let attributes = slot.get_gamelift_attributes(&user.nickname);

                players.push(Player {
                    latency_in_ms: None,
                    player_attributes: Some(attributes),
                    player_id: Some(slot.user_id.to_string()),
                    team: Some(slot.team.to_string()),
                });
            }
        }

        StartMatchmakingInput {
            configuration_name: self.current_map.to_string(),
            players: players,
            ticket_id: Some(ticket_id.to_string()),
        }
    }
}

pub async fn get_by_id(id: i32, pool: &DbPool) -> DbResult<CustomRoomDAO> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        SELECT id, label, user_id, nb_teams, max_player_per_team,
               current_game_mode as "current_game_mode: GameModes", current_map as "current_map: Maps", matchmaking_ticket
        FROM custom_rooms WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_user_id(user_id: i32, pool: &DbPool) -> DbResult<CustomRoomDAO> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        SELECT id, label, user_id, nb_teams, max_player_per_team,
               current_game_mode as "current_game_mode: GameModes", current_map as "current_map: Maps", matchmaking_ticket
        FROM custom_rooms WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_ticket_id(ticket_id: Uuid, pool: &DbPool) -> DbResult<CustomRoomDAO> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        SELECT id, label, user_id, nb_teams, max_player_per_team,
               current_game_mode as "current_game_mode: GameModes", current_map as "current_map: Maps", matchmaking_ticket
        FROM custom_rooms WHERE matchmaking_ticket = $1
        "#,
        ticket_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_all(pool: &DbPool) -> DbResult<Vec<CustomRoomDAO>> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        SELECT id, label, user_id, nb_teams, max_player_per_team,
               current_game_mode as "current_game_mode: GameModes", current_map as "current_map: Maps", matchmaking_ticket
        FROM custom_rooms
        "#
    )
    .fetch_all(pool)
    .await
}

// Inserts the room and its owner's first slot (team 0, position 0, leader) in
// one transaction, replacing the separate insert + `SELECT id ORDER BY id DESC`
// diesel used to locate the new room's id.
pub async fn create_with_owner_slot(
    label: &str,
    user_id: i32,
    nb_teams: i32,
    max_player_per_team: i32,
    game_mode: Option<GameModes>,
    map: Option<Maps>,
    pool: &DbPool,
) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    // Mirrors the DB column defaults (migrations 2022-03-15-113708 /
    // 2022-04-15-082107): current_game_mode defaults to 'king_of_the_hill',
    // current_map to 'inferno'.
    let game_mode = game_mode.unwrap_or(GameModes::KingOfTheHill);
    let map = map.unwrap_or(Maps::Inferno);

    let mut tx = pool.begin().await?;

    let custom_room = sqlx::query_as!(
        CustomRoomDAO,
        r#"
        INSERT INTO custom_rooms (label, user_id, nb_teams, max_player_per_team, current_game_mode, current_map)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, label, user_id, nb_teams, max_player_per_team,
                  current_game_mode as "current_game_mode: GameModes", current_map as "current_map: Maps", matchmaking_ticket
        "#,
        label, user_id, nb_teams, max_player_per_team, game_mode as GameModes, map as Maps
    )
    .fetch_one(&mut *tx)
    .await?;

    let slot = sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        INSERT INTO custom_room_slots (custom_room_id, team, team_position, user_id, current_archetype)
        VALUES ($1, 0, 0, $2, 'leader')
        RETURNING id, custom_room_id, team, team_position, user_id, current_archetype as "current_archetype: Archetypes"
        "#,
        custom_room.id, user_id
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((custom_room, vec![slot]))
}

pub async fn update(
    custom_room_id: i32,
    label: &str,
    nb_teams: i32,
    max_player_per_team: i32,
    game_mode: Option<GameModes>,
    map: Option<Maps>,
    pool: &DbPool,
) -> DbResult<CustomRoomDAO> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        UPDATE custom_rooms
        SET label = $1, nb_teams = $2, max_player_per_team = $3,
            current_game_mode = COALESCE($4, current_game_mode),
            current_map = COALESCE($5, current_map)
        WHERE id = $6
        RETURNING id, label, user_id, nb_teams, max_player_per_team,
                  current_game_mode as "current_game_mode: GameModes", current_map as "current_map: Maps", matchmaking_ticket
        "#,
        label, nb_teams, max_player_per_team, game_mode as Option<GameModes>, map as Option<Maps>, custom_room_id
    )
    .fetch_one(pool)
    .await
}

pub async fn update_ticket(custom_room_id: i32, ticket_id: Option<Uuid>, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        "UPDATE custom_rooms SET matchmaking_ticket = $1 WHERE id = $2",
        ticket_id, custom_room_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Slots cascade-delete at the DB level (fk_custom_room ON DELETE CASCADE).
pub async fn delete_by_user_id(user_id: i32, pool: &DbPool) -> DbResult<()> {
    sqlx::query!("DELETE FROM custom_rooms WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_with_slots(id: i32, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    let custom_room = get_by_id(id, pool).await?;
    let slots = custom_room_slots::get_by_custom_room_id(id, pool).await?;

    Ok((custom_room, slots))
}

pub async fn get_by_user_id_with_slots(user_id: i32, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    let custom_room = get_by_user_id(user_id, pool).await?;
    let slots = custom_room_slots::get_by_custom_room_id(custom_room.id, pool).await?;

    Ok((custom_room, slots))
}

pub async fn get_by_ticket_id_with_slots(ticket_id: Uuid, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    let custom_room = get_by_ticket_id(ticket_id, pool).await?;
    let slots = custom_room_slots::get_by_custom_room_id(custom_room.id, pool).await?;

    Ok((custom_room, slots))
}

pub async fn get_all_with_slots(pool: &DbPool) -> DbResult<Vec<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)>> {
    let rooms = get_all(pool).await?;
    let mut result = Vec::with_capacity(rooms.len());

    for room in rooms {
        let slots = custom_room_slots::get_by_custom_room_id(room.id, pool).await?;
        result.push((room, slots));
    }

    Ok(result)
}

pub async fn get_with_users(id: i32, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<(CustomRoomSlotDAO, UserDAO)>)> {
    let (custom_room, slots) = get_with_slots(id, pool).await?;
    let user_ids: Vec<i32> = slots.iter().map(|s| s.user_id).collect();
    let fetched_users = users::get_by_ids(&user_ids, pool).await?;

    let mut tuples = Vec::with_capacity(slots.len());
    for slot in slots {
        if let Some(user) = fetched_users.iter().find(|u| u.id == slot.user_id) {
            tuples.push((slot, user.clone()));
        }
    }

    Ok((custom_room, tuples))
}
