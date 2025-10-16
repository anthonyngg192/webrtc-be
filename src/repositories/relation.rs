use super::AbstractRelation;
use crate::{
    infra::mongodb::MongoDb,
    models::relation::{NewRelation, Relation},
    utils::result::{Error, Result},
};
use mongodb::{bson::doc, options::IndexOptions, Database, IndexModel};
pub static COL: &str = "relations";

#[async_trait]
impl AbstractRelation for MongoDb {
    async fn new_relation(&self, user_code: &str, payload: &NewRelation) -> Result<bool> {
        let new_relation = Relation {
            code: format!("{}_f_{}", user_code, payload.user_code),
            from_user_code: user_code.to_string(),
            to_user_code: payload.user_code.clone(),
        };

        let result = self.col::<Relation>(COL).insert_one(new_relation).await;
        match result {
            Ok(_) => Ok(true),
            Err(_) => Err(Error::YouAlreadyFollowedUser),
        }
    }
}

pub async fn create_relation_index(db: Database) {
    let index_model = IndexModel::builder()
        .keys(doc! { "code": 1, "name": "text" })
        .options(Some(IndexOptions::builder().unique(true).build()))
        .build();

    let _ = db
        .collection::<Relation>(COL)
        .create_index(index_model)
        .await;
}
