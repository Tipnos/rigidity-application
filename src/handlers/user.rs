use serde::{Deserialize};
use utoipa::ToSchema;
use crate::chrono::{DateTime, Utc};
use axum::{extract::State, Json};
use crate::database::{self, users as user_dao};
use crate::{errors::{AppResult, AppError}};
use crate::services::{steam, auth as auth_service};

#[derive(Deserialize, ToSchema)]
pub struct CreateUserData {
    pub email: String,
    pub nickname: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: DateTime<Utc>,
    pub auth: steam::SteamAuthData,
}

#[utoipa::path(
    post,
    path = "/user/create",
    tag = "user",
    request_body = CreateUserData,
    responses(
        (status = 200, description = "User created, confirmation email sent", body = user_dao::UserDAO),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
pub async fn create(
    State(pool): State<database::DbPool>,
    Json(data): Json<CreateUserData>
) -> AppResult<Json<user_dao::UserDAO>> {
    let steam_id = auth_service::steam_authenticate_and_ownership_check(&data.auth).await?;
    let email_confirmation_hash = auth_service::new_reset_password_hash()?;

    let (user, expire_timestamp) = user_dao::create(
        &data.email,
        &data.nickname,
        &steam_id.to_string(),
        &data.first_name,
        &data.last_name,
        "Waiting for init",
        data.birth_date.naive_utc(),
        &email_confirmation_hash,
        &pool
    ).await?;

    match &user.reset_password_hash {
        Some(hash) => {
            let _r = auth_service::send_confirmation_email(&user.email, expire_timestamp, &hash).await;
            Ok(Json(user))
        }
        None => return Err(AppError::InternalServerError(
            format!("Reset password hash was not set up properly.")))
    }
}
