use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app_conf::SECRET_KEY;
use crate::errors::{AppError, AppResult};

use super::{DbPool, DbResult};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserDAO {
    pub id: i32,
    pub email: String,
    pub nickname: String,
    #[serde(skip_serializing)]
    pub hash: String,
    #[serde(skip_serializing)]
    pub reset_password_hash: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash_expire_at: Option<NaiveDateTime>,
    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,
    pub steam_id: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: NaiveDateTime,
    #[serde(skip_serializing)]
    pub email_confirmation_required: bool,
}

impl UserDAO {
    pub fn is_password_ok(&self, password: &str) -> AppResult<bool> {
        argon2::verify_encoded_ext(
            &self.hash,
            password.as_bytes(),
            SECRET_KEY.as_bytes(),
            &[])
            .map_err(|err| {
                dbg!(err);
                AppError::Unauthorized
            })
    }

    pub fn can_login(&self) -> bool {
        !self.email_confirmation_required
    }
}

fn reset_password_hash_expiry() -> (i64, Option<NaiveDateTime>) {
    let timestamp = (Utc::now() + Duration::hours(4)).timestamp();
    (timestamp, NaiveDateTime::from_timestamp_opt(timestamp, 0))
}

pub async fn get(id: i32, pool: &DbPool) -> DbResult<UserDAO> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, email, nickname, hash, reset_password_hash, password_hash_expire_at,
               created_at, steam_id, first_name, last_name, birth_date, email_confirmation_required
        FROM users WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_ids(ids: &[i32], pool: &DbPool) -> DbResult<Vec<UserDAO>> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, email, nickname, hash, reset_password_hash, password_hash_expire_at,
               created_at, steam_id, first_name, last_name, birth_date, email_confirmation_required
        FROM users WHERE id = ANY($1)
        "#,
        ids
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_email(email: &str, pool: &DbPool) -> DbResult<UserDAO> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, email, nickname, hash, reset_password_hash, password_hash_expire_at,
               created_at, steam_id, first_name, last_name, birth_date, email_confirmation_required
        FROM users WHERE email = $1
        "#,
        email
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_steam_id(steam_id: &str, pool: &DbPool) -> DbResult<UserDAO> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, email, nickname, hash, reset_password_hash, password_hash_expire_at,
               created_at, steam_id, first_name, last_name, birth_date, email_confirmation_required
        FROM users WHERE steam_id = $1
        "#,
        steam_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_reset_password_hash(hash: &str, pool: &DbPool) -> DbResult<UserDAO> {
    sqlx::query_as!(
        UserDAO,
        r#"
        SELECT id, email, nickname, hash, reset_password_hash, password_hash_expire_at,
               created_at, steam_id, first_name, last_name, birth_date, email_confirmation_required
        FROM users WHERE reset_password_hash = $1
        "#,
        hash
    )
    .fetch_one(pool)
    .await
}

pub async fn create(
    email: &str,
    nickname: &str,
    steam_id: &str,
    first_name: &str,
    last_name: &str,
    hash: &str,
    birth_date: NaiveDateTime,
    email_confirmation_hash: &str,
    pool: &DbPool,
) -> DbResult<(UserDAO, i64)> {
    let (expire_timestamp, expire_at) = reset_password_hash_expiry();

    let user = sqlx::query_as!(
        UserDAO,
        r#"
        INSERT INTO users (email, nickname, steam_id, first_name, last_name, hash, birth_date, reset_password_hash, password_hash_expire_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, email, nickname, hash, reset_password_hash, password_hash_expire_at,
                  created_at, steam_id, first_name, last_name, birth_date, email_confirmation_required
        "#,
        email, nickname, steam_id, first_name, last_name, hash, birth_date, email_confirmation_hash, expire_at
    )
    .fetch_one(pool)
    .await?;

    Ok((user, expire_timestamp))
}

pub async fn update_reset_password_hash(email: &str, hash: &str, pool: &DbPool) -> DbResult<i64> {
    let (expire_timestamp, expire_at) = reset_password_hash_expiry();

    sqlx::query!(
        "UPDATE users SET reset_password_hash = $1, password_hash_expire_at = $2 WHERE email = $3",
        hash, expire_at, email
    )
    .execute(pool)
    .await?;

    Ok(expire_timestamp)
}

pub async fn update_email(
    new_email: &str,
    steam_id: &u64,
    email_confirmation_hash: &str,
    pool: &DbPool,
) -> DbResult<(UserDAO, i64)> {
    let (expire_timestamp, expire_at) = reset_password_hash_expiry();
    let steam_id = steam_id.to_string();

    let user = sqlx::query_as!(
        UserDAO,
        r#"
        UPDATE users
        SET email = $1, reset_password_hash = $2, password_hash_expire_at = $3
        WHERE steam_id = $4
        RETURNING id, email, nickname, hash, reset_password_hash, password_hash_expire_at,
                  created_at, steam_id, first_name, last_name, birth_date, email_confirmation_required
        "#,
        new_email, email_confirmation_hash, expire_at, steam_id
    )
    .fetch_one(pool)
    .await?;

    Ok((user, expire_timestamp))
}

pub async fn update_password(reset_password_hash: &str, new_hash: &str, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET reset_password_hash = NULL, password_hash_expire_at = NULL, hash = $1
        WHERE reset_password_hash = $2
        "#,
        new_hash, reset_password_hash
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn cancel_reset_password_hash(hash: &str, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET reset_password_hash = NULL, password_hash_expire_at = NULL
        WHERE reset_password_hash = $1
        "#,
        hash
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn confirm_email(reset_password_hash: &str, pool: &DbPool) -> DbResult<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET reset_password_hash = NULL, password_hash_expire_at = NULL, email_confirmation_required = false
        WHERE reset_password_hash = $1
        "#,
        reset_password_hash
    )
    .execute(pool)
    .await?;

    Ok(())
}
