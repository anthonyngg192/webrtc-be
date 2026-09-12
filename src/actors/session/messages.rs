use actix::{Addr, Message};
use mediasoup::prelude::{DtlsParameters, MediaKind, ProducerId, RtpCapabilities, RtpParameters};
use serde::{Deserialize, Serialize};

use crate::actors::{chat::messages::NewMessageEvent, room_session::actor::RoomActor};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomRequest {
    pub room_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConsumerRequest {
    pub producer_id: ProducerId,
    pub rtp_capabilities: RtpCapabilities,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateProducerRequest {
    pub kind: MediaKind,
    pub rtp_parameters: RtpParameters,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResumePauseCloseProducer {
    pub producer_id: ProducerId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewRoomMessage {
    pub text: Option<String>,
    pub gif: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BannedUserOutRoom {
    pub user_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PeerLeaveRoom {}

#[derive(Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum ClientEvent {
    NewMessage { data: NewMessageEvent },
    JoinRoomRequest { data: JoinRoomRequest },
    CreateTransportRequest,
    ConnectConsumerTransportRequest { data: DtlsParameters },
    ConnectProducerTransportRequest { data: DtlsParameters },
    NewConsumerRequest { data: CreateConsumerRequest },
    NewProducerRequest { data: CreateProducerRequest },
    PauseProducer { data: ResumePauseCloseProducer },
    ResumeProducer { data: ResumePauseCloseProducer },
    CloseProducer { data: ResumePauseCloseProducer },
    NewRoomMessage { data: NewRoomMessage },
    BannedUserOutRoom { data: BannedUserOutRoom },
    PeerLeaveRoom,
    Ping,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub content: Vec<u8>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AssignRoom {
    pub addr: Addr<RoomActor>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionDisconnect {}
