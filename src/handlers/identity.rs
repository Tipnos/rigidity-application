use axum::{extract::{FromRef, FromRequestParts}, http::request::Parts};
use axum_extra::extract::cookie::{Cookie, Key, PrivateCookieJar, SameSite};
use uuid::Uuid;
use crate::errors::AppError;

const IDENTITY_COOKIE: &str = "id";

// Public UUID of the logged in user, read from the private identity cookie.
// Rejects the request with 401 when the user isn't logged in.
pub struct Identity(pub Uuid);

impl<S> FromRequestParts<S> for Identity
where
    S: Send + Sync,
    Key: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = PrivateCookieJar::<Key>::from_request_parts(parts, state).await
            .map_err(|_| AppError::Unauthorized)?;

        jar.get(IDENTITY_COOKIE)
            .and_then(|cookie| cookie.value().parse::<Uuid>().ok())
            .map(Identity)
            .ok_or(AppError::Unauthorized)
    }
}

pub fn login(jar: PrivateCookieJar, user_id: Uuid) -> PrivateCookieJar {
    let cookie = Cookie::build((IDENTITY_COOKIE, user_id.to_string()))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax);

    jar.add(cookie)
}

pub fn logout(jar: PrivateCookieJar) -> PrivateCookieJar {
    jar.remove(Cookie::build(IDENTITY_COOKIE).path("/"))
}
