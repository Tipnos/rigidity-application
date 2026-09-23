use reqwest::{header, Client};
use crate::services::make_path_and_query;
use std::collections::HashMap;
use crate::app_conf::config;
use crate::errors::{AppResult, AppError};
use serde::Deserialize;
use serde_json;

const STEAM_DOMAIN: &str = "partner.steam-api.com";
const UNIVERSAL_STEAM_APP_ID: u64 = 480;

#[derive(Deserialize)]
pub struct SteamAuthData {
    pub app_id: u64,
    pub auth_ticket: String
}

#[derive(Deserialize)]
struct AuthResponseBase<T> {
    pub response: T
}

#[derive(Deserialize)]
struct AuthResponse<T> {
    pub params: T
}

#[derive(Deserialize)]
struct AuthenticateUserTicketResponse {
    pub result: String,
    #[serde(rename = "steamid")]
    pub steam_id: String,
    // #[serde(rename = "ownersteamid")]
    // pub owner_steam_id: String,
    #[serde(rename = "vacbanned")]
    pub vac_banned: bool,
    #[serde(rename = "publisherbanned")] 
    pub publisher_banned: bool,
}

#[derive(Deserialize)]
struct ErrorResponse {
    // #[serde(rename = "errorcode")]
    // error_code: i32,
    // #[serde(rename = "errordesc")]
    // message: String,
}

#[derive(Deserialize)]
struct Error {
    // error: ErrorResponse
}

pub async fn authenticate_user_ticket(data: &SteamAuthData) -> AppResult<u64> {
    let mut params = HashMap::new();
    params.insert("key", config().steam_secret_access_key.clone());
    params.insert("appid", data.app_id.to_string());
    params.insert("ticket", data.auth_ticket.to_string());

    let uri = format!("https://{}{}", STEAM_DOMAIN, make_path_and_query(
        "/ISteamUserAuth/AuthenticateUserTicket/v1", &params));
    
    let client = Client::new();
    let result = client.get(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .send()
        .await?;

    let body = result.bytes().await?;
    match serde_json::from_slice::<AuthResponseBase<AuthResponse<AuthenticateUserTicketResponse>>>(&body) {
        Ok(steam_response) => {
            if steam_response.response.params.result == "OK" && !steam_response.response.params.vac_banned &&
            ! steam_response.response.params.publisher_banned {
                Ok(steam_response.response.params.steam_id.parse::<u64>().unwrap())
            } else {
                Err(AppError::Unauthorized)
            }
        },
        Err(_) => {
            // let _steam_error = serde_json::from_slice::<AuthResponseBase<Error>>(&body)?;
            Err(AppError::Unauthorized)
        }
    }
}

#[derive(Deserialize)]
struct OwnershipBaseResponse<T> {
    #[serde(rename = "appownership")]
    pub app_ownership: T
}

#[derive(Deserialize)]
struct OwnershipResponse {
    #[serde(rename = "ownsapp")]
    pub owns_app: bool,
    // pub permanent: bool,
    // pub timestamp: DateTime<Utc>,
    // #[serde(rename = "ownersteamid")]
    // pub owner_steam_id: String,
    // #[serde(rename = "sitelicense")] 
    // pub site_license: bool,
    // #[serde(rename = "timedtrial")] 
    // pub timed_trial: bool,
    pub result: String
}

pub async fn check_app_ownership(app_id: &u64, steam_id: &u64) -> AppResult<()> {
    if app_id == &UNIVERSAL_STEAM_APP_ID {
        Ok(())
    } else {
        let mut params = HashMap::new();
        params.insert("key", config().steam_secret_access_key.clone());
        params.insert("appid", app_id.to_string());
        params.insert("steamid", steam_id.to_string());
    
        let uri = format!("https://{}{}", STEAM_DOMAIN,
            make_path_and_query("/ISteamUser/CheckAppOwnership/v2", &params));
        
        let client = Client::new();
        let response = client.get(uri)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .send()
            .await?;
            
        let body = response.bytes().await?;
        match serde_json::from_slice::<OwnershipBaseResponse<OwnershipResponse>>(&body) {
            Ok(steam_response) => {
                if steam_response.app_ownership.result == "OK" && steam_response.app_ownership.owns_app {
                    Ok(())
                } else {
                    Err(AppError::Unauthorized)
                }
            },
            Err(_) => {
                Err(AppError::Unauthorized)
            }
        }
    }
    
}