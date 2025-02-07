pub mod migrations;

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(database_url: &str) -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file(database_url);
    Pool::new(manager)
}

pub fn init_db() -> DbPool {
    let pool = create_pool("Database/pos.db").expect("Failed to create database pool");
    
    // Run migrations
    migrations::run_migrations(&pool).expect("Failed to run database migrations");
    
    pool
}
