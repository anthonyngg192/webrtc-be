use actix::{Actor, Context};

use crate::services::{conversation_service::ConversationService, message_service::MessageService};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChatActor {
    pub conversation_service: Arc<ConversationService>,
    pub message_service: Arc<MessageService>,
}

impl ChatActor {
    pub fn new(
        conversation_service: Arc<ConversationService>,
        message_service: Arc<MessageService>,
    ) -> Self {
        ChatActor {
            conversation_service,
            message_service,
        }
    }
}

impl Actor for ChatActor {
    type Context = Context<Self>;
}
