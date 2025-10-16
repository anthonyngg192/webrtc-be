use actix_web::web;
pub mod handler;

pub fn router_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("api/room")
            .service(handler::filter)
            .service(handler::create_room)
            .service(handler::get_room_info),
    );
}
