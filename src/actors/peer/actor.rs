use crate::services::peer_service::PeerService;
use actix::{Actor, Context, Supervised, SystemService};

pub struct PeerActor {
    pub service: PeerService,
}

impl Default for PeerActor {
    fn default() -> Self {
        Self {
            service: PeerService::new(),
        }
    }
}

impl Supervised for PeerActor {}
impl SystemService for PeerActor {}

impl Actor for PeerActor {
    type Context = Context<Self>;
}
