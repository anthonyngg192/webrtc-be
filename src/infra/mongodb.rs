use crate::{
    repositories::AbstractDatabase,
    utils::{
        environment::{DATABASE_NAME, DB_CONNECTION_STRING},
        result::{Error, Result},
    },
};
use mongodb::{bson::Document, results::InsertOneResult};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, ops::Deref};

pub async fn init_mongodb() -> Result<MongoDb> {
    let client = mongodb::Client::with_uri_str(DB_CONNECTION_STRING.clone())
        .await
        .map_err(|_err| "Failed to connection to database".to_string());

    match client {
        Err(_) => Err(Error::DatabaseError {
            operation: "init_mongodb",
            with: "mongodb",
        }),
        Ok(cli) => Ok(MongoDb(cli, DATABASE_NAME.to_owned())),
    }
}

impl AbstractDatabase for MongoDb {}

#[derive(Clone)]
pub struct MongoDb(pub ::mongodb::Client, pub String);

impl Deref for MongoDb {
    type Target = mongodb::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MongoDb {
    pub fn db(&self) -> mongodb::Database {
        self.database(&DATABASE_NAME)
    }

    pub fn col<T>(&self, collection: &str) -> mongodb::Collection<T>
    where
        T: Send + Sync,
    {
        self.db().collection(collection)
    }

    pub async fn insert_one<T>(
        &self,
        collection: &'static str,
        document: T,
    ) -> Result<InsertOneResult>
    where
        T: Send + Sync + Serialize,
    {
        self.col::<T>(collection)
            .insert_one(document)
            .await
            .map_err(|err| {
                println!("{}", err.clone());
                Error::DatabaseError {
                    operation: "insert_one",
                    with: collection,
                }
            })
    }

    pub async fn find_one<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<T> {
        self.find_one_with_options(collection, projection).await
    }

    pub async fn find_one_with_options<T: DeserializeOwned + Unpin + Send + Sync>(
        &self,
        collection: &'static str,
        projection: Document,
    ) -> Result<T> {
        self.col::<T>(collection)
            .find_one(projection)
            .await
            .map_err(|_| Error::DatabaseError {
                operation: "find_one",
                with: collection,
            })?
            .ok_or(Error::NotFound)
    }
}

#[derive(Deserialize)]
pub struct DocumentId {
    #[serde(rename = "_id")]
    pub id: String,
}

pub trait IntoDocumentPath: Send + Sync {
    fn as_path(&self) -> Option<&'static str>;
}

pub fn prefix_key<T: Serialize>(t: &T, prefix: &str) -> HashMap<String, serde_json::Value> {
    let v: String = serde_json::to_string(t).unwrap();
    let v: HashMap<String, serde_json::Value> = serde_json::from_str(&v).unwrap();

    v.into_iter()
        .filter(|(_k, v)| !v.is_null())
        .map(|(k, v)| (format!("{}{}", prefix.to_owned(), k), v))
        .collect()
}
