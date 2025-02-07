use pos_backend::db;
use rusqlite::Connection;
use std::path::Path;

fn print_item_details(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(
        "SELECT 
            i.item_id,
            i.name as item_name,
            i.regular_price,
            i.event_price,
            i.available,
            c.name as category_name,
            m.name as modifier_name,
            m.single_selection,
            o.name as option_name,
            o.price as option_price,
            o.available as option_available
         FROM items i
         LEFT JOIN categories c ON i.category_id = c.category_id
         LEFT JOIN modifiers m ON m.item_id = i.item_id
         LEFT JOIN options o ON o.modifier_id = m.modifier_id
         ORDER BY c.sort_order, i.sort_order, m.sort_order, o.sort_order"
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?, // item_id
            row.get::<_, String>(1)?, // item_name
            row.get::<_, f64>(2)?,    // regular_price
            row.get::<_, f64>(3)?,    // event_price
            row.get::<_, bool>(4)?,   // available
            row.get::<_, String>(5)?, // category_name
            row.get::<_, Option<String>>(6)?, // modifier_name
            row.get::<_, Option<bool>>(7)?,   // single_selection
            row.get::<_, Option<String>>(8)?, // option_name
            row.get::<_, Option<f64>>(9)?,    // option_price
            row.get::<_, Option<bool>>(10)?,  // option_available
        ))
    })?;

    let mut current_item = String::new();
    let mut current_modifier = String::new();

    println!("\n=== Current Items in Database ===\n");

    for row in rows {
        let (
            item_id,
            item_name,
            regular_price,
            event_price,
            available,
            category_name,
            modifier_name,
            single_selection,
            option_name,
            option_price,
            option_available,
        ) = row?;

        if item_id != current_item {
            println!("\nItem: {} ({})", item_name, item_id);
            println!("Category: {}", category_name);
            println!("Regular Price: ${:.2}", regular_price);
            println!("Event Price: ${:.2}", event_price);
            println!("Available: {}", if available { "Yes" } else { "No" });
            current_item = item_id;
            current_modifier = String::new();
        }

        if let Some(mod_name) = modifier_name {
            if mod_name != current_modifier {
                println!("\n  Modifier: {} ({})", 
                    mod_name,
                    if single_selection.unwrap_or(true) { "Single Selection" } else { "Multi Selection" }
                );
                current_modifier = mod_name;
            }

            if let Some(opt_name) = option_name {
                println!("    Option: {} (${:.2}) {}", 
                    opt_name,
                    option_price.unwrap_or(0.0),
                    if option_available.unwrap_or(true) { "" } else { "[Unavailable]" }
                );
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = Path::new("Database/pos.db");
    if !db_path.exists() {
        println!("Database file not found at: {}", db_path.display());
        return Ok(());
    }

    let conn = Connection::open(db_path)?;
    print_item_details(&conn)?;

    Ok(())
}
