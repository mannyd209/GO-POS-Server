use actix_web::{
    dev::ServiceResponse,
    test::{self, TestRequest},
    web, App,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use pos_backend::{
    db,
    handlers::{catalog, discounts, health, modifiers, staff},
    middleware::auth::AdminAuth,
    models::Staff,
    websocket::Broadcaster,
};

pub async fn setup_test_app() -> (
    impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
    Pool<SqliteConnectionManager>,
    Staff,
) {
    // Create in-memory database for testing
    let manager = SqliteConnectionManager::memory();
    let pool = Pool::new(manager).expect("Failed to create test database pool");

    // Run migrations
    db::migrations::run_migrations(&pool).expect("Failed to run database migrations");

    // Create test admin
    let admin = Staff {
        staff_id: Uuid::new_v4().to_string(),
        pin: "1432".to_string(),
        first_name: "Test".to_string(),
        last_name: "Admin".to_string(),
        hourly_wage: 30.0,
        is_admin: true,
    };

    let conn = pool.get().unwrap();
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
    .expect("Failed to create test admin");

    // Create test app
    let broadcaster = web::Data::new(Broadcaster::new());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(broadcaster)
            .service(
                web::scope("/api")
                    .wrap(AdminAuth::new())
                    .configure(staff::config)
                    .configure(catalog::config)
                    .configure(modifiers::config)
                    .configure(discounts::config)
                    .configure(health::config),
            ),
    )
    .await;

    (app, pool, admin)
}

pub async fn test_request<T>(
    app: &impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
    method: actix_web::http::Method,
    uri: &str,
    body: Option<&[u8]>,
    admin_pin: Option<&str>,
) -> T
where
    T: DeserializeOwned,
{
    let mut req = TestRequest::with_uri(uri).method(method);

    if let Some(pin) = admin_pin {
        req = req.insert_header(("Authorization", format!("Bearer {}", pin)));
    }

    if let Some(body) = body {
        req = req.set_payload(body);
        req = req.insert_header(("content-type", "application/json"));
    }

    let resp = test::call_service(app, req.to_request()).await;
    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    serde_json::from_slice(&body).unwrap()
}
