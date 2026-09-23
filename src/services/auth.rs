use argon2::Config;
use rand::Rng;
use crate::app_conf::config;
use crate::errors::{AppResult, AppError};
use crate::app_conf::get_base_url;
use crate::services::email::EmailService;
use chrono::{Utc, NaiveDateTime};
use crate::database::{self, users as user_dao};
use crate::services::steam;

pub fn new_reset_password_hash() -> AppResult<String> {
    let rng = rand::thread_rng().gen::<i64>().to_string();
    hash_password(&rng)
}

pub fn hash_password(to_hash: &str) -> AppResult<String> {
    let argon_config = Config {
        secret: config().secret_key.as_bytes(),
        ..Default::default()
    };
    
    argon2::hash_encoded(to_hash.as_bytes(), config().secret_key.as_bytes(), &argon_config)
        .map_err(|err| {
        AppError::InternalServerError(err.to_string())
    })
}

pub async fn check_reset_password_hash(hash: &str, pool: &database::DbPool) -> AppResult<()> {
    let expired_error = Err(AppError::BadRequest(String::from("The link you used has expired. Make a new request.")));

    let user = user_dao::get_by_reset_password_hash(hash, pool).await?;
    if let Some(expire_date) = user.password_hash_expire_at {
        let now = NaiveDateTime::from_timestamp_opt(Utc::now().timestamp(), 0).unwrap();
        if expire_date >= now {
            Ok(())
        } else {
            if let Err(err) = user_dao::cancel_reset_password_hash(&hash, pool).await {
                return Err(AppError::BadRequest(err.to_string()));
            }
            return expired_error;
        }
    } else {
        return expired_error;
    }
}

pub async fn send_confirmation_email(email: &str, expire_timestamp: i64, hash: &str) -> AppResult<()> {
    let url = format!("{}/static/email_confirmation.html?id={}", get_base_url(), hash);
    let expire_time = NaiveDateTime::from_timestamp_opt(
        expire_timestamp, 0).unwrap().format("%c");
    let studio_logo_url = format!("{}/static/assets/images/logo_studio.png", 
        get_base_url());
    let link = format!("<p>Hello, </p><p>Welcome to rigidity!</p><p>Please click on the following link to confirm you email address: <a href='{}'>confirm link</a></p><p>Your link we'll expire at {} (UTC time)</p></br></br><img src='{}'>", url, expire_time, studio_logo_url);

    let email_service = EmailService::new(
        email,
        String::from("Rigidity email confirmation"),
        link
    );
    
    email_service.send().await?;
    Ok(())
}

pub async fn email_confirmation(hash: &str, pool: &database::DbPool) -> AppResult<()> {
    check_reset_password_hash(hash, pool).await?;
    user_dao::confirm_email(hash, pool).await?;

    Ok(())
}

pub async fn update_email_confirmation(
    email: String, steam_id: u64, pool: &database::DbPool) -> AppResult<()> {
    let user = user_dao::get_by_steam_id(&steam_id.to_string(), pool).await?;

    if !user.email_confirmation_required {
        return Err(AppError::Forbidden);
    }

    let email_confirmation_hash = new_reset_password_hash()?;
    let (_user, expire_time_stamp) = user_dao::update_email(
        &email, &steam_id, &email_confirmation_hash, pool).await?;
    send_confirmation_email(&email, expire_time_stamp, &email_confirmation_hash).await?;

    Ok(())
}

pub async fn steam_authenticate_and_ownership_check(
    data: &steam::SteamAuthData) -> AppResult<u64> {
    let steam_id = steam::authenticate_user_ticket(data).await?;
    steam::check_app_ownership(&data.app_id, &steam_id).await?; 

    Ok(steam_id)
}