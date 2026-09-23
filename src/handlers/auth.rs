use axum::{extract::State, http::{header, StatusCode}, response::IntoResponse, Json};
use axum_extra::extract::cookie::PrivateCookieJar;
use serde::Deserialize;
use utoipa::ToSchema;
use crate::errors::{AppResult, AppError};
use crate::database::{self, users as user_dao};
use crate::services::{email::EmailService, steam::SteamAuthData};
use chrono::NaiveDateTime;
use crate::app_conf::get_base_url;
use crate::services::{steam, auth as auth_service};
use super::identity::{self, Identity};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthData {
    pub email: String,
    pub password: String
}

#[utoipa::path(
    post,
    path = "/login",
    tag = "auth",
    request_body = AuthData,
    responses(
        (status = 200, description = "Logged in, identity cookie set", body = user_dao::UserDAO),
        (status = 400, description = "Bad request", body = String),
        (status = 403, description = "Forbidden", body = String),
    )
)]
pub async fn login(
    jar: PrivateCookieJar,
    State(pool): State<database::DbPool>,
    Json(auth_data): Json<AuthData>
) -> AppResult<impl IntoResponse> {
    let user = t_login(auth_data, &pool).await?;
    let jar = identity::login(jar, user.id);

    Ok((jar, Json(user)))
}

async fn t_login(
    datas: AuthData,
    pool: &database::DbPool
) -> AppResult<user_dao::UserDAO> {
    let email = datas.email.clone();
    match user_dao::get_by_email(&email, pool).await {
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

#[utoipa::path(
    post,
    path = "/login-steam",
    tag = "auth",
    request_body = SteamAuthData,
    responses(
        (status = 200, description = "Logged in, identity cookie set", body = user_dao::UserDAO),
        (status = 403, description = "Email confirmation required", body = user_dao::UserDAO),
        (status = 400, description = "Bad request", body = String),
    )
)]
pub async fn login_steam(
    jar: PrivateCookieJar,
    State(pool): State<database::DbPool>,
    Json(auth_data): Json<SteamAuthData>
) -> AppResult<impl IntoResponse> {
    let steam_id = steam::authenticate_user_ticket(&auth_data).await?;

    let user = user_dao::get_by_steam_id(&steam_id.to_string(), &pool).await?;

    steam::check_app_ownership(&auth_data.app_id, &steam_id).await?;
    if user.can_login() {
        let jar = identity::login(jar, user.id);
        return Ok((StatusCode::OK, jar, Json(user)));
    }
    
    Ok((StatusCode::FORBIDDEN, jar, Json(user)))
}

#[utoipa::path(
    post,
    path = "/logout",
    tag = "auth",
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Logged out, identity cookie removed"),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn logout(
    _: Identity,
    jar: PrivateCookieJar,
) -> AppResult<impl IntoResponse> {    
    Ok((identity::logout(jar), StatusCode::OK))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AskPassData {
    pub email: String
}

#[utoipa::path(
    post,
    path = "/password",
    tag = "auth",
    request_body = AskPassData,
    responses(
        (status = 200, description = "Password reset email sent"),
        (status = 500, description = "Internal server error", body = String),
        (status = 503, description = "Service unavailable", body = String),
    )
)]
pub async fn ask_password_reset(
    State(pool): State<database::DbPool>,
    Json(data): Json<AskPassData>
) -> AppResult<StatusCode> {
    match t_ask_password_reset(data, &pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(err) => Err(err)
    }
}

async fn t_ask_password_reset(
    data: AskPassData,
    pool: &database::DbPool
) -> AppResult<()> {
    let hash = auth_service::new_reset_password_hash()?;
    let result  = user_dao::update_reset_password_hash(
        &data.email,
        &hash,
        pool
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPassData {
    pub hash: String,
    pub new_password: String
}

#[utoipa::path(
    put,
    path = "/password",
    tag = "auth",
    request_body = ResetPassData,
    responses(
        (status = 303, description = "Password updated, redirects to the login page"),
        (status = 400, description = "Bad request", body = String),
        (status = 500, description = "Internal server error", body = String),
    )
)]
pub async fn reset_password(
    State(pool): State<database::DbPool>,
    Json(data): Json<ResetPassData>
) -> AppResult<impl IntoResponse> {
    t_reset_password(data, &pool).await?;

    Ok((StatusCode::SEE_OTHER, [(header::LOCATION, "/login.html")]))
}

async fn t_reset_password(
    data: ResetPassData,
    pool: &database::DbPool
) -> AppResult<()> {
    auth_service::check_reset_password_hash(&data.hash, pool).await?;
    match auth_service::hash_password(&data.new_password) {
        Ok(new_hash) => user_dao::update_password(&data.hash, &new_hash, pool).await?,
        Err(err) => return Err(AppError::InternalServerError(err.to_string()))
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = "/refresh-cookie",
    tag = "auth",
    security(("cookie_auth" = [])),
    responses(
        (status = 200, description = "Identity cookie refreshed"),
        (status = 401, description = "Not logged in", body = String),
    )
)]
pub async fn refresh_cookie(
    Identity(user_id): Identity,
    jar: PrivateCookieJar,
) -> AppResult<impl IntoResponse> {
    let jar = identity::login(jar, user_id);

    Ok((jar, StatusCode::OK))
}

#[derive(Deserialize, ToSchema)]
pub struct EmailConfirmationData {
    pub hash: String
}

#[utoipa::path(
    post,
    path = "/email-confirmation",
    tag = "auth",
    request_body = EmailConfirmationData,
    responses(
        (status = 200, description = "Email confirmed"),
        (status = 400, description = "Bad request", body = String),
    )
)]
pub async fn email_confirmation(
    State(pool): State<database::DbPool>,
    Json(data): Json<EmailConfirmationData>
) -> AppResult<StatusCode> {
    auth_service::email_confirmation(&data.hash, &pool).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateEmailConfirmationData {
    pub email: String,
    pub auth: steam::SteamAuthData
}

#[utoipa::path(
    put,
    path = "/email-confirmation",
    tag = "auth",
    request_body = UpdateEmailConfirmationData,
    responses(
        (status = 200, description = "Email updated and confirmation email sent"),
        (status = 400, description = "Bad request", body = String),
    )
)]
pub async fn update_email_confirmation(
    State(pool): State<database::DbPool>,
    Json(data): Json<UpdateEmailConfirmationData>
) -> AppResult<StatusCode> {
    let steam_id = auth_service::steam_authenticate_and_ownership_check(&data.auth).await?;

    auth_service::update_email_confirmation(
        data.email, steam_id, &pool).await?;
    
    Ok(StatusCode::OK)
}