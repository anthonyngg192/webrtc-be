use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, HttpServer};
use core::bootstrap::bootstrap;
use std::{env, sync::Arc};

// Modules
pub mod actors;
pub mod adapters;
pub mod core;
pub mod infra;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod utils;

#[macro_use]
extern crate async_trait;

use utils::environment::API_PORT;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=debug");

    log::info!("starting HTTP server at http://localhost:8000");

    let state = bootstrap().await;
    let arc_state = Arc::new(state);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(arc_state.clone()))
            .configure(routes::ws::router_config)
            .configure(routes::conversation::router_config)
            .configure(routes::message::router_config)
            .configure(routes::relation::router_config)
            .configure(routes::room::router_config)
            .configure(routes::auth::router_config)
            .configure(routes::user::router_config)
            .configure(routes::metrics::router_config)
            .wrap(
                Cors::default()
                    //TODO: remove allow_any_origin when deployment
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    ])
                    // .supports_credentials()
                    .max_age(3600),
            )
            .wrap(middleware::Logger::default())
    })
    .workers(8)
    .bind(("127.0.0.1", *API_PORT as u16))?
    .run()
    .await
}
