use std::sync::Arc;

use crate::{
    adapters::redis_adapter::RedisAdapter,
    infra::mongodb::MongoDb,
    models::room::{CreateRoom, FilterRoom, RoomActive, RoomDetail, RoomInfo, RoomQueryResult},
    repositories::AbstractRoom,
    utils::result::{Error, Result},
};

pub struct RoomService {
    pub db: Arc<MongoDb>,
    pub redis_pool: Arc<RedisAdapter>,
}

impl RoomService {
    pub fn new(db: Arc<MongoDb>, redis_pool: Arc<RedisAdapter>) -> Self {
        Self { db, redis_pool }
    }

    pub async fn filter(&self, user_code: &str, query: &FilterRoom) -> Result<RoomQueryResult> {
        let result = self.db.filter_room(user_code, query).await;
        match result {
            Ok(result) => Ok(result),
            Err(_) => Err(Error::BadRequest),
        }
    }

    pub async fn new_room(&self, user_code: &str, payload: &CreateRoom) -> Option<RoomInfo> {
        let room_code = match &payload.room_code {
            Some(code) => self
                .db
                .get_room(&code.to_string())
                .await
                .map(|room| RoomInfo {
                    code: room.code,
                    owner_code: room.owner_code,
                }),
            None => None,
        };

        if let Some(info) = room_code {
            return Some(info);
        }

        if let Some(new_code) = self.db.create_room(user_code, payload).await {
            return Some(RoomInfo {
                code: new_code,
                owner_code: user_code.to_string(),
            });
        }

        None
    }

    pub async fn new_participant(&self, room_code: &str, user_code: &str, peer_id: &str) -> bool {
        self.db.new_participant(room_code, user_code, peer_id).await
    }

    pub async fn get_room_actives(&self) -> Vec<RoomActive> {
        self.db.get_room_actives().await
    }

    pub async fn pre_reload_room_to_ram(&self) {
        self.db.set_room_empty().await;
    }

    pub async fn get_room_info(&self, room_code: &str) -> Result<RoomDetail> {
        let room = self.db.get_room_info(room_code).await;
        if room.is_none() {
            return Err(Error::RoomNotFound);
        }

        //TODO: update feature at here.
        Ok(room.unwrap())
    }
}
