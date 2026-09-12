use super::peer_session::ParticipantId;
use crate::{actors::session::actor::Session, services::room_service::RoomService};
use actix::Addr;
use mediasoup::{
    prelude::{
        Consumer, ConsumerId, MediaKind, PlainTransport, Producer, ProducerId, WebRtcServer,
        WebRtcTransport,
    },
    router::Router,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct RoomCode(pub String);

#[derive(Clone)]
pub struct RoomSession {
    pub code: RoomCode,
    pub router: Router,
    pub webrtc_server: Arc<WebRtcServer>,
    pub peers: HashMap<ParticipantId, Arc<Addr<Session>>>,
    pub consumers: HashMap<ParticipantId, HashSet<ConsumerId>>,
    pub producers: HashMap<ParticipantId, HashSet<ProducerId>>,
    pub consumer_transports: HashMap<ParticipantId, Arc<WebRtcTransport>>,
    pub producer_transports: HashMap<ParticipantId, Arc<WebRtcTransport>>,
    pub peer_consumers: HashMap<ConsumerId, Arc<Consumer>>,
    pub peer_producers: HashMap<ProducerId, Arc<Producer>>,
    pub muted_peers: HashMap<ParticipantId, HashSet<MutedPeer>>,
    pub owner_code: String,
    pub service: Arc<RoomService>,
    pub plain_transport: Option<Arc<PlainTransport>>,
}

#[derive(Clone)]
pub struct MutedPeer {
    pub participant_id: ParticipantId,
    pub kids: MediaKind,
}
