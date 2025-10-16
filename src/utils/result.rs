use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::ValidationErrors;

#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Display)]
pub enum Error {
    #[display(fmt = "Not Found")]
    NotFound,

    #[display(fmt = "Bad Request")]
    BadRequest,

    #[display(fmt = "Internal Server")]
    InternalServer,

    #[display(fmt = "Failed validation")]
    FailedValidation {
        #[serde(skip_serializing, skip_deserializing)]
        error: ValidationErrors,
    },

    #[display(fmt = "Database Error")]
    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },

    #[display(fmt = "Email already exited")]
    EmailAlreadyExisted,

    #[display(fmt = "Email not found")]
    EmailNotFound,

    #[display(fmt = "Invalid password")]
    InvalidPassword,

    #[display(fmt = "User already existed")]
    UserAlreadyExisted,

    #[display(fmt = "Coupon already exists")]
    CouponAlreadyExisted,

    #[display(fmt = "Pack not found")]
    PackNotFound,

    #[display(fmt = "You already followed this user")]
    YouAlreadyFollowedUser,

    #[display(fmt = "You already followed this user")]
    RoomNotFound,

    #[display(fmt = "User not found")]
    UserNotFound,
}

impl Error {
    pub fn from_invalid<T>(validation_error: ValidationErrors) -> Result<T> {
        Err(Error::FailedValidation {
            error: validation_error,
        })
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Error::BadRequest => HttpResponse::BadRequest().finish(),
            Error::NotFound => HttpResponse::NotFound().finish(),
            Error::FailedValidation { error: _ } => HttpResponse::BadRequest().finish(),
            Error::EmailAlreadyExisted => HttpResponse::BadRequest().json("Email already exited"),
            Error::DatabaseError {
                operation: _,
                with: _,
            } => HttpResponse::BadGateway().finish(),
            Error::InternalServer => HttpResponse::InternalServerError().finish(),
            Error::InvalidPassword => HttpResponse::BadRequest().finish(),
            Error::UserAlreadyExisted => HttpResponse::BadRequest().finish(),
            Error::CouponAlreadyExisted => HttpResponse::BadRequest().json(json!({
                "message":"Coupon already exists"
            })),
            Error::EmailNotFound => HttpResponse::BadRequest().json(json!(
                {"message":"Email not found"}
            )),
            Error::PackNotFound => HttpResponse::BadRequest().json(json!(
                {"message":"Pack not found"}
            )),
            Error::YouAlreadyFollowedUser => HttpResponse::BadRequest().json(json!(
                {"message":"You already followed this user "}
            )),
            Error::RoomNotFound => HttpResponse::BadRequest().json(json!(
                {"message":"Room Not found"}
            )),
            Error::UserNotFound => HttpResponse::BadRequest().json(json!(
                {"message":"User not found"}
            )),
        }
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}
