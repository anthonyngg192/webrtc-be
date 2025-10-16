use actix_web::web;

pub mod handler;

pub fn router_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("").service(handler::metrics));
}
