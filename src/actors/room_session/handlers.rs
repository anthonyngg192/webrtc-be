use super::messages::*;
use crate::{
    actors::{
        peer::{
            actor::PeerActor,
            messages::{GetPeerByUserCode, GetPeers, SendPeer},
        },
        room_session::actor::RoomActor,
        session::messages::{AssignRoom, SendMessage},
    },
    models::peer_session::ParticipantId,
    repositories::AbstractRoom,
    services::room_session_service::ActionResult,
};
use actix::{AsyncContext, Handler, SystemService};
use mediasoup::prelude::{Producer, ProducerId};
use std::sync::Arc;

impl Handler<PeerJoinRoomSession> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinRoomSession, ctx: &mut Self::Context) -> Self::Result {
        let session = msg.session.clone();
        let peer = session.participant_id.clone();
        self.room
            .peers
            .insert(peer.clone(), Arc::new(msg.peer_addr.clone()));
        let rtp_capabilities = self.room.router.rtp_capabilities();
        PeerActor::from_registry().do_send(SendPeer {
            data: serde_json::to_vec(&JoinRoomRespond {
                rtp_capabilities: rtp_capabilities.clone(),
                event_name: "JoinRoomRespond".to_string(),
                peer_id: peer.peer_id.clone(),
                user_code: peer.user_code.clone(),
            })
            .unwrap(),
            participant_id: peer.clone(),
        });

        msg.peer_addr.do_send(AssignRoom {
            addr: ctx.address(),
        });

        ctx.address().do_send(NewRoomMessage {
            text: Some(format!("{} joined room", msg.session.name)),
            gif: None,
            r#type: super::messages::MessageType::Notification,
            participant_id: None,
        });

        let user_code = peer.user_code.clone();
        let service = self.room.service.clone();
        let room_code = self.room.code.clone();
        actix::spawn(async move {
            service.db.peer_join_room(room_code.0, user_code).await;
        });
    }
}

impl Handler<PeerLeaveRoomSession> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: PeerLeaveRoomSession, ctx: &mut Self::Context) -> Self::Result {
        self.room.peer_leave_room(msg.participant_id.clone());
        ctx.address().do_send(NewRoomMessage {
            text: Some(format!("{} leaved room", msg.name)),
            gif: None,
            r#type: super::messages::MessageType::Notification,
            participant_id: None,
        });
        ctx.address().do_send(GetRoomInfo {});
        let service = self.room.service.clone();
        let user_code = msg.participant_id.user_code.clone();
        let room_code = self.room.code.clone();
        actix::spawn(async move {
            service.db.peer_leave_room(room_code.0, user_code).await;
        });
    }
}

impl Handler<CreateTransport> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: CreateTransport, ctx: &mut Self::Context) -> Self::Result {
        let participant = msg.participant.clone();
        let mut room = self.room.clone();
        let addr = ctx.address();
        actix::spawn(async move {
            let _ = room.new_transport(participant.clone(), addr.clone()).await;
            let _ = room.create_plain_transport(addr.clone()).await;
        });
    }
}

impl Handler<ConnectConsumerTransport> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: ConnectConsumerTransport, _: &mut Self::Context) -> Self::Result {
        let room = self.room.clone();
        let participant = msg.participant.clone();
        let dtls_parameters = msg.dtls_parameters.clone();
        actix::spawn(async move {
            room.connect_consume_transport(participant, dtls_parameters)
                .await;
        });
    }
}

impl Handler<ConnectProducerTransport> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: ConnectProducerTransport, _: &mut Self::Context) -> Self::Result {
        let room = self.room.clone();
        let participant = msg.participant.clone();
        let dtls_parameters = msg.dtls_parameters.clone();
        actix::spawn(async move {
            room.connect_produce_transport(participant, dtls_parameters)
                .await;
        });
    }
}

impl Handler<CreateProducer> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: CreateProducer, ctx: &mut Self::Context) -> Self::Result {
        let mut room = self.room.clone();
        let participant = msg.participant.clone();
        let kind = msg.kind;
        let rtp_parameter = msg.rtp_parameter.clone();
        let addr = ctx.address().clone();
        actix::spawn(async move {
            let producer = room
                .new_producer(participant.clone(), kind, rtp_parameter)
                .await;
            match producer {
                Some(producer) => {
                    let producer = Arc::new(producer);
                    addr.do_send(NewProducer {
                        producer: Arc::clone(&producer),
                        peer: participant,
                    });
                }
                None => {
                    log::error!("can not create Producer");
                }
            };
        });
    }
}

impl Handler<CreateConsumer> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: CreateConsumer, ctx: &mut Self::Context) -> Self::Result {
        let mut room = self.room.clone();
        let producer_id = msg.producer_id;
        let participant = msg.participant.clone();
        let client_rtp_capabilities = msg.rtp_capabilities.clone();
        let addr = ctx.address().clone();
        actix::spawn(async move {
            let consumer = room
                .new_consumer(producer_id, participant.clone(), client_rtp_capabilities)
                .await;
            match consumer {
                Some(consumer) => {
                    let consumer = Arc::new(consumer);
                    addr.do_send(NewConsumer {
                        consumer: Arc::clone(&consumer),
                        peer: participant.clone(),
                    });
                }
                None => {
                    log::error!("can not create Consumer");
                }
            };
        });
    }
}

impl Handler<MessageToPeersInRoom> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: MessageToPeersInRoom, _: &mut Self::Context) -> Self::Result {
        let content = msg.message.into_bytes();
        self.room.peers.iter().for_each(|peer| {
            peer.1.do_send(SendMessage {
                content: content.clone(),
            });
        });
    }
}

impl Handler<MessageToPeerInRoom> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: MessageToPeerInRoom, _: &mut Self::Context) -> Self::Result {
        let peer = self.room.peers.get(&msg.participant).unwrap();
        peer.do_send(SendMessage {
            content: msg.message.into_bytes(),
        });
    }
}

impl Handler<GetRoomInfo> for RoomActor {
    type Result = ();

    fn handle(&mut self, _: GetRoomInfo, _: &mut Self::Context) -> Self::Result {
        let participant_ids: Vec<ParticipantId> = self
            .room
            .peers
            .iter()
            .map(|entity| entity.0.clone())
            .collect();
        let producers = self.room.producers.clone();
        let peer_producers = self.room.peer_producers.clone();
        let peers_addr = self.room.peers.clone();
        actix::spawn(async move {
            let peers = PeerActor::from_registry()
                .send(GetPeers { participant_ids })
                .await
                .unwrap();
            let mut peer_infos: Vec<PeerInfo> = vec![];
            for peer in peers {
                let mut produces: Vec<ProducerDetail> = vec![];
                let producer_ids: Vec<ProducerId> = producers
                    .get(&peer.participant)
                    .map(|p| p.iter().cloned().collect())
                    .unwrap_or_default();

                let producers: Vec<Arc<Producer>> = producer_ids
                    .iter()
                    .filter_map(|id| peer_producers.get(id).cloned())
                    .collect();

                for producer in producers {
                    produces.push(ProducerDetail {
                        kind: producer.kind(),
                        producer_id: producer.id().to_string(),
                        broadcaster_id: peer.participant.peer_id.to_string(),
                        broadcaster_code: peer.participant.user_code.to_string(),
                    });
                }

                peer_infos.push(PeerInfo {
                    id: peer.participant.peer_id.to_string(),
                    user_code: peer.participant.user_code.to_string(),
                    name: peer.name.clone(),
                    produces,
                    joined: true,
                });
            }

            let data = DataPeerInfoRespond {
                event_name: "RoomMediasRespond".to_string(),
                peer_info: peer_infos,
            };

            for (_, addr) in peers_addr {
                addr.do_send(SendMessage {
                    content: serde_json::to_vec(&data).unwrap(),
                });
            }
        });
    }
}

impl Handler<AddTransports> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: AddTransports, ctx: &mut Self::Context) -> Self::Result {
        self.room
            .producer_transports
            .entry(msg.peer.clone())
            .or_insert(msg.producer_transport);
        self.room
            .consumer_transports
            .entry(msg.peer.clone())
            .or_insert(msg.consumer_transport.clone());

        ctx.address().do_send(GetRoomInfo {});
    }
}

impl Handler<NewConsumer> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: NewConsumer, _: &mut Self::Context) -> Self::Result {
        self.room
            .consumers
            .entry(msg.peer.clone())
            .or_default()
            .insert(msg.consumer.id());
        self.room
            .peer_consumers
            .insert(msg.consumer.id(), msg.consumer);
    }
}

impl Handler<NewProducer> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: NewProducer, ctx: &mut Self::Context) -> Self::Result {
        self.room
            .producers
            .entry(msg.peer.clone())
            .or_default()
            .insert(msg.producer.id());
        self.room
            .peer_producers
            .insert(msg.producer.id(), msg.producer);
        ctx.address().do_send(GetRoomInfo {});
    }
}

impl Handler<RoomPauserResumeProducer> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: RoomPauserResumeProducer, ctx: &mut Self::Context) -> Self::Result {
        let participant = msg.peer;
        let producer_id = msg.producer_id;
        let addr = ctx.address().clone();
        let room = self.room.clone();
        actix::spawn(async move {
            let result = room
                .pause_or_resume_producer(participant, producer_id)
                .await;
            match result {
                ActionResult::Success => {
                    addr.do_send(GetRoomInfo {});
                }
                ActionResult::Failed => {
                    log::error!("Invalid producer_id");
                }
            };
        });
    }
}

impl Handler<RoomCloseProducer> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: RoomCloseProducer, ctx: &mut Self::Context) -> Self::Result {
        let producers = self.room.producers.get_mut(&msg.peer);
        if let Some(producers) = producers {
            let is_owner_producer = producers.contains(&msg.producer_id);
            if !is_owner_producer {
                log::error!("Producer invalid");
                return;
            }
            self.room.peer_producers.remove(&msg.producer_id);
            producers.remove(&msg.producer_id);
            ctx.address().do_send(GetRoomInfo {});
        }
    }
}

impl Handler<NewRoomMessage> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: NewRoomMessage, _: &mut Self::Context) -> Self::Result {
        let data = NewRoomMessageDataRespond {
            text: msg.text,
            gif: msg.gif,
            owner_code: msg.participant_id.map(|p| p.user_code),
            r#type: msg.r#type,
        };

        for (_, addr) in self.room.peers.clone() {
            addr.do_send(SendMessage {
                content: serde_json::to_vec(&NewRoomMessageRespond {
                    data: data.clone(),
                    event_name: "NewRoomMessageRespond".to_string(),
                })
                .unwrap(),
            });
        }
    }
}

impl Handler<ClearPeerData> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: ClearPeerData, _: &mut Self::Context) -> Self::Result {
        self.room.peer_leave_room(msg.participant_id);
    }
}

//TODO: Update into database
impl Handler<BannedUserOutRoom> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: BannedUserOutRoom, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address().clone();
        let user_code = msg.user_code.clone();
        if msg.participant_id.user_code != self.room.owner_code {
            log::error!("Permission denied");
            return;
        }

        let peers = self.room.peers.clone();
        let service = self.room.service.clone();
        let room_code = self.room.code.clone();
        actix::spawn(async move {
            let participant_id = PeerActor::from_registry()
                .send(GetPeerByUserCode {
                    user_code: user_code.clone(),
                })
                .await
                .unwrap();

            match participant_id {
                Some(peer) => {
                    let peer_was_banned = peers.get(&peer.participant.clone());
                    if peer_was_banned.is_none() {
                        log::error!("Peer not in room");
                        return;
                    }

                    addr.do_send(ClearPeerData {
                        participant_id: peer.participant.clone(),
                    });
                    addr.do_send(NewRoomMessage {
                        text: Some(format!("{} was banned out of room", peer.name)),
                        gif: None,
                        r#type: super::messages::MessageType::Notification,
                        participant_id: None,
                    });
                    addr.do_send(GetRoomInfo {});

                    peer_was_banned.unwrap().do_send(SendMessage {
                        content: serde_json::to_vec(&RoomBanRespond {
                            event_name: "RoomBannedRespond".to_string(),
                        })
                        .unwrap(),
                    });

                    service.db.peer_leave_room(room_code.0, user_code).await;
                }
                None => {
                    log::error!("invalid user_code");
                }
            };
        });
    }
}

impl Handler<AddPlainTransport> for RoomActor {
    type Result = ();

    fn handle(&mut self, msg: AddPlainTransport, ctx: &mut Self::Context) -> Self::Result {
        self.room.plain_transport = Some(msg.plain_trans);
        ctx.address().do_send(GetRoomInfo {});
    }
}
