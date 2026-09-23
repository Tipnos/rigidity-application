use actix_web::{error::{ResponseError, PayloadError}, HttpResponse};
use derive_more::Display;
use std::convert::From;
use serde_json;
use serde::{Serialize};
use awc::error::{SendRequestError, HttpError};

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

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::ServiceUnavailable(ref message) => HttpResponse::ServiceUnavailable()
                .json(message),
            AppError::InternalServerError(ref trace) => {
                HttpResponse::InternalServerError()
                .json(trace)
            }
            AppError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(message)
            }
            AppError::Unauthorized => {
                HttpResponse::Unauthorized().json("Unauthorized")
            }
            AppError::Forbidden => {
                HttpResponse::Forbidden().json("Forbidden")
            }
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> AppError {
        AppError::InternalServerError(format!("Error while parsing json. {}", error.to_string()))
    }
}

impl From<PayloadError> for AppError {
    fn from(error: PayloadError) -> AppError {
        AppError::InternalServerError(format!("Payload error. {}", error.to_string()))
    }
}

impl From<SendRequestError> for AppError {
    fn from(error: SendRequestError) -> AppError {
        AppError::BadRequest(format!("A request send by the server has failed. {}", error.to_string()))
    }
}

impl From<HttpError> for AppError {
    fn from(error: HttpError) -> AppError {
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