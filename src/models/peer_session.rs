use std::sync::Arc;

use actix::Addr;
use mediasoup::prelude::RtpCapabilities;

use crate::actors::{room_session::actor::RoomActor, session::actor::Session};

use super::room_session::RoomCode;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum ParticipantType {
    User,
    Anonymous,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ParticipantId {
    pub user_code: String,
    pub peer_id: String,
    pub r#type: ParticipantType,
}

#[derive(Clone)]
pub struct PeerSession {
    pub participant: ParticipantId,
    pub room_code: Option<RoomCode>,
    pub rtp_capabilities: Option<RtpCapabilities>,
    pub name: String,
    pub addr: Arc<Addr<Session>>,
    pub room_addr: Option<Arc<Addr<RoomActor>>>,
}
