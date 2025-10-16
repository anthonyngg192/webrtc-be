use super::conversation::Participant;
use crate::utils::serde_helpers::hex_string_as_string;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: ObjectId,

    #[serde(rename = "conversationId")]
    pub conversation_id: ObjectId,

    pub content: Option<String>,

    pub gif: Option<String>,

    pub images: Vec<String>,

    #[serde(rename = "ownerCode")]
    pub owner_code: Option<String>,

    #[serde(rename = "ownerType")]
    pub owner_type: MessageOwnerType,

    #[serde(rename = "type")]
    pub message_type: MessageType,

    pub metadata: Option<MetadataMessage>,

    #[serde(rename = "createdAt")]
    pub created_at: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageOwnerType {
    User,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataMessage {
    #[serde(rename = "callStart")]
    pub call_start: Option<f64>,

    #[serde(rename = "callEnd")]
    pub call_end: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Notification,
    Message,
    CompleteCall,
    MissCall,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewMessage {
    pub content: Option<String>,
    pub gif: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MessageDetail {
    #[serde(rename = "_id", with = "hex_string_as_string")]
    pub id: String,

    #[serde(rename = "conversationId", with = "hex_string_as_string")]
    pub conversation_id: String,

    pub content: Option<String>,

    pub gif: Option<String>,

    pub images: Option<Vec<String>>,

    #[serde(rename = "ownerCode")]
    pub owner_code: Option<String>,

    #[serde(rename = "ownerType")]
    pub owner_type: MessageOwnerType,

    #[serde(rename = "type")]
    pub message_type: MessageType,

    pub metadata: Option<MetadataMessage>,

    pub owner: Option<Participant>,

    #[serde(rename = "createdAt")]
    pub created_at: Option<f64>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MessageQueryResult {
    pub data: Vec<MessageDetail>,
    pub total: i32,
    pub page: i32,
}
