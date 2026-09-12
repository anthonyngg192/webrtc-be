use crate::{
    actors::{
        room_session::{actor::RoomActor, messages::*},
        session::messages::SendMessage,
    },
    models::{
        peer_session::ParticipantId,
        room_session::{RoomCode, RoomSession},
    },
};
use actix::Addr;
use mediasoup::prelude::*;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    num::{NonZeroU32, NonZeroU8},
    sync::Arc,
};

use super::room_service::RoomService;

#[derive(Clone)]
pub struct NewTransport {
    pub consumer_transport: WebRtcTransport,
    pub producer_transport: WebRtcTransport,
}

impl RoomSession {
    pub async fn new(
        worker: &Worker,
        webrtc_server: Arc<WebRtcServer>,
        room_code: RoomCode,
        owner_code: String,
        service: Arc<RoomService>,
    ) -> Result<Self, String> {
        Self::new_with_id(&worker, webrtc_server, room_code, owner_code, service).await
    }

    async fn new_with_id(
        worker: &Worker,
        webrtc_server: Arc<WebRtcServer>,
        code: RoomCode,
        owner_code: String,
        service: Arc<RoomService>,
    ) -> Result<RoomSession, String> {
        let router = worker
            .create_router(RouterOptions::new(media_codecs()))
            .await
            .map_err(|error| format!("Failed to create router: {error}"))?;

        log::info!("Room {} created", code.0.clone());

        Ok(Self {
            owner_code,
            code: code.clone(),
            router,
            webrtc_server: webrtc_server.clone(),
            consumers: HashMap::new(),
            producers: HashMap::new(),
            peers: HashMap::new(),
            producer_transports: HashMap::new(),
            consumer_transports: HashMap::new(),
            muted_peers: HashMap::new(),
            peer_consumers: HashMap::new(),
            peer_producers: HashMap::new(),
            service,
            plain_transport: None,
        })
    }

    pub async fn new_producer(
        &mut self,
        peer: ParticipantId,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
    ) -> Option<Producer> {
        let transport = self.producer_transports.get(&peer.clone());
        if transport.is_some() {
            let producer = transport
                .unwrap()
                .produce(ProducerOptions::new(kind, rtp_parameters))
                .await;

            match producer {
                Ok(producer) => return Some(producer),
                Err(_) => {
                    log::error!("Failed to create producer");
                    return None;
                }
            }
        }
        None
    }

    pub async fn new_consumer(
        &mut self,
        producer_id: ProducerId,
        peer: ParticipantId,
        client_rtp_capabilities: RtpCapabilities,
    ) -> Option<Consumer> {
        let transport = self.consumer_transports.get(&peer.clone());
        if transport.is_some() {
            let mut options = ConsumerOptions::new(producer_id, client_rtp_capabilities.clone());
            options.enable_rtx = Some(true);
            options.ignore_dtx = true;
            options.paused = true;
            let consumer = transport.unwrap().consume(options).await;
            let result = match consumer {
                Ok(consumer) => {
                    let _ = consumer.resume().await;
                    let peer_addr = self.peers.get(&peer.clone());
                    if let Some(addr) = peer_addr {
                        let data_to_peer = CreateConsumerRespond {
                            event_name: "CreateConsumerRespond".to_string(),
                            data: CreateConsumerResult {
                                producer_id,
                                id: consumer.id(),
                                kind: consumer.kind(),
                                rtp_parameters: consumer.rtp_parameters().clone(),
                                r#type: consumer.r#type(),
                                producer_paused: consumer.producer_paused(),
                            },
                        };

                        addr.do_send(SendMessage {
                            content: serde_json::to_vec(&data_to_peer).unwrap(),
                        });
                    }

                    consumer
                }
                Err(_) => {
                    log::error!("Failed to create consumer");
                    return None;
                }
            };
            return Some(result);
        }
        None
    }

    pub async fn new_transport(&mut self, peer: ParticipantId, room_addr: Addr<RoomActor>) {
        let webrtc_server = self.webrtc_server.as_ref().clone();
        let transport_options = WebRtcTransportOptions::new_with_server(webrtc_server);

        let producer_transport = self
            .router
            .create_webrtc_transport(transport_options.clone())
            .await
            .map_err(|error| format!("Failed to create producer transport: {error}"))
            .unwrap();

        let consumer_transport = self
            .router
            .create_webrtc_transport(transport_options)
            .await
            .map_err(|error| format!("Failed to create consumer transport: {error}"))
            .unwrap();

        let consumer_trans = Arc::new(consumer_transport);
        let transport_cons_clone = Arc::clone(&consumer_trans);

        let produce_trans = Arc::new(producer_transport);
        let transport_prod_clone = Arc::clone(&produce_trans);

        let data = CreateTransportRespond {
            event_name: "CreateTransportRespond".to_string(),
            producer: {
                let transport = transport_prod_clone.clone();
                ProducerTransport {
                    id: transport.id().to_string(),
                    ice_parameters: transport.ice_parameters().clone(),
                    ice_candidates: transport.ice_candidates().clone(),
                    dtls_parameters: transport.dtls_parameters().clone(),
                    sctp_parameters: transport.sctp_parameters(),
                }
            },
            consumer: {
                let transport = transport_cons_clone.clone();
                ConsumerTransport {
                    id: transport.id().to_string(),
                    ice_parameters: transport.ice_parameters().clone(),
                    ice_candidates: transport.ice_candidates().clone(),
                    dtls_parameters: transport.dtls_parameters().clone(),
                    sctp_parameters: transport.sctp_parameters(),
                }
            },
        };
        let peer_addr = self.peers.get(&peer.clone());
        if let Some(addr) = peer_addr {
            addr.do_send(SendMessage {
                content: serde_json::to_vec(&data).unwrap(),
            });
        }

        room_addr.do_send(AddTransports {
            producer_transport: transport_prod_clone,
            consumer_transport: transport_cons_clone,
            peer,
        });
    }

    pub async fn connect_produce_transport(
        &self,
        peer: ParticipantId,
        dtls_parameters: DtlsParameters,
    ) {
        let transport = self.producer_transports.get(&peer.clone());
        match transport {
            Some(producer) => {
                let _ = producer
                    .connect(WebRtcTransportRemoteParameters { dtls_parameters })
                    .await;
            }
            None => {
                println!("transport not found");
            }
        };
    }

    pub async fn connect_consume_transport(
        &self,
        peer: ParticipantId,
        dtls_parameters: DtlsParameters,
    ) {
        let transport = self.consumer_transports.get(&peer.clone());
        match transport {
            Some(producer) => {
                let _ = producer
                    .connect(WebRtcTransportRemoteParameters { dtls_parameters })
                    .await;
            }
            None => {
                println!("transport not found");
            }
        };
    }

    //TODO;
    pub fn peer_leave_room(&mut self, peer: ParticipantId) {
        self.peers.remove(&peer);

        let _ = self.producer_transports.remove(&peer);
        let _ = self.consumer_transports.remove(&peer);

        self.muted_peers.remove(&peer);

        let producer = self.producers.remove(&peer);
        if producer.is_some() {
            let producer = producer.unwrap();
            let producer_ids = producer.into_iter().collect::<Vec<ProducerId>>();

            for producer_id in producer_ids {
                let _ = self.peer_producers.remove(&producer_id);
            }
        }
        let consumers = self.consumers.remove(&peer);
        if consumers.is_some() {
            let consumer_ids = consumers.unwrap().into_iter().collect::<Vec<ConsumerId>>();
            for consumer_id in consumer_ids {
                let _ = self.peer_consumers.remove(&consumer_id);
            }
        }
    }

    pub async fn pause_or_resume_producer(
        &self,
        participant_id: ParticipantId,
        producer_id: ProducerId,
    ) -> ActionResult {
        let producers = self.producers.get(&participant_id);
        if let Some(producers) = producers {
            let is_owner_producer = producers.contains(&producer_id);
            if !is_owner_producer {
                log::error!("Producer invalid");
                return ActionResult::Failed;
            }

            let producer = self.peer_producers.get(&producer_id).unwrap();
            match producer.paused() {
                true => {
                    let _ = producer.resume().await;
                }
                _ => {
                    let _ = producer.pause().await;
                }
            };

            return ActionResult::Success;
        }
        ActionResult::Failed
    }

    //TODO: enable srtp - security realtime transport protocol.
    pub async fn create_plain_transport(&mut self, room_addr: Addr<RoomActor>) {
        let plain_trans = self
            .router
            .create_plain_transport(PlainTransportOptions::new(ListenInfo {
                protocol: Protocol::Udp,
                ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
                announced_address: Some("0.0.0.0".to_string()),
                port: None,
                port_range: None,
                flags: None,
                send_buffer_size: None,
                recv_buffer_size: None,
            }))
            .await
            .expect("error in case create plain transport");

        //TODO. Remove.
        // let remote_ip = "127.0.0.1".to_string();
        // let remote_port = 5004;

        match plain_trans
            .connect(PlainTransportRemoteParameters {
                ip: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                port: Some(10001),
                rtcp_port: None,
                srtp_parameters: None,
            })
            .await
        {
            Ok(_) => println!("Connected successfully!"),
            Err(e) => eprintln!("Connect failed: {}", e),
        };
        let arc_plain = Arc::new(plain_trans);
        let arc_plain_clone = Arc::clone(&arc_plain);
        room_addr.do_send(AddPlainTransport {
            plain_trans: arc_plain_clone.clone(),
        });
    }

    pub async fn create_producer_plain_transport(
        &mut self,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
    ) -> Option<Producer> {
        if kind == MediaKind::Video {
            let transport = &self.plain_transport;
            if let Some(plain_transport) = transport {
                let producer = plain_transport
                    .produce(ProducerOptions::new(kind, rtp_parameters))
                    .await;

                match producer {
                    Ok(producer) => return Some(producer),
                    Err(_) => {
                        log::error!("Failed to create producer");
                        return None;
                    }
                }
            }
            return None;
        }
        None
    }
}

pub fn media_codecs() -> Vec<RtpCodecCapability> {
    vec![
        RtpCodecCapability::Audio {
            mime_type: MimeTypeAudio::Opus,
            preferred_payload_type: None,
            clock_rate: NonZeroU32::new(48000).unwrap(),
            channels: NonZeroU8::new(2).unwrap(),
            parameters: RtpCodecParametersParameters::from([("useinbandfec", 1_u32.into())]),
            rtcp_feedback: vec![RtcpFeedback::TransportCc],
        },
        RtpCodecCapability::Video {
            mime_type: MimeTypeVideo::Vp8,
            preferred_payload_type: None,
            clock_rate: NonZeroU32::new(90000).unwrap(),
            parameters: RtpCodecParametersParameters::default(),
            rtcp_feedback: vec![
                RtcpFeedback::Nack,
                RtcpFeedback::NackPli,
                RtcpFeedback::CcmFir,
                RtcpFeedback::GoogRemb,
                RtcpFeedback::TransportCc,
            ],
        },
    ]
}

pub enum ActionResult {
    Success,
    Failed,
}
