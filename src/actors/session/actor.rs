use actix::{Actor, Addr, AsyncContext, SystemService};
use actix_web_actors::ws;
use mediasoup::prelude::RtpCapabilities;
use std::sync::Arc;
use std::time::Instant;

use crate::{
    actors::{
        chat::actor::Chat,
        peer::messages::{NewPeer, PeerDisconnect},
        room_session::messages::PeerLeaveRoomSession,
    },
    models::{
        peer_session::{ParticipantId, ParticipantType},
        room_session::RoomSession,
    },
    services::peer_service::PeerService,
    utils::generate::generate_string,
};

#[derive(Clone)]
pub struct Session {
    pub participant_id: ParticipantId,
    pub room_addr: Option<Addr<RoomSession>>,
    pub rtp_capabilities: Option<RtpCapabilities>,
    pub chat_addr: Arc<Addr<Chat>>,
    pub name: String,
    pub hb: Instant,
}

impl Session {
    pub fn new(user_code: String, name: String, chat_addr: Arc<Addr<Chat>>) -> Self {
        let pid = generate_string();
        let participant_id = ParticipantId {
            user_code,
            peer_id: pid,
            r#type: ParticipantType::User,
        };

        Self {
            participant_id,
            room_addr: None,
            rtp_capabilities: None,
            name,
            chat_addr,
            hb: Instant::now(),
        }
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        log::info!("Session disconnected");
        PeerService::from_registry().do_send(PeerDisconnect {
            participant_id: self.participant_id.clone(),
        });

        if let Some(room) = &self.room_addr {
            room.do_send(PeerLeaveRoomSession {
                participant_id: self.participant_id.clone(),
                name: self.name.clone(),
            });
        }
    }

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("new session connect");
        PeerService::from_registry().do_send(NewPeer {
            participant_id: self.participant_id.clone(),
            name: self.name.clone(),
            addr: ctx.address(),
        });
    }
}
