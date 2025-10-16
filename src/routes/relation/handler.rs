use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, ResponseError,
};
use serde_json::json;

use crate::{
    core::state::SharedState, middlewares::auth_guard::AuthGuard, models::relation::NewRelation,
};

#[post("new_relation")]
pub async fn new_relation(
    AuthGuard(credential): AuthGuard,
    db: Data<SharedState>,
    payload: Json<NewRelation>,
) -> HttpResponse {
    let res = db
        .relation_service
        .new_relation(&credential.user.code, &payload)
        .await;
    match res {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
        })),
        Err(e) => e.error_response(),
    }
}
