use actix_web::{web, HttpResponse};
use uuid::Uuid;
use ring::digest;
use data_encoding::HEXLOWER;
use crate::{
    models::{ApiError, Staff, StaffCredentials, ValidatedJson, WsMessage},
    db::DbPool,
    websocket::Broadcaster,
};

fn hash_pin(pin: &str) -> Result<String, ApiError> {
    let digest = digest::digest(&digest::SHA256, pin.as_bytes());
    Ok(HEXLOWER.encode(digest.as_ref()))
}

pub fn staff_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/staff")
            .route("", web::post().to(create_staff))
            .route("", web::get().to(list_staff))
            .route("/{staff_id}", web::get().to(get_staff))
            .route("/{staff_id}", web::put().to(update_staff))
            .route("/{staff_id}", web::delete().to(delete_staff))
            .route("/auth", web::post().to(authenticate_staff)),
    );
}

pub async fn create_staff(
    staff: web::Json<Staff>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut staff = staff.validate()?;
    staff.staff_id = Uuid::new_v4().to_string();
    staff.pin = hash_pin(&staff.pin)?;

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
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
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::StaffCreated(staff.clone()));
    Ok(HttpResponse::Created().json(staff))
}

pub async fn update_staff(
    staff_id: web::Path<String>,
    staff: web::Json<Staff>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut staff = staff.validate()?;
    staff.pin = hash_pin(&staff.pin)?;
    staff.staff_id = staff_id.to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "UPDATE staff SET pin = ?1, first_name = ?2, last_name = ?3, hourly_wage = ?4, is_admin = ?5 
         WHERE staff_id = ?6",
        (
            &staff.pin,
            &staff.first_name,
            &staff.last_name,
            staff.hourly_wage,
            staff.is_admin as i32,
            &staff.staff_id,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Staff member {} not found", staff_id)));
    }

    broadcaster.broadcast(WsMessage::StaffUpdated(staff.clone()));
    Ok(HttpResponse::Ok().json(staff))
}

async fn delete_staff(
    staff_id: web::Path<String>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
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

    conn.execute("DELETE FROM staff WHERE staff_id = ?1", [staff_id.as_str()])
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::StaffDeleted(staff.staff_id));
    Ok(HttpResponse::NoContent().finish())
}

pub async fn authenticate_staff(
    credentials: web::Json<StaffCredentials>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let credentials = credentials.validate()?;
    let conn = pool.get()?;

    let staff = conn.query_row(
        "SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin FROM staff WHERE staff_id = ?1 AND pin = ?2",
        [&credentials.staff_id, &credentials.pin],
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
    ).map_err(|err| {
        match err {
            rusqlite::Error::QueryReturnedNoRows => {
                ApiError::Unauthorized("Invalid credentials".to_string())
            }
            _ => ApiError::DatabaseError(err.to_string()),
        }
    })?;

    Ok(HttpResponse::Ok().json(staff))
}

async fn list_staff(
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT staff_id, pin, first_name, last_name, hourly_wage, is_admin FROM staff"
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let staff_iter = stmt.query_map([], |row| {
        Ok(Staff {
            staff_id: row.get(0)?,
            pin: row.get(1)?,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
            hourly_wage: row.get(4)?,
            is_admin: row.get::<_, i32>(5)? != 0,
        })
    }).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let staff: Result<Vec<_>, _> = staff_iter.collect();
    let staff = staff.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(staff))
}

async fn get_staff(
    staff_id: web::Path<String>,
    pool: web::Data<DbPool>,
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
