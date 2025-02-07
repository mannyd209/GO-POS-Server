mod db;
mod handlers;
mod middleware;
mod models;
mod utils;
mod websocket;

use actix_cors::Cors;
use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
use env_logger::Env;
use log::info;
use std::net::Ipv4Addr;

use crate::middleware::auth::AdminAuth;
use crate::models::{Discount, Option as ModOption, Staff};
use crate::utils::get_host_ipv4;

const PORT: u16 = 8000;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // Initialize database with migrations
    let pool = db::init_db();

    // Create default admin if none exists
    let conn = pool.get().expect("Failed to get database connection");
    if let Err(_) = conn.query_row("SELECT 1 FROM staff WHERE is_admin = 1", [], |_| Ok(())) {
        let admin = Staff {
            staff_id: uuid::Uuid::new_v4().to_string(),
            pin: "1432".to_string(),
            first_name: "Manny".to_string(),
            last_name: "Duarte".to_string(),
            hourly_wage: 30.0,
            is_admin: true,
        };

        conn.execute(
            "INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &admin.staff_id,
                &admin.pin,
                &admin.first_name,
                &admin.last_name,
                admin.hourly_wage,
                admin.is_admin as i32,
            ),
        )
        .expect("Failed to create default admin");
    }

    // Create default category if none exists
    if let Err(_) = conn.query_row("SELECT 1 FROM categories", [], |_| Ok(())) {
        conn.execute(
            "INSERT INTO categories (category_id, name, sort_order)
             VALUES (?1, ?2, ?3)",
            (
                uuid::Uuid::new_v4().to_string(),
                "Wings",
                1,
            ),
        )
        .expect("Failed to create default category");
    }

    // Create WebSocket broadcaster
    let broadcaster = websocket::Broadcaster::new();
    let broadcaster = Data::new(broadcaster);

    // Start mDNS service
    if let Err(e) = utils::start_mdns_service(PORT) {
        log::error!("Failed to start mDNS service: {}", e);
    }

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
                    .allow_any_header(),
            )
            .app_data(Data::new(pool.clone()))
            .app_data(broadcaster.clone())
            .service(
                web::scope("/api")
                    .wrap(AdminAuth::new())
                    .configure(handlers::staff::config)
                    .configure(handlers::catalog::config)
                    .configure(handlers::modifiers::config)
                    .configure(handlers::discounts::config)
                    .configure(handlers::health::config),
            )
            .service(web::resource("/ws").to(websocket::ws_handler))
    })
    .bind((Ipv4Addr::UNSPECIFIED, PORT))?
    .run()
    .await
}
