use std::sync::Arc;

use actix::SystemService;
use actix_web::{
    get, post,
    web::{Data, Json, Path},
    HttpResponse, ResponseError,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    actors::room_management::{actors::RoomManager, messages::RoomManagementCreateRoom},
    core::state::SharedState,
    middlewares::auth_guard::AuthGuard,
    models::{
        room::{CreateRoom, FilterRoom},
        room_session::RoomCode,
    },
};

#[post("home")]
pub async fn filter(
    AuthGuard(credential): AuthGuard,
    db: Data<SharedState>,
    payload: Json<FilterRoom>,
) -> HttpResponse {
    let res = db
        .room_service
        .filter(&credential.user.code, &payload)
        .await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(e) => e.error_response(),
    }
}

#[post("/new")]
pub async fn create_room(
    AuthGuard(credential): AuthGuard,
    db: Data<SharedState>,
    payload: Json<CreateRoom>,
) -> HttpResponse {
    let res = db
        .room_service
        .new_room(&credential.user.code, &payload)
        .await;

    match res {
        Some(room_info) => {
            let _ = RoomManager::from_registry()
                .send(RoomManagementCreateRoom {
                    room_code: RoomCode(room_info.code.clone()),
                    owner_code: room_info.owner_code.clone(),
                    room_service: Arc::clone(&db.room_service),
                })
                .await;

            HttpResponse::Ok().json(json!(room_info))
        }
        None => HttpResponse::BadRequest().finish(),
    }
}

#[derive(Deserialize)]
struct RoomPath {
    pub room_code: String,
}

#[get("{room_code}/info")]
pub async fn get_room_info(
    AuthGuard(_): AuthGuard,
    db: Data<SharedState>,
    payload: Path<RoomPath>,
) -> HttpResponse {
    let res = db.room_service.get_room_info(&payload.room_code).await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(e) => e.error_response(),
    }
}
