pub mod handler;
use actix_web::web;

pub fn router_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("api/conversation").service(handler::filter));
}
