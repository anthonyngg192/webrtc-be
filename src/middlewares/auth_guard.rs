use crate::utils::jwt_validation::{decrypt_token, validate_token, CredentialUser};
use actix_web::{
    body::BoxBody,
    dev::{Payload, ServiceRequest, ServiceResponse},
    error::InternalError,
    http::header,
    middleware::Next,
    Error, FromRequest, HttpRequest, HttpResponse,
};

use serde::Serialize;
use serde_json::json;
use std::future::{ready, Ready};

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

pub async fn validate_jwt_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ");

                if validate_token(token) {
                    return next.call(req).await;
                }
            }
        }
    }

    let error_response = ErrorResponse {
        message: "Unauthorized".to_string(),
    };

    let response = HttpResponse::Unauthorized()
        .insert_header(header::ContentType::json())
        .json(error_response);

    Ok(req.into_response(response.map_into_boxed_body()))
}

pub async fn optional_jwt_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    let auth_header = req.headers().get("Authorization");

    if let Some(auth_value) = auth_header {
        if let Ok(auth_str) = auth_value.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = auth_str.trim_start_matches("Bearer ");

                if validate_token(token) {
                    return next.call(req).await;
                }
            }
        }
        let error_response = ErrorResponse {
            message: "Unauthorized".to_string(),
        };

        let response = HttpResponse::Unauthorized()
            .insert_header(header::ContentType::json())
            .json(error_response);

        return Ok(req.into_response(response.map_into_boxed_body()));
    }
    next.call(req).await
}

pub struct AuthGuard(pub CredentialUser);

impl FromRequest for AuthGuard {
    type Error = InternalError<String>;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let access_token = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|str| str.split(" ").nth(1));

        match access_token {
            Some(token) => {
                let result = decrypt_token(token);
                match result {
                    Ok(user) => ready(Ok(AuthGuard(user))),

                    Err(_) => ready(Err(InternalError::from_response(
                        String::from("Unauthorized"),
                        HttpResponse::Unauthorized().json(json!({
                            "success": false,
                            "message": "Unauthorized"
                        })),
                    ))),
                }
            }

            None => ready(Err(InternalError::from_response(
                String::from("No token provided"),
                HttpResponse::Unauthorized().json(json!({
                  "success": false,
                  "data": {
                    "message": "No token provided"
                  }
                })),
            ))),
        }
    }
}
