use std::sync::Arc;

use crate::{
    infra::mongodb::MongoDb,
    models::relation::NewRelation,
    repositories::{AbstractConversation, AbstractRelation, AbstractUser},
    utils::result::Result,
};

pub struct RelationService {
    pub db: Arc<MongoDb>,
}

impl RelationService {
    pub fn new(db: Arc<MongoDb>) -> Self {
        Self { db }
    }

    pub async fn new_relation(&self, user_code: &str, payload: &NewRelation) -> Result<bool> {
        match self.db.new_relation(user_code, payload).await {
            Ok(res) => {
                let _ = self
                    .db
                    .update_users_follow(user_code.to_string(), payload.user_code.clone())
                    .await;

                let _ = self
                    .db
                    .new_conversation(user_code, &payload.user_code)
                    .await;
                Ok(res)
            }
            Err(res) => Err(res),
        }
    }
}
