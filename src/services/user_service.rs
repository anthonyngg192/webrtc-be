use std::sync::Arc;

use crate::{
    infra::mongodb::MongoDb,
    models::user::{FilterUserQuery, UserInfo},
    repositories::AbstractUser,
    utils::result::{Error, Result},
};

pub struct UserService {
    pub db: Arc<MongoDb>,
}

impl UserService {
    pub fn new(db: Arc<MongoDb>) -> Self {
        Self { db }
    }

    pub async fn filter_users(
        &self,
        user_code: String,
        query: &FilterUserQuery,
    ) -> Result<Vec<UserInfo>> {
        self.db.filter_users(query, user_code).await
    }

    pub async fn get_user(&self, user_code: &str) -> Result<UserInfo> {
        let res = self.db.get_user(user_code).await;
        match res {
            Some(user) => Ok(user),
            None => Err(Error::UserNotFound),
        }
    }
}
