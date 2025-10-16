use actix::Message;
use serde::{Deserialize, Serialize};

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct NewMessage {
    pub conversation_id: String,
    pub content: Option<String>,
    pub gif: Option<String>,
    pub user_code: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageEvent {
    pub conversation_id: String,
    pub content: Option<String>,
    pub gif: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageEventData {
    pub conversation_id: String,
    pub content: Option<String>,
    pub gif: Option<String>,
    pub direct_from: String,
    pub r#type: String,

    #[serde(rename = "_id")]
    pub id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessageRespond {
    pub event_name: String,
    pub data: NewMessageEventData,
}
