use std::sync::Arc;

use crate::{
    adapters::redis_adapter::RedisAdapter,
    infra::mongodb::MongoDb,
    models::{
        conversation::{Conversation, ConversationQueryResult, FilterConversation},
        message::Message,
    },
    repositories::AbstractConversation,
    utils::result::Result,
};

pub struct ConversationService {
    pub db: Arc<MongoDb>,
    pub redis_pool: Arc<RedisAdapter>,
}

impl ConversationService {
    pub fn new(db: Arc<MongoDb>, redis_pool: Arc<RedisAdapter>) -> Self {
        Self { db, redis_pool }
    }

    pub async fn filter_conversation(
        &self,
        user_code: &str,
        filter: &FilterConversation,
    ) -> Result<ConversationQueryResult> {
        let res = self.db.filter_conversations(user_code, filter).await;
        match res {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    pub async fn get_conversation(&self, conversation_id: &str) -> Option<Conversation> {
        let res = self.db.get_conversation(conversation_id).await;
        match res {
            Ok(conversation) => Some(conversation),
            Err(_) => None,
        }
    }

    pub async fn get_other_user_in_conversation(
        &self,
        conversation_id: &str,
        user_code: &str,
    ) -> String {
        let conversation_key = self
            .redis_pool
            .get_conversation_user_key(user_code, conversation_id)
            .await;

        match conversation_key.as_ref() {
            "None" => {
                let other_user_code = self
                    .db
                    .check_conversation_valid(user_code, conversation_id)
                    .await;

                self.redis_pool
                    .set_conversation_user_key(user_code, conversation_id, &other_user_code.clone())
                    .await;
                other_user_code
            }
            _ => conversation_key,
        }
    }

    pub async fn new_conversation(&self, user_code: &str, other_user_code: &str) -> Result<()> {
        let _ = self.db.new_conversation(user_code, other_user_code).await;
        Ok(())
    }

    pub async fn set_last_message(&self, conversation_id: &str, message: &Message) -> Result<()> {
        let _ = self.db.set_last_message(conversation_id, message).await;
        Ok(())
    }
}
