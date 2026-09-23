use axum::{body::Bytes, extract::State, http::{HeaderMap, StatusCode}};
use crate::errors::{AppError, AppResult};
use serde_json::{from_slice};
use serde::Deserialize;
use crate::services::aws::*;
use crate::services::custom_room;
use crate::database;
use crate::services::{as_json_string, websocket::WebsocketLobby};

pub async fn sns(
    headers: HeaderMap,
    State(ws): State<WebsocketLobby>,
    State(pool): State<database::DbPool>,
    body: Bytes
) -> AppResult<StatusCode> {
    let error = Err(AppError::BadRequest(String::from("x-amz-sns-message-type header is unknown or missing.")));

    if let Some(message_type_header)  = headers.get("x-amz-sns-message-type") {
        if let Ok(message_type) = message_type_header.to_str() {
            match message_type {
                "SubscriptionConfirmation" => {
                    return handle_sns_subscription(body).await
                },
                "Notification" => {
                    return handle_sns_notification(body,ws,pool).await
                },
                _ => {
                    return error
                }
            }
        }
    }

    error
}

async fn handle_sns_subscription(
    body: Bytes
) -> AppResult<StatusCode> {
    #[derive(Deserialize)]
    struct SnsData {
        #[serde(rename = "SubscribeURL")]
        pub subscribe_url: String
    }
    if let Ok(obj) = from_slice::<SnsData>(&body) {
        let obj: SnsData = obj;

        match reqwest::get(obj.subscribe_url).await {
            Ok(_) => return Ok(StatusCode::OK),
            Err(err) => {
                return Err(AppError::BadRequest(err.to_string()))
            }
        }
    }

    Err(AppError::BadRequest(String::from("Json body has wrong format.")))    
}

async fn handle_sns_notification(
    body: Bytes,
    ws: WebsocketLobby,
    pool: database::DbPool
) -> AppResult<StatusCode> {
    #[derive(Deserialize)]
    struct SnsData {
        #[serde(rename = "Message", with = "as_json_string")]
        pub message: FlexMatchData<FlexMatchDetail>,
    }

    if let Ok(obj) = from_slice::<SnsData>(&body) {
        match obj.message.detail.e_type {
            FlexMatchEvents::MatchmakingSucceeded => {
                #[derive(Deserialize)]
                struct SnsDataSucceeded {
                    #[serde(rename = "Message", with = "as_json_string")]
                    pub message: FlexMatchData<FlexMatchSucceededDetail>,
                }
                let data = from_slice::<SnsDataSucceeded>(&body).unwrap();
                if let Err(err) = custom_room::matchmaking_succeeded(
                    data.message,
                    ws.clone(),
                    &pool
                ).await {
                    return Err(err)
                }
            },
            FlexMatchEvents::MatchmakingTimedOut |
            FlexMatchEvents::MatchmakingCancelled |
            FlexMatchEvents::MatchmakingFailed => {
                let ticket_id = &obj.message.detail.tickets[0].ticket_id;
                if let Err(err) = custom_room::matchmaking_failed(
                    obj.message.detail.e_type,
                    ticket_id,
                    ws.clone(),
                    &pool
                ).await {
                    return Err(err)
                }
            }
           _ => {
                
           }
        }
        return Ok(StatusCode::OK)
    }

    Err(AppError::BadRequest(String::from("Json body has wrong format.")))   
}