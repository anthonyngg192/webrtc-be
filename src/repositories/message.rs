use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    infra::mongodb::MongoDb,
    models::{
        conversation::FilterMessage,
        message::{
            Message, MessageDetail, MessageOwnerType, MessageQueryResult, MessageType, NewMessage,
        },
    },
    utils::result::{Error, Result},
};

use super::AbstractMessage;

pub static COL: &str = "messages";

#[async_trait]
impl AbstractMessage for MongoDb {
    async fn filter_messages(
        &self,
        conversation_id: &str,
        filter: &FilterMessage,
    ) -> Result<MessageQueryResult> {
        let skip = filter.page - 1;
        let page_size = filter.limit;

        let data: Vec<MessageDetail> = self
            .col::<MessageDetail>(COL)
            .aggregate([
                doc! {
                    "$match":{
                        "conversationId" : ObjectId::parse_str(conversation_id).unwrap()
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
                        },
                        {"$limit": 1}
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
            .col::<Message>(COL)
            .count_documents(doc! {
                "createdAt" : {
                    "$gte" : 0
                },
                "conversationId" : ObjectId::parse_str(conversation_id).unwrap()
            })
            .await
            .unwrap();

        let result = MessageQueryResult {
            data,
            total: total as i32,
            page: filter.page,
        };
        Ok(result)
    }

    async fn new_message(
        &self,
        conversation_id: &str,
        user_code: &str,
        message: &NewMessage,
    ) -> Result<Message> {
        let id = ObjectId::new();
        let now = Utc::now();
        let timestamp_millis = now.timestamp_millis();
        let new_message = Message {
            id,
            conversation_id: ObjectId::parse_str(conversation_id).unwrap(),
            content: message.content.clone(),
            gif: message.gif.clone(),
            images: vec![],
            owner_code: Some(user_code.to_string().clone()),
            owner_type: MessageOwnerType::User,
            message_type: MessageType::Message,
            created_at: Some(timestamp_millis as f64),
            metadata: None,
        };

        let res = self
            .col::<Message>(COL)
            .insert_one(new_message.clone())
            .await;
        match res {
            Ok(_) => Ok(new_message),
            Err(_) => Err(Error::BadRequest),
        }
    }
}
