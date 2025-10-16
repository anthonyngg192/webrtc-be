use super::AbstractConversation;
use crate::{
    infra::mongodb::MongoDb,
    models::{
        conversation::{
            Conversation, ConversationDetail, ConversationQueryResult, FilterConversation,
        },
        message::Message,
    },
    utils::result::{Error, Result},
};
use chrono::Utc;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, oid::ObjectId};

pub static COL: &str = "conversations";

#[async_trait]
impl AbstractConversation for MongoDb {
    async fn get_conversation(&self, id: &str) -> Result<Conversation> {
        let _id = ObjectId::parse_str(id).unwrap();
        println!("id: {}", _id);
        let result = self
            .col::<Conversation>(COL)
            .find_one(doc! {
                "_id": _id
            })
            .await;

        match result {
            Ok(data) => match data {
                Some(conversation) => Ok(conversation),
                None => Err(Error::NotFound),
            },
            Err(_) => Err(Error::NotFound),
        }
    }

    async fn filter_conversations(
        &self,
        user_code: &str,
        filter: &FilterConversation,
    ) -> Result<ConversationQueryResult> {
        let skip = filter.page - 1;
        let page_size = filter.limit;

        let mut data: Vec<ConversationDetail> = self
            .col::<ConversationDetail>(COL)
            .aggregate([
                doc! {
                    "$match":{
                        "createdAt":{
                            "$gte":0
                        },
                        "conversationMembersCodes":{ "$in": [user_code]}
                    }
                },
                doc! {
                  "$lookup": {
                    "from": "users",
                    "localField": "conversationMembersCodes",
                    "foreignField": "code",
                    "as": "userDetails",
                    "pipeline": [
                        {
                          "$project": {
                            "name": 1,
                            "code": 1,
                            "avatar": 1,
                          },
                        },
                      ]
                  }
                },
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

        for item in data.iter_mut() {
            item.user = item
                .user_details
                .iter()
                .find(|&x| x.code != user_code)
                .cloned();
        }

        let total = self
            .col::<Conversation>(COL)
            .count_documents(doc! {
                    "createdAt" : {
                        "$gte" : 0
                    },
                    "conversationMembersCodes":{ "$in": [user_code]}
            })
            .await
            .unwrap();

        let result = ConversationQueryResult {
            data,
            total: total as i32,
            page: filter.page,
        };
        Ok(result)
    }

    async fn new_conversation(&self, user_code: &str, other_user_code: &str) -> Result<()> {
        let is_exist_conversation = self
            .col::<Conversation>(COL)
            .find_one(doc! {
                "conversationMembersCodes": {
                    "$all": [user_code, other_user_code]
                }
            })
            .await
            .unwrap();
        match is_exist_conversation {
            Some(_) => Ok(()),
            None => {
                let now = Utc::now();
                let timestamp_millis = now.timestamp_millis();
                let id = ObjectId::new();
                self.col::<Conversation>(COL)
                    .insert_one(Conversation {
                        id,
                        conversation_members_codes: vec![
                            user_code.to_string(),
                            other_user_code.to_string(),
                        ],
                        created_at: timestamp_millis as f64,
                        updated_at: timestamp_millis as f64,
                        last_message: None,
                    })
                    .await
                    .unwrap();
                Ok(())
            }
        }
    }

    async fn check_conversation_valid(&self, user_code: &str, conversation_id: &str) -> String {
        let _id = ObjectId::parse_str(conversation_id);
        if _id.is_err() {
            return "None".to_string();
        }

        let result = self
            .col::<Conversation>(COL)
            .find_one(doc! {
                "_id": _id.unwrap(),
                "conversationMembersCodes": {
                    "$in": [user_code]
                }
            })
            .await
            .unwrap();

        match result {
            Some(conversation) => {
                let other_user_code = conversation
                    .conversation_members_codes
                    .iter()
                    .find(|&r| r != user_code)
                    .unwrap();
                other_user_code.to_string()
            }
            None => "None".to_string(),
        }
    }

    async fn set_last_message(&self, conversation_id: &str, message: &Message) -> Result<()> {
        let message_bson = mongodb::bson::to_bson(message).unwrap();

        self.col::<Conversation>(COL)
            .update_one(
                doc! {"_id": ObjectId::parse_str(conversation_id).unwrap()},
                doc! { "$set": { "lastMessage": message_bson } },
            )
            .await
            .unwrap();
        Ok(())
    }
}
