use super::messages::*;
use crate::{
    actors::{
        peer::actor::PeerActor,
        room_management::{actors::RoomsActor, messages::GetRoomAddr},
        room_session::messages::PeerJoinRoomSession,
        session::messages::SendMessage,
    },
    models::{peer_session::PeerSession, room_session::RoomCode},
};
use actix::{AsyncContext, Handler, SystemService};

impl Handler<GetPeers> for PeerActor {
    type Result = Vec<PeerSession>;

    fn handle(&mut self, msg: GetPeers, _: &mut Self::Context) -> Self::Result {
        let peers: Vec<PeerSession> = msg
            .participant_ids
            .iter()
            .filter_map(|id| self.service.peers.get(id).cloned())
            .collect();

        peers
    }
}

impl Handler<PeerJoinRoom> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinRoom, ctx: &mut Self::Context) -> Self::Result {
        let session = msg.session.clone();
        let peer = self
            .service
            .peers
            .get(&session.participant_id.clone())
            .unwrap()
            .clone();
        let room_code = msg.room_code.clone();
        let ss_addr = msg.addr.clone();
        let peer_service_addr = ctx.address().clone();
        actix::spawn(async move {
            let room_code = RoomCode(room_code.clone());
            let room = RoomsActor::from_registry()
                .send(GetRoomAddr {
                    room_code: room_code.clone(),
                })
                .await
                .unwrap();
            match room {
                Some(room) => {
                    room.do_send(PeerJoinRoomSession {
                        room_code: room_code.clone(),
                        session: session.clone(),
                        peer_addr: ss_addr.clone(),
                    });

                    peer_service_addr.do_send(JoinRoom {
                        room_code: room_code.clone(),
                        participant_id: peer.participant,
                        room_addr: room,
                    });
                }
                None => {
                    log::error!("Room not found with ");
                }
            }
        });
    }
}

//TODO: change it to state
impl Handler<PeerLeaveRoom> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: PeerLeaveRoom, _: &mut Self::Context) -> Self::Result {
        let peer = self.service.peers.get_mut(&msg.participant_id).unwrap();
        peer.room_code = None;
    }
}

impl Handler<GetPeerByUserCode> for PeerActor {
    type Result = Option<PeerSession>;

    fn handle(&mut self, msg: GetPeerByUserCode, _: &mut Self::Context) -> Self::Result {
        let participant_id = self.service.user_code_to_participant_id.get(&msg.user_code);

        match participant_id {
            Some(participant_id) => self.service.peers.get(participant_id).cloned(),
            None => None,
        }
    }
}

impl Handler<SendPeer> for PeerActor {
    type Result = ();
    fn handle(&mut self, msg: SendPeer, _: &mut Self::Context) -> Self::Result {
        let peer = self.service.peers.get(&msg.participant_id);
        if let Some(peer) = peer {
            peer.addr.do_send(SendMessage { content: msg.data });
        }
    }
}

impl Handler<SendPeerByUser> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: SendPeerByUser, _: &mut Self::Context) -> Self::Result {
        let participant_id = self.service.user_code_to_participant_id.get(&msg.user_code);
        if let Some(participant_id) = participant_id {
            let peer = self.service.peers.get(participant_id);
            if let Some(peer) = peer {
                peer.addr.do_send(SendMessage { content: msg.data });
            }
        }
    }
}

impl Handler<SendPeersByUser> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: SendPeersByUser, _: &mut Self::Context) -> Self::Result {
        for user_code in msg.user_codes {
            let participant_id = self.service.user_code_to_participant_id.get(&user_code);
            let Some(participant_id) = participant_id else {
                continue;
            };
            let Some(peer) = self.service.peers.get(participant_id) else {
                continue;
            };
            peer.addr.do_send(SendMessage {
                content: msg.data.clone(),
            });
        }
    }
}

impl Handler<SendPeers> for PeerActor {
    type Result = ();
    fn handle(&mut self, msg: SendPeers, _: &mut Self::Context) -> Self::Result {
        for participant_id in msg.participant_ids {
            if let Some(peer) = self.service.peers.get(&participant_id) {
                peer.addr.do_send(SendMessage {
                    content: msg.data.clone(),
                });
            }
        }
    }
}

impl Handler<NewPeer> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: NewPeer, _: &mut Self::Context) -> Self::Result {
        log::info!("new peer");
        self.service
            .new_peer(msg.participant_id, msg.name, msg.addr);
    }
}

impl Handler<PeerDisconnect> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: PeerDisconnect, _: &mut Self::Context) -> Self::Result {
        self.service.peers.remove(&msg.participant_id);
        self.service
            .users
            .remove(&msg.participant_id.user_code.clone());
        self.service
            .user_code_to_participant_id
            .remove(&msg.participant_id.user_code.clone());
    }
}

impl Handler<JoinRoom> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _: &mut Self::Context) -> Self::Result {
        if let Some(peer) = self.service.peers.get_mut(&msg.participant_id) {
            peer.room_code = Some(msg.room_code);
            peer.room_addr = Some(msg.room_addr);
        }
    }
}

impl Handler<ErrorEventByUser> for PeerActor {
    type Result = ();

    fn handle(&mut self, msg: ErrorEventByUser, _: &mut Self::Context) -> Self::Result {
        let participant_id = self.service.user_code_to_participant_id.get(&msg.user_code);
        if let Some(participant_id) = participant_id {
            let peer = self.service.peers.get(participant_id);
            if let Some(_) = peer {
                // peer_ss.addr.do_send(SendMessage {
                //     content: serde_json::to_vec(ErrorEvent {
                //         event: msg.event_name,
                //         message: msg.data,
                //     })
                //     .unwrap(),
                // });
            }
        }
    }
}
