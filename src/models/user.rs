use crate::utils::serde_helpers::hex_string_as_string;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    #[serde(
        rename = "_id",
        with = "mongodb::bson::serde_helpers::hex_string_as_object_id"
    )]
    pub id: String,
    pub name: String,
    pub avatar: Option<String>,
    pub code: String,
    pub password: String,
    pub email: String,

    #[serde(rename = "passwordUpdatedAt")]
    pub password_update_at: f64,

    #[serde(rename = "createdAt")]
    pub created_at: f64,

    pub active: Option<bool>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    #[serde(rename = "_id")]
    pub id: String,

    pub name: String,
    pub avatar: Option<String>,
    pub code: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessLogin {
    pub access_token: String,
    pub profile: UserProfile,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUserPayload {
    pub name: String,
    pub password: String,
    pub email: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterUserQuery {
    pub page: i32,
    pub limit: i32,

    #[serde(rename = "fullTextSearch")]
    pub full_text_search: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(rename = "_id", with = "hex_string_as_string")]
    id: String,

    name: String,
    code: String,

    avatar: Option<String>,
    follower: Option<i32>,
    following: Option<i32>,
    metadata: Option<String>,
}
