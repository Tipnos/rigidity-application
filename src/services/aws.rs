use rusoto_gamelift::GameLiftClient;
use rusoto_core::credential::StaticProvider;
use rusoto_core::request::HttpClient;
use rusoto_core::region::Region;
use serde::Deserialize;
use crate::app_conf::config;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub async fn get_gamelift_client() -> GameLiftClient {
    let config = config();
    let cred = StaticProvider::new_minimal(
        config.aws_access_key_id.clone(),
        config.aws_secret_access_key.clone());
    let client = HttpClient::new().unwrap();
    GameLiftClient::new_with(client, cred, Region::EuWest1)
}

#[derive(Deserialize, Debug)]
pub enum FlexMatchEvents {
    MatchmakingSearching,
    PotentialMatchCreated,
    AcceptMatch,
    AcceptMatchCompleted,
    MatchmakingSucceeded,
    MatchmakingTimedOut,
    MatchmakingCancelled,
    MatchmakingFailed,
}

impl Display for FlexMatchEvents {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

#[derive(Deserialize, Debug)]
pub struct FlexMatchData<T> 
{
    pub id: String,
    pub account: String,
    pub region: String,
    pub resources: Vec<String>,
    pub detail: T
}

#[derive(Deserialize, Debug)]
pub struct FlexMatchDetail {
    pub tickets: Vec<FlexMatchTicket>,
    #[serde(rename = "type")]
    pub e_type: FlexMatchEvents
}

#[derive(Deserialize, Debug)]
pub struct FlexMatchPotentialDetail {
    pub tickets: Vec<FlexMatchTicket>,
    #[serde(rename = "type")]
    pub e_type: FlexMatchEvents,
    #[serde(rename = "matchId")]
    pub match_id: String
}

#[derive(Deserialize, Debug)]
pub struct FlexMatchSucceededDetail {
    pub tickets: Vec<FlexMatchTicket>,
    #[serde(rename = "type")]
    pub e_type: FlexMatchEvents,
    #[serde(rename = "matchId")]
    pub match_id: String,
    #[serde(rename = "gameSessionInfo")]
    pub game_session_info: FlexMatchGameSession
}

#[derive(Deserialize, Debug)]
pub struct FlexMatchGameSession {
    #[serde(rename = "ipAddress")]
    pub ip_address: String,
    pub port: i32,
    pub players: Vec<FlexMatchPlayerGameSession>
}

#[derive(Deserialize, Debug)]
pub struct FlexMatchPlayerGameSession {
    #[serde(rename = "playerId")]
    pub player_id: String,
    #[serde(rename = "playerSessionId")]
    pub player_session_id: String
}

#[derive(Deserialize, Debug)]
pub struct FlexMatchTicket {
    #[serde(rename = "ticketId")]
    pub ticket_id: String
}