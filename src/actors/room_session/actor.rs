use crate::models::room_session::RoomSession;
use actix::{Actor, Context};

pub struct RoomActor {
    pub room: RoomSession,
}

impl RoomActor {
    pub fn new(room: RoomSession) -> Self {
        Self { room }
    }
}

impl Actor for RoomActor {
    type Context = Context<Self>;
}

unsafe impl Send for RoomActor {}
unsafe impl Sync for RoomActor {}
