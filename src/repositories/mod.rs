use crate::{
    models::{
        conversation::{Conversation, ConversationQueryResult, FilterConversation, FilterMessage},
        message::{Message, MessageQueryResult, NewMessage},
        relation::NewRelation,
        room::{CreateRoom, FilterRoom, Room, RoomActive, RoomDetail, RoomQueryResult},
        user::{FilterUserQuery, LoginPayload, NewUserPayload, User, UserInfo},
    },
    utils::result::Result,
};

pub mod conversation;
pub mod message;
pub mod relation;
pub mod room;
pub mod user;

#[async_trait]
pub trait AbstractConversation: Sync + Send {
    async fn get_conversation(&self, id: &str) -> Result<Conversation>;
    async fn filter_conversations(
        &self,
        user_code: &str,

        filter: &FilterConversation,
    ) -> Result<ConversationQueryResult>;

    async fn new_conversation(&self, user_code: &str, other_user_code: &str) -> Result<()>;
    async fn check_conversation_valid(&self, user_code: &str, conversation_id: &str) -> String;
    async fn set_last_message(&self, conversation_id: &str, message: &Message) -> Result<()>;
}

#[async_trait]
pub trait AbstractMessage: Sync + Send {
    async fn filter_messages(
        &self,
        conversation_id: &str,
        filter: &FilterMessage,
    ) -> Result<MessageQueryResult>;

    async fn new_message(
        &self,
        conversation_id: &str,
        user_code: &str,
        message: &NewMessage,
    ) -> Result<Message>;
}

#[async_trait]
pub trait AbstractRelation: Sync + Send {
    async fn new_relation(&self, user_code: &str, payload: &NewRelation) -> Result<bool>;
}

#[async_trait]
pub trait AbstractRoom: Sync + Send {
    async fn filter_room(&self, user_code: &str, query: &FilterRoom) -> Result<RoomQueryResult>;
    async fn create_room(&self, user_code: &str, payload: &CreateRoom) -> Option<String>;
    async fn new_participant(&self, room_code: &str, user_code: &str, peer_id: &str) -> bool;
    async fn participant_leave(&self, room_code: &str, user_code: &str, peer_id: &str) -> bool;
    async fn get_room(&self, room_code: &str) -> Option<Room>;
    async fn delete_room(&self, room_code: &str);
    async fn get_room_actives(&self) -> Vec<RoomActive>;
    async fn get_room_info(&self, room_code: &str) -> Option<RoomDetail>;
    async fn peer_leave_room(&self, room_code: String, user_code: String);
    async fn peer_join_room(&self, room_code: String, user_code: String);
    async fn set_room_empty(&self);
}

#[async_trait]
pub trait AbstractUser: Sync + Send {
    async fn user_login(&self, payload: &LoginPayload) -> Result<User>;
    async fn new_user(&self, payload: &NewUserPayload) -> Result<User>;
    async fn filter_users(
        &self,
        payload: &FilterUserQuery,
        user_code: String,
    ) -> Result<Vec<UserInfo>>;

    async fn get_user(&self, user_code: &str) -> Option<UserInfo>;
    async fn update_users_follow(&self, user_code: String, other_user_code: String);
}
pub trait AbstractDatabase:
    Sync
    + Send
    + AbstractConversation
    + AbstractMessage
    + AbstractRelation
    + AbstractRoom
    + AbstractUser
{
}
