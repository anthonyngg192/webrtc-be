use crate::{
    actors::session::actor::Session,
    models::{
        peer_session::ParticipantId,
        room_session::{RoomCode, RoomSession},
    },
};
use actix::{Addr, Message};
use mediasoup::{
    consumer::ConsumerType,
    prelude::{
        Consumer, ConsumerId, DtlsParameters, IceCandidate, IceParameters, MediaKind,
        PlainTransport, Producer, ProducerId, RtpCapabilities, RtpCapabilitiesFinalized,
        RtpParameters, WebRtcTransport,
    },
    sctp_parameters::SctpParameters,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "()")]
pub struct PeerJoinRoomSession {
    pub session: Session,
    pub room_code: RoomCode,
    pub peer_addr: Addr<Session>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PeerLeaveRoomSession {
    pub participant_id: ParticipantId,
    pub name: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CreateTransport {
    pub participant: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectConsumerTransport {
    pub participant: ParticipantId,
    pub dtls_parameters: DtlsParameters,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ConnectProducerTransport {
    pub participant: ParticipantId,
    pub dtls_parameters: DtlsParameters,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CreateConsumer {
    pub participant: ParticipantId,
    pub producer_id: ProducerId,
    pub rtp_capabilities: RtpCapabilities,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct CreateProducer {
    pub participant: ParticipantId,
    pub kind: MediaKind,
    pub rtp_parameter: RtpParameters,
}

#[derive(Message)]
#[rtype(result = "Option<RoomSession>")]
pub struct GetRoom {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MessageToPeerInRoom {
    pub participant: ParticipantId,
    pub message: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct MessageToPeersInRoom {
    pub message: String,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct GetRoomInfo {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddTransports {
    pub producer_transport: Arc<WebRtcTransport>,
    pub consumer_transport: Arc<WebRtcTransport>,
    pub peer: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewProducer {
    pub producer: Arc<Producer>,
    pub peer: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClearPeerData {
    pub participant_id: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewConsumer {
    pub consumer: Arc<Consumer>,
    pub peer: ParticipantId,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomPauserResumeProducer {
    pub producer_id: ProducerId,
    pub peer: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomCloseProducer {
    pub producer_id: ProducerId,
    pub peer: ParticipantId,
}
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewRoomMessage {
    pub text: Option<String>,
    pub gif: Option<String>,
    pub participant_id: Option<ParticipantId>,
    pub r#type: MessageType,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BannedUserOutRoom {
    pub user_code: String,
    pub participant_id: ParticipantId,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct PrometheusPulling {}

#[derive(Message)]
#[rtype(result = "()")]
pub struct AddPlainTransport {
    pub plain_trans: Arc<PlainTransport>,
}

/*
    Data respond for client.
*/

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConsumerResult {
    pub producer_id: ProducerId,
    pub id: ConsumerId,
    pub kind: MediaKind,
    pub rtp_parameters: RtpParameters,
    pub r#type: ConsumerType,
    pub producer_paused: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConsumerRespond {
    pub event_name: String,
    pub data: CreateConsumerResult,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoomRespond {
    pub rtp_capabilities: RtpCapabilitiesFinalized,
    pub event_name: String,
    pub peer_id: String,
    pub user_code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransportRespond {
    pub event_name: String,
    pub producer: ProducerTransport,
    pub consumer: ConsumerTransport,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerTransport {
    pub id: String,
    pub ice_parameters: IceParameters,
    pub ice_candidates: Vec<IceCandidate>,
    pub dtls_parameters: DtlsParameters,
    pub sctp_parameters: Option<SctpParameters>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsumerTransport {
    pub id: String,
    pub ice_parameters: IceParameters,
    pub ice_candidates: Vec<IceCandidate>,
    pub dtls_parameters: DtlsParameters,
    pub sctp_parameters: Option<SctpParameters>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProducerDetail {
    pub kind: MediaKind,
    pub producer_id: String,
    pub broadcaster_id: String,
    pub broadcaster_code: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub id: String,
    pub user_code: String,
    pub name: String,
    pub produces: Vec<ProducerDetail>,
    pub joined: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPeerInfoRespond {
    pub event_name: String,
    pub peer_info: Vec<PeerInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRoomMessageRespond {
    pub event_name: String,
    pub data: NewRoomMessageDataRespond,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomBanRespond {
    pub event_name: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRoomMessageDataRespond {
    pub text: Option<String>,
    pub gif: Option<String>,
    pub owner_code: Option<String>,
    pub r#type: MessageType,
}

#[derive(Clone, Serialize)]
pub enum MessageType {
    Message,
    Notification,
}
