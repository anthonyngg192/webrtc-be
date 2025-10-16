use actix_web::{middleware::from_fn, web};

use crate::middlewares::auth_guard::validate_jwt_middleware;
pub mod handler;

pub fn router_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/user")
            .service(handler::filter)
            .wrap(from_fn(validate_jwt_middleware))
            .service(handler::get_user),
    );
}
