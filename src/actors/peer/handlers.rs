use crate::{
    actors::{
        room_management::{actors::RoomManager, messages::GetRoomAddr},
        room_session::messages::PeerJoinRoomSession,
        session::messages::SendMessage,
    },
    models::{peer_session::PeerSession, room_session::RoomCode},
    services::peer_service::PeerService,
    utils::to_binary,
};
use actix::{AsyncContext, Handler, SystemService};

use super::messages::{
    ErrorEvent, ErrorEventByUser, GetPeerByUserCode, GetPeers, JoinRoom, NewPeer, PeerDisconnect,
    PeerJoinRoom, PeerLeaveRoom, SendPeer, SendPeerByUser, SendPeers, SendPeersByUser,
};

mod error_event {
    include!(concat!(env!("OUT_DIR"), "/error_event.rs"));
}

impl Handler<GetPeers> for PeerService {
    type Result = Vec<PeerSession>;

    fn handle(&mut self, msg: GetPeers, _: &mut Self::Context) -> Self::Result {
        let peers: Vec<PeerSession> = msg
            .participant_ids
            .iter()
            .filter_map(|id| self.peers.get(id).cloned())
            .collect();

        peers
    }
}

impl Handler<PeerJoinRoom> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: PeerJoinRoom, ctx: &mut Self::Context) -> Self::Result {
        let session = msg.session.clone();
        let peer = self
            .peers
            .get(&session.participant_id.clone())
            .unwrap()
            .clone();
        let room_code = msg.room_code.clone();
        let ss_addr = msg.addr.clone();
        let peer_service_addr = ctx.address().clone();
        actix::spawn(async move {
            let room_code = RoomCode(room_code.clone());
            let room = RoomManager::from_registry()
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
impl Handler<PeerLeaveRoom> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: PeerLeaveRoom, _: &mut Self::Context) -> Self::Result {
        let peer = self.peers.get_mut(&msg.participant_id).unwrap();
        peer.room_code = None;
    }
}

impl Handler<GetPeerByUserCode> for PeerService {
    type Result = Option<PeerSession>;

    fn handle(&mut self, msg: GetPeerByUserCode, _: &mut Self::Context) -> Self::Result {
        let participant_id = self.user_code_to_participant_id.get(&msg.user_code);

        match participant_id {
            Some(participant_id) => self.peers.get(participant_id).cloned(),
            None => None,
        }
    }
}

impl Handler<SendPeer> for PeerService {
    type Result = ();
    fn handle(&mut self, msg: SendPeer, _: &mut Self::Context) -> Self::Result {
        let peer = self.peers.get(&msg.participant_id);
        if let Some(peer) = peer {
            peer.addr.do_send(SendMessage { content: msg.data });
        }
    }
}

impl Handler<SendPeerByUser> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: SendPeerByUser, _: &mut Self::Context) -> Self::Result {
        let participant_id = self.user_code_to_participant_id.get(&msg.user_code);
        if let Some(participant_id) = participant_id {
            let peer = self.peers.get(participant_id);
            if let Some(peer) = peer {
                peer.addr.do_send(SendMessage { content: msg.data });
            }
        }
    }
}

impl Handler<SendPeersByUser> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: SendPeersByUser, _: &mut Self::Context) -> Self::Result {
        for user_code in msg.user_codes {
            let participant_id = self.user_code_to_participant_id.get(&user_code);
            if let Some(participant_id) = participant_id {
                let peer = self.peers.get(participant_id);
                if let Some(peer) = peer {
                    peer.addr.do_send(SendMessage {
                        content: msg.data.clone(),
                    });
                }
            }
        }
    }
}

impl Handler<ErrorEvent> for PeerService {
    type Result = ();
    fn handle(&mut self, msg: ErrorEvent, _: &mut Self::Context) -> Self::Result {
        let peer = self.peers.get(&msg.participant_id);
        if let Some(peer_addr) = peer {
            peer_addr.addr.do_send(SendMessage {
                content: to_binary(&error_event::ErrorEvent {
                    event: msg.event_name,
                    message: msg.data,
                }),
            });
        }
    }
}

impl Handler<ErrorEventByUser> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: ErrorEventByUser, _: &mut Self::Context) -> Self::Result {
        let participant_id = self.user_code_to_participant_id.get(&msg.user_code);
        if let Some(participant_id) = participant_id {
            let peer = self.peers.get(participant_id);
            if let Some(peer) = peer {
                peer.addr.do_send(SendMessage {
                    content: to_binary(&error_event::ErrorEvent {
                        event: msg.event_name,
                        message: msg.data,
                    }),
                });
            }
        }
    }
}

impl Handler<SendPeers> for PeerService {
    type Result = ();
    fn handle(&mut self, msg: SendPeers, _: &mut Self::Context) -> Self::Result {
        for participant_id in msg.participant_ids {
            let peer = self.peers.get(&participant_id);
            if let Some(peer_addr) = peer {
                peer_addr.addr.do_send(SendMessage {
                    content: msg.data.clone(),
                });
            }
        }
    }
}

impl Handler<NewPeer> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: NewPeer, _: &mut Self::Context) -> Self::Result {
        log::info!("new peer");
        self.new_peer(msg.participant_id, msg.name, msg.addr);
    }
}

impl Handler<PeerDisconnect> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: PeerDisconnect, _: &mut Self::Context) -> Self::Result {
        self.peers.remove(&msg.participant_id);
        self.users.remove(&msg.participant_id.user_code.clone());
        self.user_code_to_participant_id
            .remove(&msg.participant_id.user_code.clone());
    }
}

impl Handler<JoinRoom> for PeerService {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _: &mut Self::Context) -> Self::Result {
        if let Some(peer) = self.peers.get_mut(&msg.participant_id) {
            peer.room_code = Some(msg.room_code);
            peer.room_addr = Some(msg.room_addr);
        }
    }
}
