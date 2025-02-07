use actix_web::{web, HttpResponse};
use uuid::Uuid;
use crate::{
    models::{ApiError, Category, Item, Modifier, Option as ModOption, Discount, ValidatedJson, WsMessage},
    db::DbPool,
    websocket::Broadcaster,
};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/catalog")
            .service(
                web::scope("/categories")
                    .route("", web::get().to(list_categories))
                    .route("", web::post().to(create_category))
                    .route("/{category_id}", web::get().to(get_category))
                    .route("/{category_id}", web::put().to(update_category))
                    .route("/{category_id}", web::delete().to(delete_category))
            )
            .service(
                web::scope("/items")
                    .route("", web::get().to(list_items))
                    .route("", web::post().to(create_item))
                    .route("/{item_id}", web::get().to(get_item))
                    .route("/{item_id}", web::put().to(update_item))
                    .route("/{item_id}", web::delete().to(delete_item))
            )
            .service(
                web::scope("/modifiers")
                    .route("", web::get().to(list_modifiers))
                    .route("", web::post().to(create_modifier))
                    .route("/{modifier_id}", web::get().to(get_modifier))
                    .route("/{modifier_id}", web::put().to(update_modifier))
                    .route("/{modifier_id}", web::delete().to(delete_modifier))
            )
            .service(
                web::scope("/options")
                    .route("", web::get().to(list_options))
                    .route("", web::post().to(create_option))
                    .route("/{option_id}", web::get().to(get_option))
                    .route("/{option_id}", web::put().to(update_option))
                    .route("/{option_id}", web::delete().to(delete_option))
            )
            .service(
                web::scope("/discounts")
                    .route("", web::get().to(list_discounts))
                    .route("", web::post().to(create_discount))
                    .route("/{discount_id}", web::get().to(get_discount))
                    .route("/{discount_id}", web::put().to(update_discount))
                    .route("/{discount_id}", web::delete().to(delete_discount))
            )
    );
}

async fn create_category(
    category: web::Json<Category>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut category = category.validate()?;
    category.category_id = Uuid::new_v4().to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    conn.execute(
        "INSERT INTO categories (category_id, name, sort_order) VALUES (?1, ?2, ?3)",
        (&category.category_id, &category.name, category.sort_order),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::CategoryCreated(category.clone()));
    Ok(HttpResponse::Created().json(category))
}

async fn list_categories(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let mut stmt = conn.prepare("SELECT category_id, name, sort_order FROM categories")?;

    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                category_id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(categories))
}

async fn get_category(
    category_id: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let category = conn.query_row(
        "SELECT category_id, name, sort_order FROM categories WHERE category_id = ?1",
        [category_id.as_str()],
        |row| {
            Ok(Category {
                category_id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        },
    ).map_err(|_| ApiError::NotFound(format!("Category {} not found", category_id)))?;

    Ok(HttpResponse::Ok().json(category))
}

async fn update_category(
    category_id: web::Path<String>,
    category: web::Json<Category>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut category = category.validate()?;
    category.category_id = category_id.to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "UPDATE categories SET name = ?1, sort_order = ?2 WHERE category_id = ?3",
        (&category.name, category.sort_order, &category.category_id),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Category {} not found", category_id)));
    }

    broadcaster.broadcast(WsMessage::CategoryUpdated(category.clone()));
    Ok(HttpResponse::Ok().json(category))
}

async fn delete_category(
    category_id: web::Path<String>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    // First check if category exists
    let category = conn.query_row(
        "SELECT category_id, name, sort_order FROM categories WHERE category_id = ?1",
        [category_id.as_str()],
        |row| {
            Ok(Category {
                category_id: row.get(0)?,
                name: row.get(1)?,
                sort_order: row.get(2)?,
            })
        },
    ).map_err(|_| ApiError::NotFound(format!("Category {} not found", category_id)))?;

    // Then delete it
    conn.execute(
        "DELETE FROM categories WHERE category_id = ?1",
        [category_id.as_str()],
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::CategoryDeleted(category.category_id));
    Ok(HttpResponse::NoContent().finish())
}

async fn create_item(
    item: web::Json<Item>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut item = item.validate()?;
    item.item_id = Uuid::new_v4().to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    conn.execute(
        "INSERT INTO items (item_id, category_id, name, regular_price, event_price, sort_order, available)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            &item.item_id,
            &item.category_id,
            &item.name,
            item.regular_price,
            item.event_price,
            item.sort_order,
            item.available as i32,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::ItemCreated(item.clone()));
    Ok(HttpResponse::Created().json(item))
}

async fn list_items(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT item_id, category_id, name, regular_price, event_price, sort_order, available 
         FROM items",
    )?;

    let items = stmt
        .query_map([], |row| {
            Ok(Item {
                item_id: row.get(0)?,
                category_id: row.get(1)?,
                name: row.get(2)?,
                regular_price: row.get(3)?,
                event_price: row.get(4)?,
                sort_order: row.get(5)?,
                available: row.get::<_, i32>(6)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(items))
}

async fn get_item(
    item_id: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let item = conn.query_row(
        "SELECT item_id, category_id, name, regular_price, event_price, sort_order, available
         FROM items WHERE item_id = ?1",
        [item_id.as_str()],
        |row| {
            Ok(Item {
                item_id: row.get(0)?,
                category_id: row.get(1)?,
                name: row.get(2)?,
                regular_price: row.get(3)?,
                event_price: row.get(4)?,
                sort_order: row.get(5)?,
                available: row.get::<_, i32>(6)? != 0,
            })
        },
    ).map_err(|_| ApiError::NotFound(format!("Item {} not found", item_id)))?;

    Ok(HttpResponse::Ok().json(item))
}

async fn update_item(
    item_id: web::Path<String>,
    item: web::Json<Item>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut item = item.validate()?;
    item.item_id = item_id.to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "UPDATE items 
         SET category_id = ?1, name = ?2, regular_price = ?3, event_price = ?4, sort_order = ?5, available = ?6
         WHERE item_id = ?7",
        (
            &item.category_id,
            &item.name,
            item.regular_price,
            item.event_price,
            item.sort_order,
            item.available as i32,
            &item.item_id,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Item {} not found", item_id)));
    }

    broadcaster.broadcast(WsMessage::ItemUpdated(item.clone()));
    Ok(HttpResponse::Ok().json(item))
}

async fn delete_item(
    item_id: web::Path<String>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "DELETE FROM items WHERE item_id = ?1",
        [item_id.as_str()],
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Item {} not found", item_id)));
    }

    broadcaster.broadcast(WsMessage::ItemDeleted(item_id.to_string()));
    Ok(HttpResponse::NoContent().finish())
}

async fn create_modifier(
    modifier: web::Json<Modifier>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut modifier = modifier.validate()?;
    modifier.modifier_id = Uuid::new_v4().to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    conn.execute(
        "INSERT INTO modifiers (modifier_id, item_id, name, single_selection, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        (
            &modifier.modifier_id,
            &modifier.item_id,
            &modifier.name,
            modifier.single_selection as i32,
            modifier.sort_order,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::ModifierCreated(modifier.clone()));
    Ok(HttpResponse::Created().json(modifier))
}

async fn list_modifiers(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT modifier_id, item_id, name, single_selection, sort_order FROM modifiers",
    )?;

    let modifiers = stmt
        .query_map([], |row| {
            Ok(Modifier {
                modifier_id: row.get(0)?,
                item_id: row.get(1)?,
                name: row.get(2)?,
                single_selection: row.get::<_, i32>(3)? != 0,
                sort_order: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(modifiers))
}

async fn get_modifier(
    modifier_id: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let modifier = conn.query_row(
        "SELECT modifier_id, item_id, name, single_selection, sort_order
         FROM modifiers WHERE modifier_id = ?1",
        [modifier_id.as_str()],
        |row| {
            Ok(Modifier {
                modifier_id: row.get(0)?,
                item_id: row.get(1)?,
                name: row.get(2)?,
                single_selection: row.get::<_, i32>(3)? != 0,
                sort_order: row.get(4)?,
            })
        },
    ).map_err(|_| ApiError::NotFound(format!("Modifier {} not found", modifier_id)))?;

    Ok(HttpResponse::Ok().json(modifier))
}

async fn update_modifier(
    modifier_id: web::Path<String>,
    modifier: web::Json<Modifier>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut modifier = modifier.validate()?;
    modifier.modifier_id = modifier_id.to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "UPDATE modifiers 
         SET item_id = ?1, name = ?2, single_selection = ?3, sort_order = ?4
         WHERE modifier_id = ?5",
        (
            &modifier.item_id,
            &modifier.name,
            modifier.single_selection as i32,
            modifier.sort_order,
            &modifier.modifier_id,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Modifier {} not found", modifier_id)));
    }

    broadcaster.broadcast(WsMessage::ModifierUpdated(modifier.clone()));
    Ok(HttpResponse::Ok().json(modifier))
}

async fn delete_modifier(
    modifier_id: web::Path<String>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "DELETE FROM modifiers WHERE modifier_id = ?1",
        [modifier_id.as_str()],
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Modifier {} not found", modifier_id)));
    }

    broadcaster.broadcast(WsMessage::ModifierDeleted(modifier_id.to_string()));
    Ok(HttpResponse::NoContent().finish())
}

async fn create_option(
    option: web::Json<ModOption>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut option = option.validate()?;
    option.option_id = Uuid::new_v4().to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    conn.execute(
        "INSERT INTO options (option_id, modifier_id, name, price, available, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &option.option_id,
            &option.modifier_id,
            &option.name,
            option.price,
            option.available as i32,
            option.sort_order,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::OptionCreated(option.clone()));
    Ok(HttpResponse::Created().json(option))
}

async fn list_options(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT option_id, modifier_id, name, price, available, sort_order FROM options",
    )?;

    let options = stmt
        .query_map([], |row| {
            Ok(ModOption {
                option_id: row.get(0)?,
                modifier_id: row.get(1)?,
                name: row.get(2)?,
                price: row.get(3)?,
                available: row.get::<_, i32>(4)? != 0,
                sort_order: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(options))
}

async fn get_option(
    option_id: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let option = conn.query_row(
        "SELECT option_id, modifier_id, name, price, available, sort_order
         FROM options WHERE option_id = ?1",
        [option_id.as_str()],
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
    ).map_err(|_| ApiError::NotFound(format!("Option {} not found", option_id)))?;

    Ok(HttpResponse::Ok().json(option))
}

async fn update_option(
    option_id: web::Path<String>,
    option: web::Json<ModOption>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut option = option.validate()?;
    option.option_id = option_id.to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "UPDATE options 
         SET modifier_id = ?1, name = ?2, price = ?3, available = ?4, sort_order = ?5
         WHERE option_id = ?6",
        (
            &option.modifier_id,
            &option.name,
            option.price,
            option.available as i32,
            option.sort_order,
            &option.option_id,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Option {} not found", option_id)));
    }

    broadcaster.broadcast(WsMessage::OptionUpdated(option.clone()));
    Ok(HttpResponse::Ok().json(option))
}

async fn delete_option(
    option_id: web::Path<String>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "DELETE FROM options WHERE option_id = ?1",
        [option_id.as_str()],
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Option {} not found", option_id)));
    }

    broadcaster.broadcast(WsMessage::OptionDeleted(option_id.to_string()));
    Ok(HttpResponse::NoContent().finish())
}

async fn create_discount(
    discount: web::Json<Discount>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut discount = discount.validate()?;
    discount.discount_id = Uuid::new_v4().to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    conn.execute(
        "INSERT INTO discounts (discount_id, name, is_percentage, amount, available, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &discount.discount_id,
            &discount.name,
            discount.is_percentage as i32,
            discount.amount,
            discount.available as i32,
            discount.sort_order,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    broadcaster.broadcast(WsMessage::DiscountCreated(discount.clone()));
    Ok(HttpResponse::Created().json(discount))
}

async fn list_discounts(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let mut stmt = conn.prepare(
        "SELECT discount_id, name, is_percentage, amount, available, sort_order FROM discounts",
    )?;

    let discounts = stmt
        .query_map([], |row| {
            Ok(Discount {
                discount_id: row.get(0)?,
                name: row.get(1)?,
                is_percentage: row.get::<_, i32>(2)? != 0,
                amount: row.get(3)?,
                available: row.get::<_, i32>(4)? != 0,
                sort_order: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(discounts))
}

async fn get_discount(
    discount_id: web::Path<String>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let discount = conn.query_row(
        "SELECT discount_id, name, is_percentage, amount, available, sort_order
         FROM discounts WHERE discount_id = ?1",
        [discount_id.as_str()],
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
    ).map_err(|_| ApiError::NotFound(format!("Discount {} not found", discount_id)))?;

    Ok(HttpResponse::Ok().json(discount))
}

async fn update_discount(
    discount_id: web::Path<String>,
    discount: web::Json<Discount>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let mut discount = discount.validate()?;
    discount.discount_id = discount_id.to_string();

    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "UPDATE discounts 
         SET name = ?1, is_percentage = ?2, amount = ?3, available = ?4, sort_order = ?5
         WHERE discount_id = ?6",
        (
            &discount.name,
            discount.is_percentage as i32,
            discount.amount,
            discount.available as i32,
            discount.sort_order,
            &discount.discount_id,
        ),
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Discount {} not found", discount_id)));
    }

    broadcaster.broadcast(WsMessage::DiscountUpdated(discount.clone()));
    Ok(HttpResponse::Ok().json(discount))
}

async fn delete_discount(
    discount_id: web::Path<String>,
    pool: web::Data<DbPool>,
    broadcaster: web::Data<Broadcaster>,
) -> Result<HttpResponse, ApiError> {
    let conn = pool.get().map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let rows_affected = conn.execute(
        "DELETE FROM discounts WHERE discount_id = ?1",
        [discount_id.as_str()],
    ).map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    if rows_affected == 0 {
        return Err(ApiError::NotFound(format!("Discount {} not found", discount_id)));
    }

    broadcaster.broadcast(WsMessage::DiscountDeleted(discount_id.to_string()));
    Ok(HttpResponse::NoContent().finish())
}
