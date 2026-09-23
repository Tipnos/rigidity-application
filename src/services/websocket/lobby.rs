use super::messages::{Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler};
use std::collections::HashMap;
use actix::Addr;
use super::{ws::WsConn, ForwardMessage, MultiForwardMessage, BroadcastExceptMessage};
use crate::database::DbPool;
use crate::services::custom_room::handle_websocket_closing as on_custom_room_disconnect;

pub struct Lobby {
    pub sessions: HashMap<i32, Addr<WsConn>>, //user_id to socket
    pub pool: DbPool
}

impl Lobby {
    pub fn new(pool: DbPool) -> Self {
        Lobby {
            sessions: HashMap::new(),
            pool
        }
    }
}

impl Lobby {
    pub fn send_message(&self, message: &str, id_to: &i32) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient
                .do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }

    pub fn send_message_to_all_except(&self, message: &str, ids_to_except: &Vec<i32>) {
        for s in &self.sessions {
            if !ids_to_except.iter().any(|id| id == s.0) {
                let _ = s.1.do_send(WsMessage(message.to_owned()));
            }
        }
    }

    pub fn send_many_message(&self, message: &str, ids: &Vec<i32>) {
        for id in ids {
            self.send_message(message, id);
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);

        let pool = self.pool.clone();
        actix::spawn(async move {
            on_custom_room_disconnect(&msg.id, msg.addr, &pool).await;
        });
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.insert(
            msg.self_id,
            msg.addr,
        );
    }
}

impl Handler<ForwardMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ForwardMessage, _: &mut Context<Self>) -> Self::Result {
        self.send_message(msg.get_message(), msg.get_id());
    }
}

impl Handler<MultiForwardMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: MultiForwardMessage, _: &mut Context<Self>) -> Self::Result {
        self.send_many_message(msg.get_message(), msg.get_ids());
    }
}

impl Handler<BroadcastExceptMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: BroadcastExceptMessage, _: &mut Context<Self>) -> Self::Result {
        self.send_message_to_all_except(msg.get_message(), msg.get_ids_to_except());
    }
}