use axum::extract::ws::{Message, WebSocket};
use super::lobby::Lobby;
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

// Drives a client socket until it closes, fails or misses its heartbeat.
pub async fn run(mut socket: WebSocket, user_id: i32, lobby: Lobby) {
    let (conn_id, mut outgoing) = lobby.connect(user_id);
    let mut hb = Instant::now();
    let mut heartbeat = tokio::time::interval(HEARTBEAT_INTERVAL);

    loop {
        tokio::select! {
            // incoming message from client
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Ping(msg))) => {
                        hb = Instant::now();
                        if socket.send(Message::Pong(msg)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        hb = Instant::now();
                    }
                    Some(Ok(Message::Text(_))) | Some(Ok(Message::Binary(_))) => (), //do nothing
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                }
            }
            // server sent a message to forward to client
            Some(message) = outgoing.recv() => {
                if socket.send(Message::Text(message.into())).await.is_err() {
                    break;
                }
            }
            _ = heartbeat.tick() => {
                if Instant::now().duration_since(hb) > CLIENT_TIMEOUT {
                    println!("Disconnecting failed heartbeat");
                    break;
                }

                if socket.send(Message::Ping("PING".into())).await.is_err() {
                    break;
                }
            }
        }
    }

    lobby.disconnect(user_id, conn_id);
}
