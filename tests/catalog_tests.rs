use actix_web::{
    http::{header, StatusCode},
    test,
};
use serde_json::json;
use uuid::Uuid;

use crate::{
    db::init_test_db,
    models::{Category, Discount, Item, Modifier, Option as ModOption},
    tests::common::test_app,
};

#[actix_web::test]
async fn test_category_crud() {
    let app = test_app().await;
    let db = init_test_db().await;

    // Create admin staff for authorization
    let admin_pin = "1234";
    db.execute(
        "INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            Uuid::new_v4().to_string(),
            admin_pin,
            "Admin",
            "User",
            20.0,
            1,
        ),
    )
    .unwrap();

    // Test create category
    let category = Category {
        category_id: String::new(),
        name: "Test Category".to_string(),
        sort_order: 1,
    };

    let req = test::TestRequest::post()
        .uri("/catalog/categories")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_pin)))
        .set_json(&category)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let created: Category = test::read_body_json(resp).await;
    assert_eq!(created.name, category.name);
    assert_eq!(created.sort_order, category.sort_order);

    // Test get category
    let req = test::TestRequest::get()
        .uri(&format!("/catalog/categories/{}", created.category_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_pin)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let fetched: Category = test::read_body_json(resp).await;
    assert_eq!(fetched.category_id, created.category_id);

    // Test update category
    let mut updated = created.clone();
    updated.name = "Updated Category".to_string();

    let req = test::TestRequest::put()
        .uri(&format!("/catalog/categories/{}", created.category_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_pin)))
        .set_json(&updated)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let updated_resp: Category = test::read_body_json(resp).await;
    assert_eq!(updated_resp.name, updated.name);

    // Test delete category
    let req = test::TestRequest::delete()
        .uri(&format!("/catalog/categories/{}", created.category_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_pin)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deletion
    let req = test::TestRequest::get()
        .uri(&format!("/catalog/categories/{}", created.category_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_pin)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_unauthorized_access() {
    let app = test_app().await;

    // Try to access without admin PIN
    let req = test::TestRequest::get()
        .uri("/catalog/categories")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Try to access with non-admin PIN
    let db = init_test_db().await;
    let non_admin_pin = "5678";
    db.execute(
        "INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            Uuid::new_v4().to_string(),
            non_admin_pin,
            "Non",
            "Admin",
            15.0,
            0,
        ),
    )
    .unwrap();

    let req = test::TestRequest::get()
        .uri("/catalog/categories")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", non_admin_pin)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_validation() {
    let app = test_app().await;
    let db = init_test_db().await;

    // Create admin staff
    let admin_pin = "1234";
    db.execute(
        "INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            Uuid::new_v4().to_string(),
            admin_pin,
            "Admin",
            "User",
            20.0,
            1,
        ),
    )
    .unwrap();

    // Test category validation
    let invalid_category = json!({
        "category_id": "",
        "name": "", // Invalid: empty name
        "sort_order": -1 // Invalid: negative sort order
    });

    let req = test::TestRequest::post()
        .uri("/catalog/categories")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_pin)))
        .set_json(&invalid_category)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    // Test item validation
    let invalid_item = json!({
        "item_id": "",
        "category_id": Uuid::new_v4().to_string(),
        "name": "", // Invalid: empty name
        "regular_price": -10.0, // Invalid: negative price
        "event_price": -5.0, // Invalid: negative price
        "sort_order": -1, // Invalid: negative sort order
        "available": true
    });

    let req = test::TestRequest::post()
        .uri("/catalog/items")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_pin)))
        .set_json(&invalid_item)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
