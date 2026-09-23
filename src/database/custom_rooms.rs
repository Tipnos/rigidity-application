use uuid::Uuid;

use crate::enums::{Archetypes, GameModes, Maps};

use super::{DbPool, DbResult};
use super::custom_room_slots::{self, CustomRoomSlotDAO};
use super::users::{self, UserDAO};

// `pk` is only visible to the `database` module: callers pass the DAO back to
// the room's queries, which use it instead of looking the room up again.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CustomRoomDAO {
    pub(super) pk: i32,
    pub id: Uuid,
    pub label: String,
    pub user_id: Uuid,
    pub nb_teams: i32,
    pub max_player_per_team: i32,
    pub current_game_mode: GameModes,
    pub current_map: Maps,
    pub matchmaking_ticket: Option<Uuid>,
}

// Settings chosen by the owner when creating or updating a room. `None` game
// mode / map means the DB default (create) or the current value (update).
#[derive(Debug, Clone)]
pub struct CustomRoomSettings {
    pub label: String,
    pub nb_teams: i32,
    pub max_player_per_team: i32,
    pub game_mode: Option<GameModes>,
    pub map: Option<Maps>,
}

impl CustomRoomDAO {
    pub fn get_capacity(&self) -> usize {
        (self.nb_teams * self.max_player_per_team) as usize
    }

    pub fn is_valid_slot(&self, team: &i32, team_position: &i32) -> bool {
        *team < self.nb_teams && *team_position < self.max_player_per_team
    }

    // TODO: `get_start_matchmaking_input` used to build the AWS GameLift
    // `StartMatchmakingInput` here (one player per slot with its FlexMatch
    // attributes, configuration_name = current map, ticket_id).
}

pub async fn get_by_id(id: Uuid, pool: &DbPool) -> DbResult<CustomRoomDAO> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        SELECT r.pk, r.id, r.label, u.id AS user_id, r.nb_teams, r.max_player_per_team,
               r.current_game_mode as "current_game_mode: GameModes", r.current_map as "current_map: Maps", r.matchmaking_ticket
        FROM custom_rooms r JOIN users u ON u.pk = r.user_pk
        WHERE r.id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_user_id(user_id: Uuid, pool: &DbPool) -> DbResult<CustomRoomDAO> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        SELECT r.pk, r.id, r.label, u.id AS user_id, r.nb_teams, r.max_player_per_team,
               r.current_game_mode as "current_game_mode: GameModes", r.current_map as "current_map: Maps", r.matchmaking_ticket
        FROM custom_rooms r JOIN users u ON u.pk = r.user_pk
        WHERE u.id = $1
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
        SELECT r.pk, r.id, r.label, u.id AS user_id, r.nb_teams, r.max_player_per_team,
               r.current_game_mode as "current_game_mode: GameModes", r.current_map as "current_map: Maps", r.matchmaking_ticket
        FROM custom_rooms r JOIN users u ON u.pk = r.user_pk
        WHERE r.matchmaking_ticket = $1
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
        SELECT r.pk, r.id, r.label, u.id AS user_id, r.nb_teams, r.max_player_per_team,
               r.current_game_mode as "current_game_mode: GameModes", r.current_map as "current_map: Maps", r.matchmaking_ticket
        FROM custom_rooms r JOIN users u ON u.pk = r.user_pk
        "#
    )
    .fetch_all(pool)
    .await
}

// Inserts the room and its owner's first slot (team 0, position 0, leader) in
// one transaction, replacing the separate insert + `SELECT id ORDER BY id DESC`
// diesel used to locate the new room's id.
pub async fn create_with_owner_slot(
    settings: &CustomRoomSettings,
    user_id: Uuid,
    pool: &DbPool,
) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    // Mirrors the DB column defaults (init migration): current_game_mode
    // defaults to 'king_of_the_hill', current_map to 'inferno'.
    let game_mode = settings.game_mode.unwrap_or(GameModes::KingOfTheHill);
    let map = settings.map.unwrap_or(Maps::Inferno);

    let mut tx = pool.begin().await?;

    let custom_room = sqlx::query_as!(
        CustomRoomDAO,
        r#"
        WITH r AS (
            INSERT INTO custom_rooms (label, user_pk, nb_teams, max_player_per_team, current_game_mode, current_map)
            VALUES ($1, (SELECT pk FROM users WHERE id = $2), $3, $4, $5, $6)
            RETURNING *
        )
        SELECT r.pk, r.id, r.label, u.id AS user_id, r.nb_teams, r.max_player_per_team,
               r.current_game_mode as "current_game_mode: GameModes", r.current_map as "current_map: Maps", r.matchmaking_ticket
        FROM r JOIN users u ON u.pk = r.user_pk
        "#,
        settings.label, user_id, settings.nb_teams, settings.max_player_per_team, game_mode as GameModes, map as Maps
    )
    .fetch_one(&mut *tx)
    .await?;

    let slot = custom_room_slots::create(&custom_room, 0, 0, user_id, Archetypes::Leader, &mut *tx).await?;

    tx.commit().await?;

    Ok((custom_room, vec![slot]))
}

pub async fn update(
    custom_room: &CustomRoomDAO,
    settings: &CustomRoomSettings,
    pool: &DbPool,
) -> DbResult<CustomRoomDAO> {
    sqlx::query_as!(
        CustomRoomDAO,
        r#"
        UPDATE custom_rooms r
        SET label = $1, nb_teams = $2, max_player_per_team = $3,
            current_game_mode = COALESCE($4, r.current_game_mode),
            current_map = COALESCE($5, r.current_map)
        FROM users u
        WHERE r.pk = $6 AND u.pk = r.user_pk
        RETURNING r.pk, r.id, r.label, u.id AS user_id, r.nb_teams, r.max_player_per_team,
                  r.current_game_mode as "current_game_mode: GameModes", r.current_map as "current_map: Maps", r.matchmaking_ticket
        "#,
        settings.label, settings.nb_teams, settings.max_player_per_team, settings.game_mode as Option<GameModes>, settings.map as Option<Maps>, custom_room.pk
    )
    .fetch_one(pool)
    .await
}

pub async fn update_ticket(custom_room: &CustomRoomDAO, ticket_id: Option<Uuid>, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        "UPDATE custom_rooms SET matchmaking_ticket = $1 WHERE pk = $2",
        ticket_id, custom_room.pk
    )
    .execute(pool)
    .await?;

    Ok(())
}

// Slots cascade-delete at the DB level (fk_custom_room ON DELETE CASCADE).
pub async fn delete(custom_room: &CustomRoomDAO, pool: &DbPool) -> DbResult<()> {
    sqlx::query!("DELETE FROM custom_rooms WHERE pk = $1", custom_room.pk)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_with_slots(id: Uuid, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    let custom_room = get_by_id(id, pool).await?;
    let slots = custom_room_slots::get_by_custom_room(&custom_room, pool).await?;

    Ok((custom_room, slots))
}

pub async fn get_by_user_id_with_slots(user_id: Uuid, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    let custom_room = get_by_user_id(user_id, pool).await?;
    let slots = custom_room_slots::get_by_custom_room(&custom_room, pool).await?;

    Ok((custom_room, slots))
}

pub async fn get_by_ticket_id_with_slots(ticket_id: Uuid, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)> {
    let custom_room = get_by_ticket_id(ticket_id, pool).await?;
    let slots = custom_room_slots::get_by_custom_room(&custom_room, pool).await?;

    Ok((custom_room, slots))
}

pub async fn get_all_with_slots(pool: &DbPool) -> DbResult<Vec<(CustomRoomDAO, Vec<CustomRoomSlotDAO>)>> {
    let rooms = get_all(pool).await?;
    let mut result = Vec::with_capacity(rooms.len());

    for room in rooms {
        let slots = custom_room_slots::get_by_custom_room(&room, pool).await?;
        result.push((room, slots));
    }

    Ok(result)
}

pub async fn get_with_users(id: Uuid, pool: &DbPool) -> DbResult<(CustomRoomDAO, Vec<(CustomRoomSlotDAO, UserDAO)>)> {
    let (custom_room, slots) = get_with_slots(id, pool).await?;
    let user_ids: Vec<Uuid> = slots.iter().map(|s| s.user_id).collect();
    let fetched_users = users::get_by_ids(&user_ids, pool).await?;

    let mut tuples = Vec::with_capacity(slots.len());
    for slot in slots {
        if let Some(user) = fetched_users.iter().find(|u| u.id == slot.user_id) {
            tuples.push((slot, user.clone()));
        }
    }

    Ok((custom_room, tuples))
}
