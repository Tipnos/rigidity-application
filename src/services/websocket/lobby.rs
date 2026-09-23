use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;
use super::{ForwardMessage, MultiForwardMessage, BroadcastExceptMessage};
use crate::database::DbPool;
use crate::services::custom_room::handle_websocket_closing as on_custom_room_disconnect;

struct Session {
    conn_id: u64,
    sender: UnboundedSender<String>,
}

#[derive(Clone)]
pub struct Lobby {
    sessions: Arc<Mutex<HashMap<Uuid, Session>>>, //user_id to socket
    next_conn_id: Arc<AtomicU64>,
    pool: DbPool
}

impl Lobby {
    pub fn new(pool: DbPool) -> Self {
        Lobby {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            next_conn_id: Arc::new(AtomicU64::new(0)),
            pool
        }
    }

    // Registers a socket for the user, replacing any previous one.
    // Returns the connection id and the receiver of messages to push to the client.
    pub fn connect(&self, user_id: Uuid) -> (u64, UnboundedReceiver<String>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        let conn_id = self.next_conn_id.fetch_add(1, Ordering::Relaxed);
        self.sessions.lock().unwrap().insert(user_id, Session { conn_id, sender });

        (conn_id, receiver)
    }

    // Only removes the session if it's still owned by this connection,
    // so a stale socket can't evict a newer one of the same user.
    pub fn disconnect(&self, user_id: Uuid, conn_id: u64) {
        {
            let mut sessions = self.sessions.lock().unwrap();
            match sessions.get(&user_id) {
                Some(session) if session.conn_id == conn_id => {
                    sessions.remove(&user_id);
                },
                _ => return,
            }
        }

        let lobby = self.clone();
        tokio::spawn(async move {
            on_custom_room_disconnect(&user_id, lobby.clone(), &lobby.pool).await;
        });
    }

    pub fn forward(&self, msg: ForwardMessage) {
        self.send_message(msg.get_message(), msg.get_id());
    }

    pub fn multi_forward(&self, msg: MultiForwardMessage) {
        self.send_many_message(msg.get_message(), msg.get_ids());
    }

    pub fn broadcast_except(&self, msg: BroadcastExceptMessage) {
        self.send_message_to_all_except(msg.get_message(), msg.get_ids_to_except());
    }

    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(session) = self.sessions.lock().unwrap().get(id_to) {
            let _ = session.sender.send(message.to_owned());
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    fn send_message_to_all_except(&self, message: &str, ids_to_except: &Vec<Uuid>) {
        for (id, session) in self.sessions.lock().unwrap().iter() {
            if !ids_to_except.iter().any(|except| except == id) {
                let _ = session.sender.send(message.to_owned());
            }
        }
    }

    fn send_many_message(&self, message: &str, ids: &Vec<Uuid>) {
        for id in ids {
            self.send_message(message, id);
        }
    }
}
