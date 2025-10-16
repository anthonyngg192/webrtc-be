use super::message::Message;
use crate::utils::serde_helpers::hex_string_as_string;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    #[serde(rename = "_id")]
    pub id: ObjectId,

    #[serde(rename = "conversationMembersCodes")]
    pub conversation_members_codes: Vec<String>,

    #[serde(rename = "lastMessage")]
    pub last_message: Option<Message>,

    #[serde(rename = "createdAt")]
    pub created_at: f64,

    #[serde(rename = "updatedAt")]
    pub updated_at: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FilterConversation {
    pub page: i32,
    pub limit: i32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum ConversationUserKey {
    None,
    Friend,
    UnKnow,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FilterMessage {
    pub page: i32,
    pub limit: i32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ConversationQueryResult {
    pub data: Vec<ConversationDetail>,
    pub total: i32,
    pub page: i32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ConversationDetail {
    #[serde(rename = "_id", with = "hex_string_as_string")]
    pub id: String,

    #[serde(rename = "conversationMembersCodes")]
    pub conversation_members_codes: Vec<String>,

    #[serde(rename = "lastMessage")]
    pub last_message: Option<Message>,

    #[serde(rename = "userDetails")]
    pub user_details: Vec<Participant>,

    pub user: Option<Participant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub code: String,
    pub name: String,
    pub avatar: Option<String>,
}
