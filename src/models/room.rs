use mongodb::bson::serde_helpers::hex_string_as_object_id;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    #[serde(rename = "_id", with = "hex_string_as_object_id")]
    pub id: String,

    #[serde(rename = "ownerCode")]
    pub owner_code: String,

    pub blacklist: Vec<String>,

    pub participants: Vec<String>,

    pub code: String,

    pub r#type: String,

    #[serde(rename = "expiredAt")]
    pub expired_at: Option<i64>,

    #[serde(rename = "displayName")]
    pub display_name: String,

    pub limit: i8,

    #[serde(rename = "roomStart")]
    pub room_start: Option<i64>,

    #[serde(rename = "activeParticipants")]
    pub active_participants: i8,

    #[serde(rename = "createdAt")]
    pub created_at: i64,

    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub code: String,
    pub name: String,
    pub avatar: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FilterRoom {
    pub page: i32,
    pub limit: i32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoom {
    pub room_code: Option<String>,
    pub display_name: String,
    pub limit: i8,
    pub category: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RoomDetail {
    #[serde(rename = "ownerCode")]
    pub owner_code: String,

    pub blacklist: Vec<String>,

    pub users: Vec<Participant>,
    pub code: String,

    pub r#type: String,

    #[serde(rename = "expiredAt")]
    pub expired_at: Option<i64>,

    #[serde(rename = "displayName")]
    pub display_name: String,

    pub limit: i8,

    #[serde(rename = "roomStart")]
    pub room_start: Option<i64>,

    #[serde(rename = "activeParticipants")]
    pub active_participants: i8,

    pub owner: Option<OwnerDetail>,

    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OwnerDetail {
    name: String,
    code: String,
    avatar: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct RoomQueryResult {
    pub data: Vec<RoomDetail>,
    pub total: i32,
    pub page: i32,
}

#[derive(Deserialize, Serialize)]
pub struct RoomInfo {
    pub code: String,

    #[serde(rename = "ownerCode")]
    pub owner_code: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RoomActive {
    pub code: String,

    #[serde(rename = "ownerCode")]
    pub owner_code: String,
}
