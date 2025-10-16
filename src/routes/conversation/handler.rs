use crate::{
    core::state::SharedState, middlewares::auth_guard::AuthGuard,
    models::conversation::FilterConversation,
};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, ResponseError,
};
use serde_json::json;

#[post("/filter")]
pub async fn filter(
    AuthGuard(credential): AuthGuard,
    db: Data<SharedState>,
    payload: Json<FilterConversation>,
) -> HttpResponse {
    let res = db
        .conversation_service
        .filter_conversation(&credential.user.code, &payload)
        .await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(e) => e.error_response(),
    }
}
