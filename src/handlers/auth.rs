use actix_identity::Identity;
use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use actix_web::http::StatusCode;
use serde::Deserialize;
use crate::errors::{AppResult, AppError};
use crate::database::{self, users as user_dao};
use crate::services::{email::EmailService, steam::SteamAuthData};
use chrono::NaiveDateTime;
use crate::app_conf::get_base_url;
use crate::services::{steam, auth as auth_service};

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String
}

pub async fn login(
    request: HttpRequest,
    auth_data: web::Json<AuthData>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let user = t_login(auth_data, pool).await?;

    if let Err(err) = Identity::login(&request.extensions(), user.id.to_string()) {
        return Err(AppError::InternalServerError(err.to_string()))
    }

    Ok(HttpResponse::Ok().json(user))
}

async fn t_login(
    auth_data: web::Json<AuthData>,
    pool: web::Data<database::DbPool>
) -> AppResult<user_dao::UserDAO> {
    let datas = auth_data.into_inner();
    let email = datas.email.clone();
    match user_dao::get_by_email(&email, &pool).await {
        Ok(user) => {
            if !user.can_login() {
                return Err(AppError::Forbidden);
            }
            
            if user.is_password_ok(&datas.password)? {
                return Ok(user);
            }
        }
        Err(_err) => {
            return Err(AppError::BadRequest(String::from("Unknown email.")));
        }
    }

    Err(AppError::BadRequest(String::from("Incorrect password.")))
}

pub async fn login_steam(
    request: HttpRequest,
    auth_data: web::Json<SteamAuthData>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let steam_id = steam::authenticate_user_ticket(&auth_data).await?;

    let user = user_dao::get_by_steam_id(&steam_id.to_string(), &pool).await?;

    steam::check_app_ownership(&auth_data.app_id, &steam_id).await?;
    if user.can_login() {
        if let Err(err) = Identity::login(&request.extensions(), user.id.to_string()) {
            return Err(AppError::InternalServerError(err.to_string()))
        }
        return Ok(HttpResponse::Ok().json(user));
    }
    
    Ok(HttpResponse::Forbidden().json(user))
}

pub async fn logout(
    id: Identity,
) -> AppResult<HttpResponse> {    
    id.logout();
    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Deserialize)]
pub struct AskPassData {
    pub email: String
}

pub async fn ask_password_reset(
    data: web::Json<AskPassData>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    match t_ask_password_reset(data, pool).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(err) => Err(err)
    }
}

async fn t_ask_password_reset(
    data: web::Json<AskPassData>,
    pool: web::Data<database::DbPool>
) -> AppResult<()> {
    let hash = auth_service::new_reset_password_hash()?;
    let result  = user_dao::update_reset_password_hash(
        &data.email,
        &hash,
        &pool
    ).await;

    match result {
        Ok(expire_time) => {
            let url = format!("{}/static/reset_password.html?id={}", get_base_url(), hash);
            let expire_time = NaiveDateTime::from_timestamp_opt(expire_time, 0).unwrap()
                .format("%c");
            let link = format!("<h1>Hello !</h1><br/><p>Here's your link: {}.</p><p>Your link we'll expire at {} (UTC time)</p>", url, expire_time);
    
            let email_service = EmailService::new(
                &data.email,
                String::from("Rigidity password reset"),
                link
            );
            
            match email_service.send().await {
                Ok(_response) => {
                    Ok(())
                }
                Err(err) => {
                    Err(AppError::ServiceUnavailable(format!("Email service unavailable. Message: {}", err)))
                }
            }
        }
        Err(err) => {
            return Err(AppError::InternalServerError(format!("Database error: {}", err.to_string())));
        }
    } 
}

#[derive(Debug, Deserialize)]
pub struct ResetPassData {
    pub hash: String,
    pub new_password: String
}

pub async fn reset_password(
    data: web::Json<ResetPassData>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    t_reset_password(data, pool).await?;

    Ok(HttpResponse::TemporaryRedirect()
        .status(StatusCode::SEE_OTHER)
        .insert_header((
            "Location",
            "/login.html"))
        .finish())
}

async fn t_reset_password(
    data: web::Json<ResetPassData>,
    pool: web::Data<database::DbPool>
) -> AppResult<()> {
    auth_service::check_reset_password_hash(&data.hash, &pool).await?;
    match auth_service::hash_password(&data.new_password) {
        Ok(new_hash) => user_dao::update_password(&data.hash, &new_hash, &pool).await?,
        Err(err) => return Err(AppError::InternalServerError(err.to_string()))
    }

    Ok(())
}

pub async fn refresh_cookie(
    request: HttpRequest,
    id: Identity,
) -> AppResult<HttpResponse> {
    let user_id = id.id().unwrap();
    id.logout();

    if let Err(err) = Identity::login(&request.extensions(), user_id) {
        return Err(AppError::InternalServerError(err.to_string()))
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct EmailConfirmationData {
    pub hash: String
}

pub async fn email_confirmation(
    data: web::Json<EmailConfirmationData>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    auth_service::email_confirmation(&data.hash, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub struct UpdateEmailConfirmationData {
    pub email: String,
    pub auth: steam::SteamAuthData
}

pub async fn update_email_confirmation(
    data: web::Json<UpdateEmailConfirmationData>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let steam_id = auth_service::steam_authenticate_and_ownership_check(&data.auth).await?;

    auth_service::update_email_confirmation(
        data.email.clone(), steam_id, pool).await?;
    
    Ok(HttpResponse::Ok().finish())
}