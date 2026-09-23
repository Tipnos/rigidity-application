use actix_web::{web, HttpRequest, web::Payload, HttpResponse};
use awc::Client;
use futures_util::StreamExt;
use crate::errors::{AppError, AppResult};
use serde_json::{from_slice};
use serde::Deserialize;
use crate::services::aws::*;
use crate::services::custom_room;
use actix::{Addr};
use crate::database;
use crate::services::{as_json_string, websocket::WebsocketLobby};

pub async fn sns(
    req: HttpRequest,
    mut stream: Payload,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
    let error = Err(AppError::BadRequest(String::from("x-amz-sns-message-type header is unknown or missing.")));

    if let Some(message_type_header)  = req
        .headers()
        .get(String::from("x-amz-sns-message-type")) {
        if let Ok(message_type) = message_type_header.to_str() {
            let mut body = web::BytesMut::new();
            while let Some(item) = stream.next().await {
                match item {
                    Ok(chunk) => body.extend_from_slice(&chunk),
                    Err(_) => return Err(AppError::BadRequest(String::from("Corrupted body.")))
                }
            }
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
    body: web::BytesMut
) -> AppResult<HttpResponse> {
    #[derive(Deserialize)]
    struct SnsData {
        #[serde(rename = "SubscribeURL")]
        pub subscribe_url: String
    }
    if let Ok(obj) = from_slice::<SnsData>(&body) {
        let obj: SnsData = obj;

        let client = Client::default();
        match client.get(obj.subscribe_url).send().await {
            Ok(_) => return Ok(HttpResponse::Ok().finish()),
            Err(err) => {
                return Err(AppError::BadRequest(err.to_string()))
            }
        }
    }

    Err(AppError::BadRequest(String::from("Json body has wrong format.")))    
}

async fn handle_sns_notification(
    body: web::BytesMut,
    ws: web::Data<Addr<WebsocketLobby>>,
    pool: web::Data<database::DbPool>
) -> AppResult<HttpResponse> {
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
                    ws.get_ref().to_owned(),
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
                    ws.get_ref().to_owned(),
                    &pool
                ).await {
                    return Err(err)
                }
            }
           _ => {
                
           }
        }
        return Ok(HttpResponse::Ok().finish())
    }

    Err(AppError::BadRequest(String::from("Json body has wrong format.")))   
}