use std::vec;

use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};

use crate::{
    infra::mongodb::MongoDb,
    models::room::{CreateRoom, FilterRoom, Room, RoomActive, RoomDetail, RoomQueryResult},
    utils::{generate::generate_string_size, result::Result},
};

use super::AbstractRoom;

pub static COL: &str = "rooms";

#[async_trait]
impl AbstractRoom for MongoDb {
    async fn filter_room(&self, user_code: &str, query: &FilterRoom) -> Result<RoomQueryResult> {
        let skip = query.page - 1;
        let page_size = query.limit;

        let data: Vec<RoomDetail> = self
            .col::<RoomDetail>(COL)
            .aggregate([
                doc! {
                    "$match": {
                        "blacklist": { "$nin": [user_code] }
                    }
                },
                doc! {
                  "$lookup": {
                    "from": "users",
                    "localField": "ownerCode",
                    "foreignField": "code",
                    "as": "owner",
                    "pipeline": [
                        {
                          "$project": {
                            "name": 1,
                            "code": 1,
                            "avatar": 1,
                          },
                        }
                      ]
                  }
                },
                doc! {
                    "$lookup": {
                        "from": "users",
                        "localField": "participants",
                        "foreignField": "code",
                        "as": "users",
                        "pipeline": [
                        {
                          "$project": {
                            "name": 1,
                            "code": 1,
                            "avatar": 1,
                          },
                        }
                      ]
                    }
                },
                doc! { "$unwind": "$owner" },
                doc! { "$limit": page_size  },
                doc! { "$skip": skip },
                doc! { "$sort": { "createdAt" : 1 } },
            ])
            .await
            .expect("Unable to aggregate posts")
            .with_type()
            .try_collect()
            .await
            .expect("Unable to collect items from Cursor");

        let total = self
            .col::<Room>(COL)
            .count_documents(doc! {
                    "match" : {
                        "blacklist": { "$nin": user_code }
                    }
            })
            .await
            .unwrap();

        let result = RoomQueryResult {
            data,
            total: total as i32,
            page: query.page,
        };
        Ok(result)
    }

    async fn get_room(&self, room_code: &str) -> Option<Room> {
        self.col::<Room>(COL)
            .find_one(doc! {
                "code":room_code.to_string()
            })
            .await
            .unwrap()
    }

    async fn create_room(&self, user_code: &str, payload: &CreateRoom) -> Option<String> {
        let mut code = generate_string_size(10);
        if payload.room_code.is_some() {
            code = payload.room_code.as_ref().unwrap().to_string();
        }
        let now = Utc::now();
        let timestamp_millis = now.timestamp_millis();
        let res = self
            .col::<Room>(COL)
            .insert_one(Room {
                id: ObjectId::new().to_string(),
                owner_code: user_code.to_string(),
                blacklist: vec![],
                participants: vec![],
                code: code.clone(),
                r#type: "PUBLIC".to_string(),
                limit: 10,
                expired_at: None,
                display_name: payload.display_name.clone(),
                room_start: None,
                active_participants: 0,
                created_at: timestamp_millis,
                category: Some(payload.category.clone()),
                description: payload.description.clone(),
            })
            .await;

        match res {
            Ok(_) => Some(code.clone()),
            Err(_) => None,
        }
    }

    async fn new_participant(&self, room_code: &str, user_code: &str, peer_id: &str) -> bool {
        self.col::<Room>(COL)
            .update_one(
                doc! {"code": room_code},
                doc! {
                 "$push":{
                     "participants":{ "userCode": user_code, "clientId": peer_id}
                 },
                 "activeParticipants": { "$inc": 1 }
                },
            )
            .await
            .is_ok()
    }

    async fn participant_leave(&self, room_code: &str, user_code: &str, peer_id: &str) -> bool {
        self.col::<Room>(COL)
            .update_one(
                doc! {"code": room_code},
                doc! {
                 "$pull":{
                     "participants": { "userCode": user_code, "clientId": peer_id }
                 },
                 "activeParticipants": { "$inc": -1 }
                },
            )
            .await
            .is_ok()
    }

    async fn delete_room(&self, room_code: &str) {
        let _ = self
            .col::<Room>(COL)
            .delete_one(doc! {
                "code": room_code.to_string()
            })
            .await;
    }

    async fn get_room_actives(&self) -> Vec<RoomActive> {
        let result: Vec<RoomActive> = self
            .col::<RoomActive>(COL)
            .find(doc! {})
            .projection(doc! { "code": 1, "_id": 0, "ownerCode":1 })
            .await
            .expect("Unable to aggregate posts")
            .with_type()
            .try_collect()
            .await
            .expect("Unable to collect items from Cursor");

        result.to_vec()
    }

    async fn get_room_info(&self, room_code: &str) -> Option<RoomDetail> {
        let data: Vec<RoomDetail> = self
            .col::<RoomDetail>(COL)
            .aggregate([
                doc! {
                    "$match": {
                        "code": room_code
                    }
                },
                doc! {
                  "$lookup": {
                    "from": "users",
                    "localField": "ownerCode",
                    "foreignField": "code",
                    "as": "owner",
                    "pipeline": [
                        {
                          "$project": {
                            "name": 1,
                            "code": 1,
                            "avatar": 1,
                          },
                        }
                      ]
                  }
                },
                doc! {
                    "$lookup": {
                        "from": "users",
                        "localField": "participants",
                        "foreignField": "code",
                        "as": "users",
                        "pipeline": [
                        {
                          "$project": {
                            "name": 1,
                            "code": 1,
                            "avatar": 1,
                          },
                        }
                      ]
                    }
                },
                doc! { "$unwind": "$owner" },
                doc! { "$limit": 1  },
            ])
            .await
            .expect("Unable to aggregate posts")
            .with_type()
            .try_collect()
            .await
            .expect("Unable to collect items from Cursor");

        data.into_iter().next()
    }

    async fn peer_leave_room(&self, user_code: String, room_code: String) {
        let res = self
            .col::<Document>(COL)
            .update_one(
                doc! {
                    "code": room_code
                },
                doc! {
                    "$pull": {
                        "participants": user_code
                    },
                   "$inc": {
                        "activeParticipants": -1
                    }
                },
            )
            .await;
        match res {
            Ok(_) => println!("peer_leave_room success"),
            Err(err) => println!("peer_leave_room failed: {}", err),
        }
    }

    async fn peer_join_room(&self, room_code: String, user_code: String) {
        let res = self
            .col::<Document>(COL)
            .update_one(
                doc! {
                    "code": room_code
                },
                doc! {
                    "$push": {
                        "participants": user_code
                    },
                    "$inc": {
                        "activeParticipants": 1
                    }
                },
            )
            .await;
        match res {
            Ok(_) => println!("peer_join_room success"),
            Err(err) => println!("peer_join_room failed: {}", err),
        }
    }

    async fn set_room_empty(&self) {
        let _ = self
            .col::<Document>(COL)
            .update_many(
                doc! {},
                doc! {
                    "$set": {
                        "activeParticipants": 0,
                        "participants": []
                    }
                },
            )
            .await;
    }
}
