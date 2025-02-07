mod db;
mod handlers;
mod middleware;
mod models;
mod utils;
mod websocket;

use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use std::net::Ipv4Addr;
use log::info;
use crate::{
    middleware::admin::AdminAuth,
    utils::get_host_ipv4,
    handlers::{staff, catalog, modifiers, discounts, health},
};
use std::sync::Arc;

const PORT: u16 = 8000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    // Initialize database with migrations
    let pool = db::init_db();
    let pool = web::Data::new(pool);

    // Create broadcaster
    let broadcaster = Arc::new(websocket::Broadcaster::new());
    let broadcaster = web::Data::new(broadcaster);

    // Start HTTP server
    if let Ok(host_ipv4) = get_host_ipv4() {
        info!("Starting server at http://{}:{}", host_ipv4, PORT);
    } else {
        info!("Starting server at http://localhost:{}", PORT);
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
            )
            .app_data(pool.clone())
            .app_data(broadcaster.clone())
            .service(
                web::scope("/api")
                    .configure(staff::staff_config)
                    .service(
                        web::scope("/catalog")
                            .wrap(AdminAuth::new())
                            .configure(catalog::config)
                    )
                    .configure(modifiers::config)
                    .configure(discounts::config)
                    .configure(health::config)
            )
            .service(web::resource("/ws").to(websocket::ws_handler))
    })
    .bind((Ipv4Addr::UNSPECIFIED, PORT))?
    .run()
    .await
}
