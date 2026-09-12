use std::collections::HashMap;

use super::messages::*;
use crate::{actors::room_session::actor::RoomActor, models::room_session::RoomCode};
use actix::{Actor, Addr, Context, Supervised, SystemService};
use mediasoup::{prelude::WebRtcServer, worker::Worker};
use std::sync::Arc;

pub struct RoomsActor {
    pub rooms: HashMap<RoomCode, Arc<Addr<RoomActor>>>,
    pub workers: HashMap<usize, Worker>,
    pub webrtc_servers: HashMap<usize, Arc<WebRtcServer>>,
    pub current_index: usize,
}

impl Default for RoomsActor {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomsActor {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            current_index: 1,
            workers: HashMap::new(),
            webrtc_servers: HashMap::new(),
        }
    }

    pub fn add_worker(&mut self, msg: AddWorker) {
        self.workers.insert(msg.id, msg.worker);
        self.webrtc_servers
            .insert(msg.id, Arc::new(msg.webrtc_server));
    }

    pub fn get_worker(&mut self) -> Option<(Worker, Arc<WebRtcServer>)> {
        if self.workers.is_empty() || self.webrtc_servers.is_empty() {
            return None;
        }
        let mut index = self.current_index;
        let worker = self.workers.get(&index).cloned().or_else(|| {
            index = 1;
            self.workers.get(&1).cloned()
        });
        let webrtc_server = self.webrtc_servers.get(&index).cloned().or_else(|| {
            index = 1;
            self.webrtc_servers.get(&1).cloned()
        });
        self.current_index = (index + 1) % self.workers.len();
        match (worker, webrtc_server) {
            (Some(worker), Some(webrtc_server)) => Some((worker, webrtc_server)),
            _ => None,
        }
    }
}

impl Actor for RoomsActor {
    type Context = Context<Self>;
}

impl Supervised for RoomsActor {}
impl SystemService for RoomsActor {}
