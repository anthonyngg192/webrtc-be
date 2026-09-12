use crate::{
    actors::peer::{
        actor::PeerActor,
        messages::{ErrorEventByUser, SendPeersByUser},
    },
    models::message::NewMessage as NewMessageModel,
};

use super::{
    actor::ChatActor,
    messages::{NewMessage, NewMessageEventData, NewMessageRespond},
};
use actix::{Handler, SystemService};

impl Handler<NewMessage> for ChatActor {
    type Result = ();

    fn handle(&mut self, msg: NewMessage, _: &mut Self::Context) {
        let message_service = self.message_service.clone();
        let conversation_service = self.conversation_service.clone();
        actix::spawn(async move {
            let other_user_code = conversation_service
                .get_other_user_in_conversation(&msg.conversation_id, &msg.user_code.clone())
                .await;
            if other_user_code != "None" {
                let message = message_service
                    .new_message(
                        &msg.conversation_id.clone(),
                        &msg.user_code.clone(),
                        &NewMessageModel {
                            content: msg.content.clone(),
                            gif: msg.gif.clone(),
                        },
                    )
                    .await;

                match message {
                    Ok(new_message) => {
                        PeerActor::from_registry().do_send(SendPeersByUser {
                            user_codes: vec![other_user_code.clone(), msg.user_code.clone()],
                            data: serde_json::to_vec(&NewMessageRespond {
                                event_name: "NewMessageRespond".to_string(),
                                data: NewMessageEventData {
                                    conversation_id: msg.conversation_id.clone(),
                                    content: msg.content,
                                    gif: msg.gif,
                                    r#type: "Message".to_string(),
                                    direct_from: msg.user_code.clone(),
                                    id: new_message.id.to_string(),
                                },
                            })
                            .unwrap(),
                        });
                    }
                    Err(_) => {
                        log::error!("some thing when wrong when create new message");
                    }
                };
            } else {
                PeerActor::from_registry().do_send(ErrorEventByUser {
                    data: "Conversation not found".to_string(),
                    user_code: msg.user_code,
                    event_name: "ErrorMessage".to_string(),
                });
            }
        });
    }
}
