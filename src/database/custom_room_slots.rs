use sqlx::PgExecutor;
use uuid::Uuid;

use crate::enums::Archetypes;

use super::{DbPool, DbResult};
use super::custom_rooms::CustomRoomDAO;

// A (team, team_position) pair inside a custom room.
#[derive(Debug, Clone, Copy)]
pub struct SlotPosition {
    pub team: i32,
    pub team_position: i32,
}

// The room and user keys are resolved to their UUIDs by the queries.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CustomRoomSlotDAO {
    pub id: Uuid,
    pub custom_room_id: Uuid,
    pub team: i32,
    pub team_position: i32,
    pub user_id: Uuid,
    pub current_archetype: Archetypes,
}

// TODO: `CustomRoomSlotDAO::get_gamelift_attributes` used to build the AWS
// FlexMatch player attributes here (team, team_position, archetype as numbers;
// nickname as a string).

pub async fn get_by_custom_room(custom_room: &CustomRoomDAO, pool: &DbPool) -> DbResult<Vec<CustomRoomSlotDAO>> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        SELECT s.id, r.id AS custom_room_id, s.team, s.team_position, u.id AS user_id,
               s.current_archetype as "current_archetype: Archetypes"
        FROM custom_room_slots s
        JOIN custom_rooms r ON r.pk = s.custom_room_pk
        JOIN users u ON u.pk = s.user_pk
        WHERE s.custom_room_pk = $1
        "#,
        custom_room.pk
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_position(
    custom_room: &CustomRoomDAO,
    team: i32,
    team_position: i32,
    pool: &DbPool,
) -> DbResult<CustomRoomSlotDAO> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        SELECT s.id, r.id AS custom_room_id, s.team, s.team_position, u.id AS user_id,
               s.current_archetype as "current_archetype: Archetypes"
        FROM custom_room_slots s
        JOIN custom_rooms r ON r.pk = s.custom_room_pk
        JOIN users u ON u.pk = s.user_pk
        WHERE s.custom_room_pk = $1 AND s.team = $2 AND s.team_position = $3
        "#,
        custom_room.pk, team, team_position
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_user_id(user_id: Uuid, pool: &DbPool) -> DbResult<CustomRoomSlotDAO> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        SELECT s.id, r.id AS custom_room_id, s.team, s.team_position, u.id AS user_id,
               s.current_archetype as "current_archetype: Archetypes"
        FROM custom_room_slots s
        JOIN custom_rooms r ON r.pk = s.custom_room_pk
        JOIN users u ON u.pk = s.user_pk
        WHERE u.id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
}

// Takes any executor so `custom_rooms::create_with_owner_slot` can run it in
// its transaction.
pub async fn create<'e>(
    custom_room: &CustomRoomDAO,
    team: i32,
    team_position: i32,
    user_id: Uuid,
    archetype: Archetypes,
    executor: impl PgExecutor<'e>,
) -> DbResult<CustomRoomSlotDAO> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        WITH s AS (
            INSERT INTO custom_room_slots (custom_room_pk, team, team_position, user_pk, current_archetype)
            VALUES ($1, $2, $3, (SELECT pk FROM users WHERE id = $4), $5)
            RETURNING *
        )
        SELECT s.id, r.id AS custom_room_id, s.team, s.team_position, u.id AS user_id,
               s.current_archetype as "current_archetype: Archetypes"
        FROM s
        JOIN custom_rooms r ON r.pk = s.custom_room_pk
        JOIN users u ON u.pk = s.user_pk
        "#,
        custom_room.pk, team, team_position, user_id, archetype as Archetypes
    )
    .fetch_one(executor)
    .await
}

// Locks every slot row of the room before moving `user_id` into a new
// position, mirroring the diesel `for_update()` row lock this replaces.
pub async fn update_position(
    user_id: Uuid,
    custom_room: &CustomRoomDAO,
    team: i32,
    team_position: i32,
    pool: &DbPool,
) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "SELECT pk FROM custom_room_slots WHERE custom_room_pk = $1 FOR UPDATE",
        custom_room.pk
    )
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        UPDATE custom_room_slots SET custom_room_pk = $1, team = $2, team_position = $3
        WHERE user_pk = (SELECT pk FROM users WHERE id = $4)
        "#,
        custom_room.pk, team, team_position, user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn update_archetype_by_user_id(user_id: Uuid, archetype: Archetypes, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        r#"
        UPDATE custom_room_slots SET current_archetype = $1
        WHERE user_pk = (SELECT pk FROM users WHERE id = $2)
        "#,
        archetype as Archetypes, user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_by_user_id(user_id: Uuid, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        "DELETE FROM custom_room_slots WHERE user_pk = (SELECT pk FROM users WHERE id = $1)",
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
