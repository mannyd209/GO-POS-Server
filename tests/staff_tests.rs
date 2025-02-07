use actix_web::http::Method;
use serde_json::json;
use uuid::Uuid;

mod common;
use common::{setup_test_app, test_request};

#[actix_rt::test]
async fn test_staff_authentication() {
    let (app, _pool, admin) = setup_test_app().await;

    // Test successful authentication
    let resp: serde_json::Value = test_request(
        &app,
        Method::POST,
        "/api/auth",
        Some(&json!({
            "pin": admin.pin
        }).to_string().into_bytes()),
        None,
    )
    .await;

    assert!(resp["success"].as_bool().unwrap());
    assert!(resp["is_admin"].as_bool().unwrap());

    // Test failed authentication
    let resp = test_request::<serde_json::Value>(
        &app,
        Method::POST,
        "/api/auth",
        Some(&json!({
            "pin": "wrong_pin"
        }).to_string().into_bytes()),
        None,
    )
    .await;

    assert!(!resp["success"].as_bool().unwrap());
}

#[actix_rt::test]
async fn test_staff_crud_operations() {
    let (app, _pool, admin) = setup_test_app().await;

    // Create new staff member
    let new_staff_id = Uuid::new_v4().to_string();
    let resp: serde_json::Value = test_request(
        &app,
        Method::POST,
        "/api/staff",
        Some(&json!({
            "staff_id": new_staff_id,
            "pin": "1234",
            "first_name": "John",
            "last_name": "Doe",
            "hourly_wage": 25.0,
            "is_admin": false
        }).to_string().into_bytes()),
        Some(&admin.pin),
    )
    .await;

    assert_eq!(resp["staff_id"].as_str().unwrap(), new_staff_id);

    // Get staff member
    let resp: serde_json::Value = test_request(
        &app,
        Method::GET,
        &format!("/api/staff/{}", new_staff_id),
        None,
        Some(&admin.pin),
    )
    .await;

    assert_eq!(resp["first_name"].as_str().unwrap(), "John");
    assert_eq!(resp["last_name"].as_str().unwrap(), "Doe");

    // Update staff member
    let resp: serde_json::Value = test_request(
        &app,
        Method::PUT,
        &format!("/api/staff/{}", new_staff_id),
        Some(&json!({
            "pin": "1234",
            "first_name": "Jane",
            "last_name": "Doe",
            "hourly_wage": 30.0,
            "is_admin": false
        }).to_string().into_bytes()),
        Some(&admin.pin),
    )
    .await;

    assert_eq!(resp["first_name"].as_str().unwrap(), "Jane");

    // Delete staff member
    let resp: serde_json::Value = test_request(
        &app,
        Method::DELETE,
        &format!("/api/staff/{}", new_staff_id),
        None,
        Some(&admin.pin),
    )
    .await;

    assert!(resp["success"].as_bool().unwrap());
}

#[actix_rt::test]
async fn test_unauthorized_access() {
    let (app, _pool, _admin) = setup_test_app().await;

    // Try to access admin endpoint without authentication
    let resp = test_request::<serde_json::Value>(
        &app,
        Method::GET,
        "/api/staff/all",
        None,
        None,
    )
    .await;

    assert!(resp["error"].as_str().unwrap().contains("Admin access required"));

    // Try to access admin endpoint with non-admin PIN
    let resp = test_request::<serde_json::Value>(
        &app,
        Method::GET,
        "/api/staff/all",
        None,
        Some("non_admin_pin"),
    )
    .await;

    assert!(resp["error"].as_str().unwrap().contains("Admin access required"));
}
