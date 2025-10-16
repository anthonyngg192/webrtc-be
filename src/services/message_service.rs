use std::sync::Arc;

use crate::{
    infra::mongodb::MongoDb,
    models::{
        conversation::FilterMessage,
        message::{Message, MessageQueryResult, NewMessage},
    },
    repositories::{AbstractConversation, AbstractMessage},
    utils::result::{Error, Result},
};

pub struct MessageService {
    pub db: Arc<MongoDb>,
}

impl MessageService {
    pub fn new(db: Arc<MongoDb>) -> Self {
        Self { db }
    }

    pub async fn filter_message(
        &self,
        conversation_id: &str,
        user_code: &str,
        filter: &FilterMessage,
    ) -> Result<MessageQueryResult> {
        let allow_filter = self
            .db
            .check_conversation_valid(user_code, conversation_id)
            .await;

        if allow_filter == *"None" {
            return Err(Error::BadRequest);
        }
        let res = self.db.filter_messages(conversation_id, filter).await;
        match res {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    pub async fn new_message(
        &self,
        conversation_id: &str,
        user_code: &str,
        message: &NewMessage,
    ) -> Result<Message> {
        let res = self
            .db
            .new_message(conversation_id, user_code, message)
            .await;
        match res {
            Ok(new_message) => {
                let _ = self
                    .db
                    .set_last_message(conversation_id, &new_message)
                    .await;
                Ok(new_message)
            }
            Err(e) => Err(e),
        }
    }
}
