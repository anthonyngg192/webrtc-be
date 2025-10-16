use crate::{actors::room_management::messages::NewRoom, models::room_session::RoomSession};
use actix::{Actor, Addr, AsyncContext, Handler};
use mediasoup::{prelude::WebRtcServer, worker::Worker};
use std::sync::Arc;

use super::{
    actors::RoomManager,
    messages::{AddWorker, GetRoomAddr, GetWorker, RoomManagementCreateRoom},
};

impl Handler<AddWorker> for RoomManager {
    type Result = ();

    fn handle(&mut self, msg: AddWorker, _: &mut Self::Context) {
        self.workers.insert(msg.id, msg.worker);
        self.webrtc_servers
            .insert(msg.id, Arc::new(msg.webrtc_server));
    }
}

impl Handler<GetWorker> for RoomManager {
    type Result = Option<(Worker, Arc<WebRtcServer>)>;

    fn handle(&mut self, _: GetWorker, _: &mut Self::Context) -> Self::Result {
        if self.workers.is_empty() || self.webrtc_servers.is_empty() {
            return None;
        }
        let mut index = self.current_index.write().unwrap();
        let worker = self.workers.get(&*index).cloned().or_else(|| {
            *index = 1;
            self.workers.get(&1).cloned()
        });
        let webrtc_server = self.webrtc_servers.get(&*index).cloned().or_else(|| {
            *index = 1;
            self.webrtc_servers.get(&1).cloned()
        });
        *index = (*index + 1) % self.workers.len();
        match (worker, webrtc_server) {
            (Some(worker), Some(webrtc_server)) => Some((worker, webrtc_server)),
            _ => None,
        }
    }
}

impl Handler<GetRoomAddr> for RoomManager {
    type Result = Option<Arc<Addr<RoomSession>>>;

    fn handle(&mut self, msg: GetRoomAddr, _: &mut Self::Context) -> Self::Result {
        let room = self.rooms.get(&msg.room_code.clone());
        match room {
            Some(room) => Some(room.clone()),
            None => {
                log::error!("room not found with {}", msg.room_code.0);
                None
            }
        }
    }
}

impl Handler<RoomManagementCreateRoom> for RoomManager {
    type Result = ();

    fn handle(&mut self, msg: RoomManagementCreateRoom, ctx: &mut Self::Context) -> Self::Result {
        let room_code = msg.room_code.clone();
        let owner_code = msg.owner_code.clone();
        let addr = ctx.address().clone();
        let service = msg.room_service.clone();
        actix::spawn(async move {
            let result_worker = addr.send(GetWorker {}).await.unwrap();
            match result_worker {
                Some(result_worker) => {
                    let worker = result_worker.0;
                    let webrtc_server = result_worker.1;
                    let room: RoomSession = RoomSession::new(
                        &worker.clone(),
                        webrtc_server.clone(),
                        room_code.clone(),
                        owner_code,
                        service,
                    )
                    .await
                    .unwrap();
                    let room_addr = room.start().clone();
                    addr.do_send(NewRoom {
                        room_code: msg.room_code.clone(),
                        addr: room_addr,
                    });
                    log::info!("Create room {} success", room_code.0.clone());
                }
                None => {
                    log::error!("create room got error");
                }
            }
        });
    }
}

impl Handler<NewRoom> for RoomManager {
    type Result = ();

    fn handle(&mut self, msg: NewRoom, _: &mut Self::Context) -> Self::Result {
        self.rooms.insert(msg.room_code.clone(), Arc::new(msg.addr));
    }
}
