use crate::{
    models::room_session::{RoomCode, RoomSession},
    services::room_service::RoomService,
};
use actix::{Addr, Message};
use mediasoup::{prelude::WebRtcServer, worker::Worker};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddWorker {
    pub id: usize,
    pub worker: Worker,
    pub webrtc_server: WebRtcServer,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomManagementCreateRoom {
    pub room_code: RoomCode,
    pub owner_code: String,
    pub room_service: Arc<RoomService>,
}

#[derive(Message)]
#[rtype(result = "Option<(Worker, Arc<WebRtcServer>)>")]
pub struct GetWorker {}

#[derive(Message)]
#[rtype(result = "Option<Arc<Addr<RoomSession>>>")]
pub struct GetRoomAddr {
    pub room_code: RoomCode,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewRoom {
    pub room_code: RoomCode,
    pub addr: Addr<RoomSession>,
}
