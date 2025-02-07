use actix_web::{web, HttpResponse};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use crate::models::{ApiError, Modifier, Option as ModOption, WsMessage};
use crate::websocket::Broadcaster;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/modifiers")
            .route("", web::post().to(create_modifier))
            .route("", web::get().to(get_all_modifiers))
            .route("/{modifier_id}", web::get().to(get_modifier))
            .route("/{modifier_id}", web::put().to(update_modifier))
            .route("/{modifier_id}", web::delete().to(delete_modifier))
            .route("/{modifier_id}/options", web::post().to(create_option))
            .route("/{modifier_id}/options", web::get().to(get_modifier_options))
            .route("/options/{option_id}", web::get().to(get_option))
            .route("/options/{option_id}", web::put().to(update_option))
            .route("/options/{option_id}", web::delete().to(delete_option)),
    );
}

pub async fn get_all_modifiers(
    pool: web::Data<Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let mut stmt = conn.prepare("SELECT * FROM modifiers")?;
    let modifiers_iter = stmt.query_map([], |row| {
        Ok(Modifier {
            modifier_id: row.get(0)?,
            item_id: row.get(1)?,
            name: row.get(2)?,
            single_selection: row.get::<_, i32>(3)? != 0,
            sort_order: row.get(4)?,
        })
    })?;

    let modifiers: Result<Vec<_>, _> = modifiers_iter.collect();
    Ok(HttpResponse::Ok().json(modifiers?))
}

pub async fn create_modifier(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    modifier: web::Json<Modifier>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let modifier = modifier.into_inner();

    conn.execute(
        "INSERT INTO modifiers (modifier_id, item_id, name, single_selection, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![
            modifier.modifier_id,
            modifier.item_id,
            modifier.name,
            modifier.single_selection as i32,
            modifier.sort_order,
        ],
    )?;

    broadcaster.broadcast(WsMessage::ModifierCreated(modifier.clone()));
    Ok(HttpResponse::Created().json(modifier))
}

pub async fn get_modifier(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    modifier_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let modifier_id = modifier_id.to_string();

    match conn.query_row(
        "SELECT * FROM modifiers WHERE modifier_id = ?1",
        [modifier_id.clone()],
        |row| {
            Ok(Modifier {
                modifier_id: row.get(0)?,
                item_id: row.get(1)?,
                name: row.get(2)?,
                single_selection: row.get::<_, i32>(3)? != 0,
                sort_order: row.get(4)?,
            })
        },
    ) {
        Ok(modifier) => Ok(HttpResponse::Ok().json(modifier)),
        Err(_) => Err(ApiError::NotFound(format!("Modifier {} not found", modifier_id))),
    }
}

pub async fn update_modifier(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    modifier_id: web::Path<String>,
    modifier: web::Json<Modifier>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let modifier_id = modifier_id.to_string();
    let modifier = modifier.into_inner();

    if let Err(_) = conn.query_row(
        "SELECT 1 FROM modifiers WHERE modifier_id = ?1",
        [modifier_id.clone()],
        |_| Ok(()),
    ) {
        return Err(ApiError::NotFound(format!("Modifier {} not found", modifier_id)));
    }

    conn.execute(
        "UPDATE modifiers SET item_id = ?1, name = ?2, single_selection = ?3, sort_order = ?4
         WHERE modifier_id = ?5",
        rusqlite::params![
            modifier.item_id,
            modifier.name,
            modifier.single_selection as i32,
            modifier.sort_order,
            modifier_id.clone(),
        ],
    )?;

    let updated_modifier = Modifier {
        modifier_id: modifier_id,
        item_id: modifier.item_id,
        name: modifier.name,
        single_selection: modifier.single_selection,
        sort_order: modifier.sort_order,
    };

    broadcaster.broadcast(WsMessage::ModifierUpdated(updated_modifier.clone()));
    Ok(HttpResponse::Ok().json(updated_modifier))
}

pub async fn delete_modifier(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    modifier_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let modifier_id = modifier_id.to_string();

    if let Err(_) = conn.query_row(
        "SELECT 1 FROM modifiers WHERE modifier_id = ?1",
        [modifier_id.clone()],
        |_| Ok(()),
    ) {
        return Err(ApiError::NotFound(format!("Modifier {} not found", modifier_id)));
    }

    conn.execute(
        "DELETE FROM modifiers WHERE modifier_id = ?1",
        [modifier_id.clone()],
    )?;

    broadcaster.broadcast(WsMessage::ModifierDeleted(modifier_id));
    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_modifier_options(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    modifier_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let modifier_id = modifier_id.to_string();

    let mut stmt = conn.prepare("SELECT * FROM options WHERE modifier_id = ?1")?;
    let options_iter = stmt.query_map([modifier_id], |row| {
        Ok(ModOption {
            option_id: row.get(0)?,
            modifier_id: row.get(1)?,
            name: row.get(2)?,
            price: row.get(3)?,
            available: row.get::<_, i32>(4)? != 0,
            sort_order: row.get(5)?,
        })
    })?;

    let options: Result<Vec<_>, _> = options_iter.collect();
    Ok(HttpResponse::Ok().json(options?))
}

pub async fn create_option(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    modifier_id: web::Path<String>,
    option: web::Json<ModOption>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let modifier_id = modifier_id.to_string();
    let option = option.into_inner();

    // Verify modifier exists
    if let Err(_) = conn.query_row(
        "SELECT 1 FROM modifiers WHERE modifier_id = ?1",
        [modifier_id.clone()],
        |_| Ok(()),
    ) {
        return Err(ApiError::NotFound(format!("Modifier {} not found", modifier_id)));
    }

    conn.execute(
        "INSERT INTO options (option_id, modifier_id, name, price, available, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            option.option_id,
            option.modifier_id,
            option.name,
            option.price,
            option.available as i32,
            option.sort_order,
        ],
    )?;

    broadcaster.broadcast(WsMessage::OptionCreated(option.clone()));
    Ok(HttpResponse::Created().json(option))
}

pub async fn get_option(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    option_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let option_id = option_id.to_string();

    match conn.query_row(
        "SELECT * FROM options WHERE option_id = ?1",
        [option_id.clone()],
        |row| {
            Ok(ModOption {
                option_id: row.get(0)?,
                modifier_id: row.get(1)?,
                name: row.get(2)?,
                price: row.get(3)?,
                available: row.get::<_, i32>(4)? != 0,
                sort_order: row.get(5)?,
            })
        },
    ) {
        Ok(option) => Ok(HttpResponse::Ok().json(option)),
        Err(_) => Err(ApiError::NotFound(format!("Option {} not found", option_id))),
    }
}

pub async fn update_option(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    option_id: web::Path<String>,
    option: web::Json<ModOption>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let option_id = option_id.to_string();
    let option = option.into_inner();

    if let Err(_) = conn.query_row(
        "SELECT 1 FROM options WHERE option_id = ?1",
        [option_id.clone()],
        |_| Ok(()),
    ) {
        return Err(ApiError::NotFound(format!("Option {} not found", option_id)));
    }

    conn.execute(
        "UPDATE options SET modifier_id = ?1, name = ?2, price = ?3, available = ?4, sort_order = ?5
         WHERE option_id = ?6",
        rusqlite::params![
            option.modifier_id,
            option.name,
            option.price,
            option.available as i32,
            option.sort_order,
            option_id.clone(),
        ],
    )?;

    let updated_option = ModOption {
        option_id: option_id,
        modifier_id: option.modifier_id,
        name: option.name,
        price: option.price,
        available: option.available,
        sort_order: option.sort_order,
    };

    broadcaster.broadcast(WsMessage::OptionUpdated(updated_option.clone()));
    Ok(HttpResponse::Ok().json(updated_option))
}

pub async fn delete_option(
    pool: web::Data<Pool<SqliteConnectionManager>>,
    broadcaster: web::Data<Broadcaster>,
    option_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get()?;
    let option_id = option_id.to_string();

    if let Err(_) = conn.query_row(
        "SELECT 1 FROM options WHERE option_id = ?1",
        [option_id.clone()],
        |_| Ok(()),
    ) {
        return Err(ApiError::NotFound(format!("Option {} not found", option_id)));
    }

    conn.execute(
        "DELETE FROM options WHERE option_id = ?1",
        [option_id.clone()],
    )?;

    broadcaster.broadcast(WsMessage::OptionDeleted(option_id));
    Ok(HttpResponse::NoContent().finish())
}
