use serde::{Deserialize};
use utoipa::ToSchema;
use crate::chrono::{DateTime, Utc};
use axum::{extract::State, Json};
use crate::database::{self, users as user_dao};
use crate::errors::AppResult;
use crate::services::steam;

#[derive(Deserialize, ToSchema)]
pub struct CreateUserData {
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
        (status = 200, description = "User created", body = user_dao::UserDAO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Steam authentication failed", body = String),
    )
)]
pub async fn create(
    State(pool): State<database::DbPool>,
    Json(data): Json<CreateUserData>
) -> AppResult<Json<user_dao::UserDAO>> {
    let steam_id = steam::authenticate_and_check_ownership(&data.auth).await?;

    let user = user_dao::create(
        &data.nickname,
        &steam_id.to_string(),
        &data.first_name,
        &data.last_name,
        data.birth_date.naive_utc(),
        &pool
    ).await?;

    Ok(Json(user))
}
