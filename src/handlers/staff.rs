use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::{
    db::DbPool,
    models::{ApiError, Staff, ValidatedJson},
    websocket::Broadcaster,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/staff")
            .route("", web::get().to(get_all_staff))
            .route("", web::post().to(create_staff))
            .route("/{staff_id}", web::get().to(get_staff))
            .route("/{staff_id}", web::put().to(update_staff))
            .route("/{staff_id}", web::delete().to(delete_staff))
            .route("/auth", web::post().to(authenticate_staff)),
    );
}

async fn get_all_staff(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin FROM staff",
    )?;

    let staff = stmt
        .query_map([], |row| {
            Ok(Staff {
                staff_id: row.get(0)?,
                pin: row.get(1)?,
                first_name: row.get(2)?,
                last_name: row.get(3)?,
                hourly_wage: row.get(4)?,
                is_admin: row.get::<_, i32>(5)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(staff))
}

async fn create_staff(
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
    staff: web::Json<Staff>,
) -> Result<HttpResponse, ApiError> {
    // Validate input
    let mut staff = staff.validate()?;
    
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    staff.staff_id = Uuid::new_v4().to_string();

    conn.execute(
        "INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &staff.staff_id,
            &staff.pin,
            &staff.first_name,
            &staff.last_name,
            staff.hourly_wage,
            staff.is_admin as i32,
        ),
    )?;

    broadcaster.broadcast("STAFF_CREATED", &staff).await;
    Ok(HttpResponse::Created().json(staff))
}

async fn get_staff(
    pool: web::Data<DbPool>,
    staff_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let staff = conn.query_row(
        "SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin
         FROM staff WHERE staff_id = ?1",
        [staff_id.as_str()],
        |row| {
            Ok(Staff {
                staff_id: row.get(0)?,
                pin: row.get(1)?,
                first_name: row.get(2)?,
                last_name: row.get(3)?,
                hourly_wage: row.get(4)?,
                is_admin: row.get::<_, i32>(5)? != 0,
            })
        },
    ).map_err(|_| ApiError::NotFound(format!("Staff member {} not found", staff_id)))?;

    Ok(HttpResponse::Ok().json(staff))
}

async fn update_staff(
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
    staff_id: web::Path<String>,
    staff: web::Json<Staff>,
) -> Result<HttpResponse, ApiError> {
    // Validate input
    let mut staff = staff.validate()?;
    
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    staff.staff_id = staff_id.to_string();

    let rows_affected = conn.execute(
        "UPDATE staff 
         SET pin = ?1, first_name = ?2, last_name = ?3, hourly_wage = ?4, is_admin = ?5
         WHERE staff_id = ?6",
        (
            &staff.pin,
            &staff.first_name,
            &staff.last_name,
            staff.hourly_wage,
            staff.is_admin as i32,
            &staff.staff_id,
        ),
    )?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Staff member {} not found", staff_id)));
    }

    broadcaster.broadcast("STAFF_UPDATED", &staff).await;
    Ok(HttpResponse::Ok().json(staff))
}

async fn delete_staff(
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
    staff_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute("DELETE FROM staff WHERE staff_id = ?1", [staff_id.as_str()])?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Staff member {} not found", staff_id)));
    }

    broadcaster.broadcast("STAFF_DELETED", &staff_id.to_string()).await;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true })))
}

#[derive(serde::Deserialize, validator::Validate)]
struct StaffCredentials {
    #[validate(length(min = 4, max = 4, message = "PIN must be exactly 4 characters"))]
    #[validate(regex(path = "regex::Regex::new(r\"^[0-9]{4}$\").unwrap()", message = "PIN must be 4 digits"))]
    pin: String,
}

async fn authenticate_staff(
    pool: web::Data<DbPool>,
    credentials: web::Json<StaffCredentials>,
) -> Result<HttpResponse, ApiError> {
    // Validate credentials
    let credentials = credentials.validate()?;
    
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let staff = conn.query_row(
        "SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin
         FROM staff WHERE pin = ?1",
        [&credentials.pin],
        |row| {
            Ok(Staff {
                staff_id: row.get(0)?,
                pin: row.get(1)?,
                first_name: row.get(2)?,
                last_name: row.get(3)?,
                hourly_wage: row.get(4)?,
                is_admin: row.get::<_, i32>(5)? != 0,
            })
        },
    ).map_err(|_| ApiError::Unauthorized("Invalid PIN".to_string()))?;

    Ok(HttpResponse::Ok().json(staff))
}
