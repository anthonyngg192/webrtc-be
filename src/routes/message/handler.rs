use actix_web::{
    post,
    web::{Data, Json, Path},
    HttpResponse, ResponseError,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    core::state::SharedState,
    middlewares::auth_guard::AuthGuard,
    models::{conversation::FilterMessage, message::NewMessage},
};

#[derive(Deserialize)]
struct ConversationPatch {
    pub conversation_id: String,
}

#[post("{conversation_id}/new_message")]
pub async fn new_message(
    AuthGuard(credential): AuthGuard,
    db: Data<SharedState>,
    path: Path<ConversationPatch>,
    payload: Json<NewMessage>,
) -> HttpResponse {
    let res = db
        .message_service
        .new_message(&path.conversation_id, &credential.user.code, &payload)
        .await;
    match res {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
        })),
        Err(e) => e.error_response(),
    }
}

#[post("{conversation_id}/filter")]
pub async fn filter(
    AuthGuard(credential): AuthGuard,
    db: Data<SharedState>,
    path: Path<ConversationPatch>,
    payload: Json<FilterMessage>,
) -> HttpResponse {
    let res = db
        .message_service
        .filter_message(&path.conversation_id, &credential.user.code, &payload)
        .await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(e) => e.error_response(),
    }
}
