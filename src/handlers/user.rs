use serde::{Deserialize};
use crate::chrono::{DateTime, Utc};
use actix_web::{HttpResponse, web};
use crate::database::{self, users as user_dao};
use crate::{errors::{AppResult, AppError}};
use crate::services::{steam, auth as auth_service};

#[derive(Deserialize)]
pub struct CreateUserData {
    pub email: String,
    pub nickname: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: DateTime<Utc>,
    pub auth: steam::SteamAuthData,
}

pub async fn create(
    create_data: web::Json<CreateUserData>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let steam_id = auth_service::steam_authenticate_and_ownership_check(&create_data.auth).await?;
    let data = create_data.into_inner();
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
            Ok(HttpResponse::Ok().json(user))
        }
        None => return Err(AppError::InternalServerError(
            format!("Reset password hash was not set up properly.")))
    }
}
