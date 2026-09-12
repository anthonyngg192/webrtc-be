use crate::{
    actors::session::{actor::Session, messages::SessionDisconnect},
    models::peer_session::{ParticipantId, PeerSession},
};
use actix::Addr;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub struct PeerService {
    pub peers: HashMap<ParticipantId, PeerSession>,
    pub users: HashSet<String>,
    pub user_code_to_participant_id: HashMap<String, ParticipantId>,
}

impl Default for PeerService {
    fn default() -> Self {
        Self::new()
    }
}

impl PeerService {
    pub fn new() -> Self {
        Self {
            peers: HashMap::new(),
            users: HashSet::new(),
            user_code_to_participant_id: HashMap::new(),
        }
    }

    pub fn new_peer(
        &mut self,
        participant_id: ParticipantId,
        name: String,
        addr: Addr<Session>,
    ) -> bool {
        let new_peer = PeerSession {
            participant: participant_id.clone(),
            room_code: None,
            rtp_capabilities: None,
            name: name.clone(),
            addr: Arc::new(addr),
            room_addr: None,
        };
        if self.users.contains(&participant_id.user_code.clone()) {
            let old_participant_id = self
                .user_code_to_participant_id
                .get_mut(&participant_id.user_code.clone())
                .unwrap();
            let latest_peer = self.peers.get_mut(&old_participant_id.clone()).unwrap();
            let old_addr = latest_peer.addr.clone();
            old_addr.do_send(SessionDisconnect {});

            self.peers.remove(&old_participant_id.clone());
            self.user_code_to_participant_id
                .remove(&participant_id.user_code.clone());
            self.users.remove(&participant_id.user_code.clone());
        }

        self.peers.insert(participant_id.clone(), new_peer);
        self.users.insert(participant_id.user_code.clone());
        self.user_code_to_participant_id
            .insert(participant_id.user_code.clone(), participant_id);
        true
    }
}
