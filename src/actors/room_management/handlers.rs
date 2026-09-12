use crate::{
    actors::{room_management::messages::NewRoom, room_session::actor::RoomActor},
    models::room_session::RoomSession,
};
use actix::{Actor, Addr, AsyncContext, Handler};
use std::sync::Arc;

use super::{actors::RoomsActor, messages::*};

impl Handler<AddWorker> for RoomsActor {
    type Result = ();

    fn handle(&mut self, msg: AddWorker, _: &mut Self::Context) {
        self.add_worker(msg);
    }
}

impl Handler<GetRoomAddr> for RoomsActor {
    type Result = Option<Arc<Addr<RoomActor>>>;

    fn handle(&mut self, msg: GetRoomAddr, _: &mut Self::Context) -> Self::Result {
        log::info!("total: {}", self.rooms.len());
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

impl Handler<RoomManagementCreateRoom> for RoomsActor {
    type Result = ();

    fn handle(&mut self, msg: RoomManagementCreateRoom, ctx: &mut Self::Context) -> Self::Result {
        let room_code = msg.room_code.clone();
        let owner_code = msg.owner_code.clone();
        let addr = ctx.address().clone();
        let service = msg.room_service.clone();
        let worker = self.get_worker();

        match worker {
            Some(result_worker) => {
                let worker = result_worker.0;
                let webrtc_server = result_worker.1;
                actix::spawn(async move {
                    let room_session = RoomSession::new(
                        &worker,
                        webrtc_server.clone(),
                        room_code.clone(),
                        owner_code,
                        service,
                    )
                    .await;
                    match room_session {
                        Ok(room) => {
                            let room_addr = RoomActor::new(room).start();
                            addr.do_send(NewRoom {
                                room_code: msg.room_code,
                                addr: room_addr,
                            });
                            log::info!("Create room {} success", room_code.0.clone());
                        }
                        Err(msg) => {
                            log::error!("can not create new room with issue {:#?}", msg);
                        }
                    }
                });
            }
            None => {
                log::error!("create room got error");
            }
        }
    }
}

impl Handler<NewRoom> for RoomsActor {
    type Result = ();

    fn handle(&mut self, msg: NewRoom, _: &mut Self::Context) -> Self::Result {
        self.rooms.insert(msg.room_code, Arc::new(msg.addr));
    }
}
