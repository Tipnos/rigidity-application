use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use derive_more::Display;
use std::convert::From;
use serde_json;
use serde::{Serialize};

pub type AppResult<R> = Result<R, AppError>;

#[derive(Debug, Display, Serialize)]
pub enum AppError {
    #[display(fmt = "Service Unavailable")]
    ServiceUnavailable(String),

    #[display(fmt = "Internal Server Error")]
    InternalServerError(String),
    
    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Forbidden")]
    Forbidden,
}

// impl IntoResponse allows to convert our errors into http responses with appropriate data
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::ServiceUnavailable(message) => (StatusCode::SERVICE_UNAVAILABLE,
                Json(message)).into_response(),
            AppError::InternalServerError(trace) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(trace)).into_response()
            }
            AppError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, Json(message)).into_response()
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, Json("Unauthorized")).into_response()
            }
            AppError::Forbidden => {
                (StatusCode::FORBIDDEN, Json("Forbidden")).into_response()
            }
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> AppError {
        AppError::InternalServerError(format!("Error while parsing json. {}", error.to_string()))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> AppError {
        AppError::BadRequest(format!("A request send by the server has failed. {}", error.to_string()))
    }
}

impl From<axum::http::Error> for AppError {
    fn from(error: axum::http::Error) -> AppError {
        AppError::InternalServerError(format!("A request build by the server has failed. {}", error.to_string()))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> AppError {
        match error {
            sqlx::Error::RowNotFound => {
                AppError::BadRequest(format!("Database error not found."))
            },
            sqlx::Error::Database(db_err) => {
                AppError::BadRequest(db_err.message().to_string())
            },
            _ => AppError::InternalServerError(String::from("Database error")),
        }
    }
}