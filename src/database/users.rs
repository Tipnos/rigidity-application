use chrono::NaiveDateTime;
use uuid::Uuid;

use super::{DbPool, DbResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserDAO {
    pub id: Uuid,
    pub nickname: String,
    pub created_at: NaiveDateTime,
    pub steam_id: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: NaiveDateTime,
}

pub async fn get_by_id(id: Uuid, pool: &DbPool) -> DbResult<UserDAO> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, nickname, created_at, steam_id, first_name, last_name, birth_date
        FROM users WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_ids(ids: &[Uuid], pool: &DbPool) -> DbResult<Vec<UserDAO>> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, nickname, created_at, steam_id, first_name, last_name, birth_date
        FROM users WHERE id = ANY($1)
        "#,
        ids
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_steam_id(steam_id: &str, pool: &DbPool) -> DbResult<UserDAO> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, nickname, created_at, steam_id, first_name, last_name, birth_date
        FROM users WHERE steam_id = $1
        "#,
        steam_id
    )
    .fetch_one(pool)
    .await
}

pub async fn create(
    nickname: &str,
    steam_id: &str,
    first_name: &str,
    last_name: &str,
    birth_date: NaiveDateTime,
    pool: &DbPool,
) -> DbResult<UserDAO> {
    sqlx::query_as!(
        UserDAO,
        r#"
        INSERT INTO users (nickname, steam_id, first_name, last_name, birth_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, nickname, created_at, steam_id, first_name, last_name, birth_date
        "#,
        nickname, steam_id, first_name, last_name, birth_date
    )
    .fetch_one(pool)
    .await
}
