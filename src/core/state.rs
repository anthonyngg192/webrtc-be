use std::sync::Arc;

use actix::Addr;

use crate::{
    actors::chat::actor::ChatActor,
    adapters::redis_adapter::RedisAdapter,
    services::{
        auth_service::AuthService, conversation_service::ConversationService,
        message_service::MessageService, relation_service::RelationService,
        room_service::RoomService, user_service::UserService,
    },
};

pub struct AppState {
    pub redis_client: Arc<RedisAdapter>,
    pub conversation_service: Arc<ConversationService>,
    pub relation_service: Arc<RelationService>,
    pub message_service: Arc<MessageService>,
    pub room_service: Arc<RoomService>,
    pub auth_service: Arc<AuthService>,
    pub user_service: Arc<UserService>,
    pub chat_addr: Arc<Addr<ChatActor>>,
}

impl AppState {
    #[warn(clippy::too_many_arguments)]
    pub fn new(
        redis_client: Arc<RedisAdapter>,
        conversation_service: Arc<ConversationService>,
        message_service: Arc<MessageService>,
        relation_service: Arc<RelationService>,
        room_service: Arc<RoomService>,
        user_service: Arc<UserService>,
        auth_service: Arc<AuthService>,
        chat_addr: Arc<Addr<ChatActor>>,
    ) -> Self {
        Self {
            redis_client,
            conversation_service,
            message_service,
            relation_service,
            room_service,
            auth_service,
            user_service,
            chat_addr,
        }
    }
}
pub type SharedState = Arc<AppState>;
