use rusqlite::Connection;
use crate::models::{Staff, Category, Item, Modifier, Option, Discount};
use ring::digest;
use data_encoding::HEXLOWER;
use crate::utils::id_generator::{generate_id, IdType};
use std::collections::HashMap;

fn hash_pin(pin: &str) -> String {
    let digest = digest::digest(&digest::SHA256, pin.as_bytes());
    HEXLOWER.encode(digest.as_ref())
}

pub fn seed_default_data(conn: &mut Connection) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;

    // Create default admin staff
    let admin_id = generate_id(IdType::Staff, &tx);
    tx.execute(
        "INSERT INTO staff (staff_id, pin, first_name, last_name, hourly_wage, is_admin) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &admin_id,
            &hash_pin("1234"), // Default admin PIN
            "Admin",
            "User",
            30.0,
            true,
        ),
    )?;

    // Create default categories
    let categories = vec![
        ("Beverages", 1),
        ("Food", 2),
        ("Desserts", 3),
    ];

    for (name, sort_order) in categories {
        tx.execute(
            "INSERT INTO categories (category_id, name, sort_order) 
             VALUES (?1, ?2, ?3)",
            (
                &generate_id(IdType::Category, &tx),
                name,
                sort_order,
            ),
        )?;
    }

    // Create default items with their IDs stored for modifiers
    let items = vec![
        ("Beverages", "Coffee", 2.50, 2.00, 1, true),
        ("Beverages", "Tea", 2.00, 1.50, 2, true),
        ("Food", "Sandwich", 8.00, 7.00, 1, true),
        ("Desserts", "Cake", 5.00, 4.00, 1, true),
    ];

    let mut item_ids = HashMap::new();

    for (category_name, name, regular_price, event_price, sort_order, available) in items {
        let category_id: String = tx.query_row(
            "SELECT category_id FROM categories WHERE name = ?1",
            [category_name],
            |row| row.get(0),
        )?;

        let item_id = generate_id(IdType::Item, &tx);
        tx.execute(
            "INSERT INTO items (item_id, category_id, name, regular_price, event_price, sort_order, available) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &item_id,
                &category_id,
                name,
                regular_price,
                event_price,
                sort_order,
                available as i32,
            ),
        )?;
        
        item_ids.insert(name, item_id);
    }

    // Create default modifiers and options
    let modifiers = vec![
        // Coffee modifiers
        ("Coffee", "Size", true, 1, vec![
            ("Small", 0.0),
            ("Medium", 0.50),
            ("Large", 1.00),
        ]),
        ("Coffee", "Extra Shot", true, 2, vec![
            ("None", 0.0),
            ("Single Shot", 0.75),
            ("Double Shot", 1.50),
        ]),
        ("Coffee", "Milk Type", true, 3, vec![
            ("Regular", 0.0),
            ("Soy", 0.50),
            ("Almond", 0.50),
            ("Oat", 0.50),
        ]),

        // Sandwich modifiers
        ("Sandwich", "Bread Type", true, 1, vec![
            ("White", 0.0),
            ("Wheat", 0.0),
            ("Sourdough", 0.0),
            ("Rye", 0.50),
        ]),
        ("Sandwich", "Extras", false, 2, vec![
            ("Cheese", 1.00),
            ("Avocado", 1.50),
            ("Bacon", 2.00),
            ("Extra Meat", 2.50),
        ]),
        
        // Cake modifiers
        ("Cake", "Size", true, 1, vec![
            ("Slice", 0.0),
            ("Double Slice", 4.00),
            ("Whole Cake", 25.00),
        ]),
    ];

    for (item_name, modifier_name, single_selection, sort_order, options) in modifiers {
        if let Some(item_id) = item_ids.get(item_name) {
            let modifier_id = generate_id(IdType::Modifier, &tx);
            
            tx.execute(
                "INSERT INTO modifiers (modifier_id, item_id, name, single_selection, sort_order) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    &modifier_id,
                    item_id,
                    modifier_name,
                    single_selection as i32,
                    sort_order,
                ),
            )?;

            // Add options for this modifier
            for (i, (option_name, price)) in options.into_iter().enumerate() {
                tx.execute(
                    "INSERT INTO options (option_id, modifier_id, name, price, available, sort_order) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    (
                        &generate_id(IdType::Option, &tx),
                        &modifier_id,
                        option_name,
                        price,
                        1, // available
                        i as i32, // sort_order
                    ),
                )?;
            }
        }
    }

    // Create default discounts
    let discounts = vec![
        ("Member Discount", true, 10.0, true, 1),
        ("Happy Hour", true, 15.0, true, 2),
        ("Employee Discount", true, 20.0, true, 3),
    ];

    for (name, is_percentage, amount, available, sort_order) in discounts {
        tx.execute(
            "INSERT INTO discounts (discount_id, name, is_percentage, amount, available, sort_order) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &generate_id(IdType::Discount, &tx),
                name,
                is_percentage as i32,
                amount,
                available as i32,
                sort_order,
            ),
        )?;
    }

    tx.commit()?;
    Ok(())
}
