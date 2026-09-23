use axum::{extract::State, Json};
use crate::database::{self, users as user_dao};
use crate::dto::{input::CreateUserDTO, output::UserDTO};
use crate::errors::AppResult;
use crate::services::steam;

#[utoipa::path(
    post,
    path = "/user/create",
    tag = "user",
    request_body = CreateUserDTO,
    responses(
        (status = 200, description = "User created", body = UserDTO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Steam authentication failed", body = String),
    )
)]
pub async fn create(
    State(pool): State<database::DbPool>,
    Json(data): Json<CreateUserDTO>
) -> AppResult<Json<UserDTO>> {
    let steam_id = steam::authenticate_and_check_ownership(&data.auth.into()).await?;

    let user = user_dao::create(
        &data.nickname,
        &steam_id.to_string(),
        &data.first_name,
        &data.last_name,
        data.birth_date.naive_utc(),
        &pool
    ).await?;

    Ok(Json(UserDTO::from(user)))
}
