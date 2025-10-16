use actix::{Actor, Context};

use crate::services::{conversation_service::ConversationService, message_service::MessageService};
use std::sync::Arc;

#[derive(Clone)]
pub struct Chat {
    pub conversation_service: Arc<ConversationService>,
    pub message_service: Arc<MessageService>,
}

impl Chat {
    pub fn new(
        conversation_service: Arc<ConversationService>,
        message_service: Arc<MessageService>,
    ) -> Self {
        Chat {
            conversation_service,
            message_service,
        }
    }
}

impl Actor for Chat {
    type Context = Context<Self>;
}
