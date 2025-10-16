use actix_web::{
    get, post,
    web::{Data, Json, Path},
    HttpResponse, ResponseError,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    core::state::SharedState, middlewares::auth_guard::AuthGuard, models::user::FilterUserQuery,
};

#[derive(Deserialize)]
struct UserParam {
    user_code: String,
}

#[post("filter")]
pub async fn filter(
    AuthGuard(credential): AuthGuard,
    db: Data<SharedState>,
    payload: Json<FilterUserQuery>,
) -> HttpResponse {
    let res = db
        .user_service
        .filter_users(credential.user.code, &payload)
        .await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(e) => e.error_response(),
    }
}

#[get("{user_code}/info")]
pub async fn get_user(db: Data<SharedState>, path: Path<UserParam>) -> HttpResponse {
    let res = db.user_service.get_user(&path.user_code).await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(e) => e.error_response(),
    }
}
