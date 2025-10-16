use std::sync::Arc;

use crate::{
    actors::session::actor::Session,
    models::{
        peer_session::{ParticipantId, PeerSession},
        room_session::{RoomCode, RoomSession},
    },
};
use actix::{Addr, Message};

#[derive(Message)]
#[rtype(result = "Vec<PeerSession>")]
pub struct GetPeers {
    pub participant_ids: Vec<ParticipantId>,
}

#[derive(Message)]
#[rtype(result = "Option<PeerSession>")]
pub struct GetPeerByUserCode {
    pub user_code: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PeerJoinRoom {
    pub room_code: String,
    pub session: Session,
    pub addr: Addr<Session>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinRoom {
    pub room_code: RoomCode,
    pub participant_id: ParticipantId,
    pub room_addr: Arc<Addr<RoomSession>>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PeerLeaveRoom {
    pub participant_id: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPeer {
    pub data: Vec<u8>,
    pub participant_id: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPeers {
    pub data: Vec<u8>,
    pub participant_ids: Vec<ParticipantId>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPeerByUser {
    pub user_code: String,
    pub data: Vec<u8>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendPeersByUser {
    pub user_codes: Vec<String>,
    pub data: Vec<u8>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ErrorEvent {
    pub data: String,
    pub participant_id: ParticipantId,
    pub event_name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ErrorEventByUser {
    pub data: String,
    pub user_code: String,
    pub event_name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewPeer {
    pub participant_id: ParticipantId,
    pub name: String,
    pub addr: Addr<Session>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PeerDisconnect {
    pub participant_id: ParticipantId,
}
