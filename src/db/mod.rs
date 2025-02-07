pub mod migrations;
pub mod seed;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use log::info;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(database_url: &str) -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file(database_url);
    Pool::new(manager)
}

fn check_existing_data(conn: &rusqlite::Connection) -> bool {
    // Check if any of our main tables have data
    let tables = ["staff", "categories", "items", "modifiers", "options", "discounts"];
    
    for table in tables.iter() {
        let count: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM {}", table),
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
            
        if count > 0 {
            info!("Found existing data in {} table", table);
            return true;
        }
    }
    
    false
}

pub fn init_db() -> DbPool {
    // Ensure Database directory exists
    std::fs::create_dir_all("Database").expect("Failed to create Database directory");
    
    let db_path = Path::new("Database/pos.db").to_str().unwrap();
    let pool = create_pool(db_path).expect("Failed to create database pool");
    
    // Run migrations
    migrations::run_migrations(&pool).expect("Failed to run database migrations");
    
    // Check if we need to seed default data
    let mut conn = pool.get().expect("Failed to get database connection");
    
    if !check_existing_data(&conn) {
        info!("No existing data found. Seeding default data...");
        seed::seed_default_data(&mut conn).expect("Failed to seed default data");
        info!("Default data seeded successfully!");
    } else {
        info!("Existing data found. Skipping default data seeding.");
    }
    
    pool
}
