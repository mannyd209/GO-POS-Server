use actix_web::{web, HttpResponse};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::models::{ApiError, Discount, WsMessage};
use crate::websocket::Broadcaster;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/discounts")
            .route("", web::post().to(create_discount))
            .route("", web::get().to(get_all_discounts))
            .route("/{discount_id}", web::get().to(get_discount))
            .route("/{discount_id}", web::put().to(update_discount))
            .route("/{discount_id}", web::delete().to(delete_discount)),
    );
}

pub async fn get_all_discounts(
    pool: web::Data<Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM discounts")?;
    let discounts_iter = stmt.query_map([], |row| {
        Ok(Discount {
            discount_id: row.get(0)?,
            name: row.get(1)?,
            is_percentage: row.get::<_, i32>(2)? != 0,
            amount: row.get(3)?,
            available: row.get::<_, i32>(4)? != 0,
            sort_order: row.get(5)?,
        })
    })?;

    let discounts: Result<Vec<_>, _> = discounts_iter.collect();
    Ok(HttpResponse::Ok().json(discounts?))
}

pub async fn create_discount(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    discount: web::Json<Discount>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let discount = discount.into_inner();

    conn.execute(
        "INSERT INTO discounts (discount_id, name, is_percentage, amount, available, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            discount.discount_id,
            discount.name,
            discount.is_percentage as i32,
            discount.amount,
            discount.available as i32,
            discount.sort_order,
        ],
    )?;

    broadcaster.broadcast(WsMessage::DiscountCreated(discount.clone()));
    Ok(HttpResponse::Created().json(discount))
}

pub async fn get_discount(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    discount_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let discount_id = discount_id.to_string();

    match conn.query_row(
        "SELECT * FROM discounts WHERE discount_id = ?1",
        [discount_id.clone()],
        |row| {
            Ok(Discount {
                discount_id: row.get(0)?,
                name: row.get(1)?,
                is_percentage: row.get::<_, i32>(2)? != 0,
                amount: row.get(3)?,
                available: row.get::<_, i32>(4)? != 0,
                sort_order: row.get(5)?,
            })
        },
    ) {
        Ok(discount) => Ok(HttpResponse::Ok().json(discount)),
        Err(_) => Err(ApiError::NotFound(format!("Discount {} not found", discount_id))),
    }
}

pub async fn update_discount(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    discount_id: web::Path<String>,
    discount: web::Json<Discount>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let discount_id = discount_id.to_string();
    let discount = discount.into_inner();

    if let Err(_) = conn.query_row(
        "SELECT 1 FROM discounts WHERE discount_id = ?1",
        [discount_id.clone()],
        |_| Ok(()),
    ) {
        return Err(ApiError::NotFound(format!("Discount {} not found", discount_id)));
    }

    conn.execute(
        "UPDATE discounts SET name = ?1, is_percentage = ?2, amount = ?3, available = ?4, sort_order = ?5
         WHERE discount_id = ?6",
        rusqlite::params![
            discount.name,
            discount.is_percentage as i32,
            discount.amount,
            discount.available as i32,
            discount.sort_order,
            discount_id.clone(),
        ],
    )?;

    let updated_discount = Discount {
        discount_id: discount_id,
        name: discount.name,
        is_percentage: discount.is_percentage,
        amount: discount.amount,
        available: discount.available,
        sort_order: discount.sort_order,
    };

    broadcaster.broadcast(WsMessage::DiscountUpdated(updated_discount.clone()));
    Ok(HttpResponse::Ok().json(updated_discount))
}

pub async fn delete_discount(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    discount_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let discount_id = discount_id.to_string();

    if let Err(_) = conn.query_row(
        "SELECT 1 FROM discounts WHERE discount_id = ?1",
        [discount_id.clone()],
        |_| Ok(()),
    ) {
        return Err(ApiError::NotFound(format!("Discount {} not found", discount_id)));
    }

    conn.execute(
        "DELETE FROM discounts WHERE discount_id = ?1",
        [discount_id.clone()],
    )?;

    broadcaster.broadcast(WsMessage::DiscountDeleted(discount_id));
    Ok(HttpResponse::NoContent().finish())
}
