use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse, ResponseError,
};
use serde_json::json;

use crate::{
    core::state::SharedState,
    middlewares::auth_guard::AuthGuard,
    models::user::{LoginPayload, NewUserPayload},
};

#[post("login")]
pub async fn login(state: Data<SharedState>, payload: Json<LoginPayload>) -> HttpResponse {
    let res = state.auth_service.login(&payload).await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!(result)),
        Err(e) => e.error_response(),
    }
}

#[get("profile")]
pub async fn profile(AuthGuard(credential): AuthGuard) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "success": true,
        "data": credential.user
    }))
}

#[post("sign_up")]
pub async fn sign_up(state: Data<SharedState>, payload: Json<NewUserPayload>) -> HttpResponse {
    let res = state.auth_service.new_user(&payload).await;
    match res {
        Ok(result) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": result
        })),
        Err(e) => e.error_response(),
    }
}
