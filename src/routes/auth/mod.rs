mod handler;
use actix_web::web;

pub fn router_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/auth")
            .service(handler::login)
            .service(handler::profile)
            .service(handler::sign_up),
    );
}
