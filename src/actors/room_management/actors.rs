use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use actix::{Actor, Addr, Context, Supervised, SystemService};
use mediasoup::{prelude::WebRtcServer, worker::Worker};

use crate::{
    models::room_session::{RoomCode, RoomSession},
    services::peer_service::PeerService,
};

pub struct RoomManager {
    pub rooms: HashMap<RoomCode, Arc<Addr<RoomSession>>>,
    pub peer_service: Option<Arc<PeerService>>,
    pub workers: HashMap<usize, Worker>,
    pub webrtc_servers: HashMap<usize, Arc<WebRtcServer>>,
    pub current_index: Arc<RwLock<usize>>,
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            peer_service: None,
            current_index: Arc::new(RwLock::new(1)),
            workers: HashMap::new(),
            webrtc_servers: HashMap::new(),
        }
    }
}

impl Actor for RoomManager {
    type Context = Context<Self>;
}

impl Supervised for RoomManager {}
impl SystemService for RoomManager {}
