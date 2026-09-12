use std::{net::IpAddr, sync::Arc};

use crate::{
    actors::{
        chat::actor::ChatActor,
        room_management::{
            actors::RoomsActor,
            messages::{AddWorker, RoomManagementCreateRoom},
        },
    },
    adapters, infra,
    models::room_session::RoomCode,
    repositories::{relation::create_relation_index, user::create_user_index},
    services::{
        auth_service::AuthService, conversation_service::ConversationService,
        message_service::MessageService, relation_service::RelationService,
        room_service::RoomService, user_service::UserService,
    },
    utils::{MEDIASOUP_ANNOUNCED_IP, MEDIASOUP_LISTEN_IP, NUM_WORKER},
};
use actix::{Actor, SystemService};
use mediasoup::{
    prelude::{ListenInfo, Protocol, WebRtcServerListenInfos, WebRtcServerOptions, WorkerManager},
    worker::{WorkerLogLevel, WorkerLogTag, WorkerSettings},
};

use super::state::AppState;

async fn preload_room_active(room_service: Arc<RoomService>) {
    let _ = room_service.pre_reload_room_to_ram().await;
    let rooms = room_service.get_room_actives().await;
    for room in rooms {
        let room_code = RoomCode(room.code);
        RoomsActor::from_registry().do_send(RoomManagementCreateRoom {
            room_code,
            owner_code: room.owner_code,
            room_service: room_service.clone(),
        });
    }
    log::info!("Room initialize Completed");
}

async fn init_worker() {
    let worker_manager = WorkerManager::new();
    for i in 0..*NUM_WORKER {
        let worker_manager = worker_manager.clone();
        actix::spawn(async move {
            let worker = worker_manager
                .create_worker({
                    let mut settings = WorkerSettings::default();
                    settings.log_level = WorkerLogLevel::Error;
                    settings.log_tags = vec![
                        WorkerLogTag::Info,
                        WorkerLogTag::Ice,
                        WorkerLogTag::Dtls,
                        WorkerLogTag::Rtp,
                        WorkerLogTag::Srtp,
                        WorkerLogTag::Rtcp,
                        WorkerLogTag::Rtx,
                        WorkerLogTag::Bwe,
                        WorkerLogTag::Score,
                        WorkerLogTag::Simulcast,
                        WorkerLogTag::Svc,
                        WorkerLogTag::Sctp,
                        WorkerLogTag::Message,
                    ];
                    settings
                })
                .await
                .unwrap();

            let ip: IpAddr = MEDIASOUP_LISTEN_IP.parse().expect("IP Invalid");

            let listen_infos = ListenInfo {
                protocol: Protocol::Udp,
                ip,
                announced_address: Some(MEDIASOUP_ANNOUNCED_IP.to_string()),
                port: None,
                port_range: Some(10000..=10100),
                flags: None,
                send_buffer_size: None,
                recv_buffer_size: None,
            };

            let webrtc_server = worker
                .create_webrtc_server(WebRtcServerOptions::new(WebRtcServerListenInfos::new(
                    listen_infos,
                )))
                .await
                .unwrap();

            RoomsActor::from_registry().do_send(AddWorker {
                id: i,
                worker,
                webrtc_server,
            });
        });
    }
}

pub async fn bootstrap() -> AppState {
    let client = Arc::new(infra::mongodb::init_mongodb().await.unwrap());
    let redis_client = Arc::new(adapters::redis_adapter::RedisAdapter::new());

    let conversation_service = Arc::new(ConversationService::new(
        client.clone(),
        Arc::clone(&redis_client),
    ));
    let message_service = Arc::new(MessageService::new(client.clone()));
    let relation_service = Arc::new(RelationService::new(client.clone()));
    let auth_service = Arc::new(AuthService::new(client.clone()));
    let room_service = Arc::new(RoomService::new(client.clone(), redis_client.clone()));
    let user_service = Arc::new(UserService::new(client.clone()));
    let chat_addr = ChatActor::new(
        Arc::clone(&conversation_service),
        Arc::clone(&message_service),
    )
    .start();

    let db = client.db();

    log::info!("start init worker");
    let _ = init_worker().await;

    log::info!("start load room active");
    let _ = preload_room_active(room_service.clone()).await;

    //Indexing
    let _ = create_user_index(db.clone()).await;
    let _ = create_relation_index(db.clone()).await;

    AppState::new(
        redis_client,
        conversation_service,
        message_service,
        relation_service,
        room_service,
        user_service,
        auth_service,
        Arc::new(chat_addr),
    )
}
