use std::time::Instant;

use crate::{
    actors::{
        chat::messages::NewMessage,
        peer::messages::PeerJoinRoom,
        room_session::messages::{
            ConnectConsumerTransport, ConnectProducerTransport, CreateConsumer, CreateProducer,
            CreateTransport, MessageType, NewRoomMessage, PeerLeaveRoomSession, RoomCloseProducer,
            RoomPauserResumeProducer,
        },
    },
    services::peer_service::PeerService,
};
use actix::{ActorContext, AsyncContext};
use actix::{Handler, StreamHandler, SystemService};
use actix_web_actors::ws;

use super::{
    actor::Session,
    messages::{AssignRoom, ClientEvent, SendMessage, SessionDisconnect},
};

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.hb = Instant::now();
        match item {
            Ok(ws::Message::Text(text)) => match serde_json::from_str::<ClientEvent>(&text) {
                Ok(event) => match event {
                    ClientEvent::NewMessage { data } => {
                        self.chat_addr.do_send(NewMessage {
                            content: data.content,
                            gif: data.gif,
                            conversation_id: data.conversation_id,
                            user_code: self.participant_id.user_code.clone(),
                        });
                    }
                    ClientEvent::JoinRoomRequest { data } => {
                        PeerService::from_registry().do_send(PeerJoinRoom {
                            room_code: data.room_code.clone(),
                            session: self.clone(),
                            addr: ctx.address(),
                        });
                    }
                    ClientEvent::CreateTransportRequest => {
                        let room = self.room_addr.clone();
                        let participant_id = self.participant_id.clone();
                        match room {
                            Some(room) => room.do_send(CreateTransport {
                                participant: participant_id.clone(),
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }
                    ClientEvent::ConnectConsumerTransportRequest { data } => {
                        let room = self.room_addr.clone();
                        let participant_id = self.participant_id.clone();
                        match room {
                            Some(room) => room.do_send(ConnectConsumerTransport {
                                participant: participant_id.clone(),
                                dtls_parameters: data,
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }
                    ClientEvent::ConnectProducerTransportRequest { data } => {
                        let room = self.room_addr.clone();
                        let participant_id = self.participant_id.clone();
                        match room {
                            Some(room) => room.do_send(ConnectProducerTransport {
                                participant: participant_id.clone(),
                                dtls_parameters: data,
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }
                    ClientEvent::NewConsumerRequest { data } => {
                        let room = self.room_addr.clone();
                        let participant_id = self.participant_id.clone();
                        let rtp_capabilities = data.rtp_capabilities.clone();
                        match room {
                            Some(room) => room.do_send(CreateConsumer {
                                participant: participant_id.clone(),
                                rtp_capabilities,
                                producer_id: data.producer_id,
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }
                    ClientEvent::NewProducerRequest { data } => {
                        let room = self.room_addr.clone();
                        let participant_id = self.participant_id.clone();
                        match room {
                            Some(room) => room.do_send(CreateProducer {
                                participant: participant_id.clone(),
                                kind: data.kind,
                                rtp_parameter: data.rtp_parameters,
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }
                    ClientEvent::PauseProducer { data } => {
                        let room = self.room_addr.clone();
                        match room {
                            Some(room) => room.do_send(RoomPauserResumeProducer {
                                producer_id: data.producer_id,
                                peer: self.participant_id.clone(),
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }
                    ClientEvent::ResumeProducer { data } => {
                        let room = self.room_addr.clone();
                        match room {
                            Some(room) => room.do_send(RoomPauserResumeProducer {
                                producer_id: data.producer_id,
                                peer: self.participant_id.clone(),
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }

                    ClientEvent::CloseProducer { data } => {
                        let room = self.room_addr.clone();
                        match room {
                            Some(room) => room.do_send(RoomCloseProducer {
                                producer_id: data.producer_id,
                                peer: self.participant_id.clone(),
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }

                    ClientEvent::NewRoomMessage { data } => {
                        let room = self.room_addr.clone();
                        match room {
                            Some(room) => room.do_send(NewRoomMessage {
                                text: data.text,
                                gif: data.gif,
                                participant_id: Some(self.participant_id.clone()),
                                r#type: MessageType::Message,
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }

                    ClientEvent::BannedUserOutRoom { data } => {
                        let room = self.room_addr.clone();
                        match room {
                            Some(room) => room.do_send(
                                crate::actors::room_session::messages::BannedUserOutRoom {
                                    user_code: data.user_code,
                                    participant_id: self.participant_id.clone(),
                                },
                            ),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }

                    ClientEvent::PeerLeaveRoom => {
                        let room = self.room_addr.clone();
                        match room {
                            Some(room) => room.do_send(PeerLeaveRoomSession {
                                participant_id: self.participant_id.clone(),
                                name: self.name.clone(),
                            }),
                            None => {
                                log::error!("You are not in a room");
                            }
                        }
                    }
                    ClientEvent::Ping => {
                        ctx.text("Pong");
                    }
                },
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}", e);
                    ctx.text(r#"{"error":"Invalid JSON format"}"#);
                }
            },
            Ok(ws::Message::Binary(bin)) => {
                println!("Received binary data: {:?}", bin);
            }
            Ok(ws::Message::Close(reason)) => {
                println!("Client disconnected: {:?}", reason);
                ctx.stop();
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(msg)) => {
                log::info!("Received Pong: {:?}", msg);
            }
            _ => {}
        }
    }
}

impl Handler<SendMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.content.clone());
    }
}

impl Handler<AssignRoom> for Session {
    type Result = ();

    fn handle(&mut self, msg: AssignRoom, _: &mut Self::Context) -> Self::Result {
        self.room_addr = Some(msg.addr);
    }
}

impl Handler<SessionDisconnect> for Session {
    type Result = ();

    fn handle(&mut self, _: SessionDisconnect, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}
