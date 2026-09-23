use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::cookie::PrivateCookieJar;
use serde::Deserialize;
use crate::errors::AppResult;
use crate::database::{self, users as user_dao};
use crate::services::steam::{self, SteamAuthData};
use super::identity::{self, Identity};

#[utoipa::path(
    post,
    path = "/login-steam",
    tag = "auth",
    request_body = SteamAuthData,
    responses(
        (status = 200, description = "Logged in, identity cookie set", body = user_dao::UserDAO),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Steam authentication failed", body = String),
    )
)]
pub async fn login_steam(
    jar: PrivateCookieJar,
    State(pool): State<database::DbPool>,
    Json(auth_data): Json<SteamAuthData>
) -> AppResult<impl IntoResponse> {
    let steam_id = steam::authenticate_and_check_ownership(&auth_data).await?;
    let user = user_dao::get_by_steam_id(&steam_id.to_string(), &pool).await?;
    let jar = identity::login(jar, user.id);

    Ok((jar, Json(user)))
}

#[derive(Debug, Deserialize)]
pub struct DevLoginData {
    pub user_id: i32
}

// Logs in as any user without Steam. Only routed when `--dev-login` is set,
// and deliberately left out of the OpenAPI spec.
pub async fn dev_login(
    jar: PrivateCookieJar,
    State(pool): State<database::DbPool>,
    Json(data): Json<DevLoginData>
) -> AppResult<impl IntoResponse> {
    let user = user_dao::get(data.user_id, &pool).await?;
    let jar = identity::login(jar, user.id);

    Ok((jar, Json(user)))
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
