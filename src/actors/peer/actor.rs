use crate::services::peer_service::PeerService;
use actix::{Actor, Context, Supervised, SystemService};

impl Supervised for PeerService {}
impl SystemService for PeerService {}

impl Actor for PeerService {
    type Context = Context<Self>;
}
