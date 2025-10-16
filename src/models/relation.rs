use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Relation {
    pub code: String,

    #[serde(rename = "fromUserCode")]
    pub from_user_code: String,

    #[serde(rename = "toUserCode")]
    pub to_user_code: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NewRelation {
    #[serde(rename = "userCode")]
    pub user_code: String,
}

// pub struct Relation
