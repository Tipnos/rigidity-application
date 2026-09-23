use crate::enums::Archetypes;

use super::{DbPool, DbResult};

// A (team, team_position) pair inside a custom room.
#[derive(Debug, Clone, Copy)]
pub struct SlotPosition {
    pub team: i32,
    pub team_position: i32,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CustomRoomSlotDAO {
    pub id: i32,
    pub custom_room_id: i32,
    pub team: i32,
    pub team_position: i32,
    pub user_id: i32,
    pub current_archetype: Archetypes,
}

// TODO: `CustomRoomSlotDAO::get_gamelift_attributes` used to build the AWS
// FlexMatch player attributes here (team, team_position, archetype as numbers;
// nickname as a string).

pub async fn get_by_custom_room_id(custom_room_id: i32, pool: &DbPool) -> DbResult<Vec<CustomRoomSlotDAO>> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        SELECT id, custom_room_id, team, team_position, user_id, current_archetype as "current_archetype: Archetypes"
        FROM custom_room_slots WHERE custom_room_id = $1
        "#,
        custom_room_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_position(
    custom_room_id: i32,
    team: i32,
    team_position: i32,
    pool: &DbPool,
) -> DbResult<CustomRoomSlotDAO> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        SELECT id, custom_room_id, team, team_position, user_id, current_archetype as "current_archetype: Archetypes"
        FROM custom_room_slots WHERE custom_room_id = $1 AND team = $2 AND team_position = $3
        "#,
        custom_room_id, team, team_position
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_user_id(user_id: i32, pool: &DbPool) -> DbResult<CustomRoomSlotDAO> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        SELECT id, custom_room_id, team, team_position, user_id, current_archetype as "current_archetype: Archetypes"
        FROM custom_room_slots WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
}

pub async fn create(
    custom_room_id: i32,
    team: i32,
    team_position: i32,
    user_id: i32,
    archetype: Archetypes,
    pool: &DbPool,
) -> DbResult<CustomRoomSlotDAO> {
    sqlx::query_as!(
        CustomRoomSlotDAO,
        r#"
        INSERT INTO custom_room_slots (custom_room_id, team, team_position, user_id, current_archetype)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, custom_room_id, team, team_position, user_id, current_archetype as "current_archetype: Archetypes"
        "#,
        custom_room_id, team, team_position, user_id, archetype as Archetypes
    )
    .fetch_one(pool)
    .await
}

// Locks every slot row of the room before moving `user_id` into a new
// position, mirroring the diesel `for_update()` row lock this replaces.
pub async fn update_position(
    user_id: i32,
    custom_room_id: i32,
    team: i32,
    team_position: i32,
    pool: &DbPool,
) -> DbResult<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "SELECT id FROM custom_room_slots WHERE custom_room_id = $1 FOR UPDATE",
        custom_room_id
    )
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE custom_room_slots SET custom_room_id = $1, team = $2, team_position = $3 WHERE user_id = $4",
        custom_room_id, team, team_position, user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

pub async fn update_archetype_by_user_id(user_id: i32, archetype: Archetypes, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        "UPDATE custom_room_slots SET current_archetype = $1 WHERE user_id = $2",
        archetype as Archetypes, user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_by_user_id(user_id: i32, pool: &DbPool) -> DbResult<()> {
    sqlx::query!("DELETE FROM custom_room_slots WHERE user_id = $1", user_id)
        .execute(pool)
        .await?;

    Ok(())
}
