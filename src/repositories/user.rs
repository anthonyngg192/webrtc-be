use crate::{
    infra::mongodb::MongoDb,
    models::user::{FilterUserQuery, LoginPayload, NewUserPayload, User, UserInfo},
    utils::{
        generate::generate_string_size,
        result::{Error, Result},
        HASH_ROUND,
    },
};

use super::AbstractUser;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    Database, IndexModel,
};

pub static COL: &str = "users";
#[async_trait]
impl AbstractUser for MongoDb {
    async fn user_login(&self, payload: &LoginPayload) -> Result<User> {
        let is_user_existed = self
            .col::<User>(COL)
            .find_one(doc! {
              "email":payload.email.to_string()
            })
            .await
            .unwrap();

        let user: User = match is_user_existed {
            Some(u) => u,
            None => return Err(Error::EmailNotFound),
        };

        if !bcrypt::verify(payload.password.clone(), &user.password).unwrap() {
            return Err(Error::InvalidPassword);
        }

        Ok(user)
    }

    async fn new_user(&self, payload: &NewUserPayload) -> Result<User> {
        let email_existed = self
            .col::<User>(COL)
            .find_one(doc! {
              "email":payload.email.to_string()
            })
            .await
            .unwrap();

        let code = generate_string_size(10);
        let now: DateTime<Utc> = Utc::now();
        let user = User {
            id: ObjectId::new().to_string(),
            name: payload.name.to_string(),
            email: payload.email.to_string(),
            password: bcrypt::hash(&payload.password, *HASH_ROUND).unwrap(),
            avatar: None,
            code,
            active: Some(true),
            password_update_at: 0.0,
            created_at: now.timestamp() as f64,
        };

        match email_existed {
            Some(_) => {
                return Err(Error::EmailAlreadyExisted);
            }
            None => {
                let result = self.insert_one(COL, user.clone()).await;
                match result {
                    Ok(_data_result) => Ok(user),
                    Err(_err) => Err(Error::EmailAlreadyExisted),
                }
            }
        }
    }

    async fn filter_users(
        &self,
        payload: &FilterUserQuery,
        user_code: String,
    ) -> Result<Vec<UserInfo>> {
        let match_pipe = match &payload.full_text_search {
            Some(search) => doc! {
                "$match": {
                    "$text": { "$search": search },
                    "blacklist":{"$nin": [user_code.clone()] },
                    "code": { "$ne": user_code.clone() }
                }
            },
            None => doc! {
                "$match": {
                    "blacklist":{"$nin": [user_code.clone()] },
                    "code": { "$ne": user_code.clone() }
                }
            },
        };

        let users: Vec<UserInfo> = self
            .col::<UserInfo>(COL)
            .aggregate([
                match_pipe,
                doc! {
                    "$lookup": {
                        "from": "relations",
                        "localField": "code",
                        "foreignField": "toUserCode",
                        "as": "follow_info"
                    }
                },
                doc! {
                    "$match": {
                        "follow_info.fromUserCode": { "$ne": user_code }
                    }
                },
                doc! {
                    "$project": {
                      "_id": 1,
                      "name": 1,
                      "code": 1,
                      "avatar": 1,
                      "follower": 1,
                      "following": 1,
                      "metadata": 1,
                    }
                },
                doc! {
                    "$skip": (payload.page - 1) * payload.limit
                },
                doc! {
                    "$limit": payload.limit
                },
            ])
            .await
            .expect("Unable to aggregate posts")
            .with_type()
            .try_collect()
            .await
            .expect("Unable to collect items from Cursor");
        Ok(users)
    }

    async fn get_user(&self, user_code: &str) -> Option<UserInfo> {
        let users: Vec<UserInfo> = self
            .col::<UserInfo>(COL)
            .aggregate([
                doc! {
                    "$match": {
                        "code": user_code
                    }
                },
                doc! {
                    "$project": {
                      "_id": 1,
                      "name": 1,
                      "code": 1,
                      "avatar": 1,
                      "follower": 1,
                      "following": 1,
                      "metadata": 1,
                    }
                },
                doc! {
                    "$skip": 0
                },
                doc! {
                    "$limit":1
                },
            ])
            .await
            .expect("Unable to aggregate posts")
            .with_type()
            .try_collect()
            .await
            .expect("Unable to collect items from Cursor");

        users.into_iter().next()
    }

    async fn update_users_follow(&self, user_code: String, other_user_code: String) {
        let _ = self
            .col::<User>(COL)
            .update_one(
                doc! {
                    "code": user_code
                },
                doc! {
                    "$inc": {
                        "following":1
                    }
                },
            )
            .await;

        let _ = self
            .col::<User>(COL)
            .update_one(
                doc! {
                    "code": other_user_code
                },
                doc! {
                    "$inc": {
                        "follower": 1
                    }
                },
            )
            .await;
    }
}

pub async fn create_user_index(db: Database) {
    let index_model = IndexModel::builder()
        .keys(doc! { "name": 1 })
        .options(Some(IndexOptions::builder().unique(true).build()))
        .build();

    let _ = db.collection::<User>(COL).create_index(index_model).await;
}
