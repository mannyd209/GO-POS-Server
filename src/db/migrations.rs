use rusqlite::Result;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub fn run_migrations(pool: &Pool<SqliteConnectionManager>) -> Result<()> {
    let mut conn = pool.get().unwrap();
    let tx = conn.transaction()?;

    // Create version table if it doesn't exist
    tx.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Get current version
    let current_version: i32 = tx
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Apply migrations in order
    for version in (current_version + 1)..=1 {
        apply_migration(&tx, version)?;
        tx.execute("INSERT INTO schema_version (version) VALUES (?1)", [version])?;
    }

    tx.commit()?;
    Ok(())
}

fn apply_migration(tx: &rusqlite::Transaction, version: i32) -> Result<(), rusqlite::Error> {
    match version {
        1 => {
            // Initial schema
            tx.execute(
                "CREATE TABLE IF NOT EXISTS staff (
                    staff_id TEXT PRIMARY KEY,
                    pin TEXT NOT NULL,
                    first_name TEXT NOT NULL,
                    last_name TEXT NOT NULL,
                    hourly_wage REAL NOT NULL,
                    is_admin INTEGER NOT NULL DEFAULT 0
                )",
                [],
            )?;

            tx.execute(
                "CREATE TABLE IF NOT EXISTS categories (
                    category_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    sort_order INTEGER NOT NULL
                )",
                [],
            )?;

            tx.execute(
                "CREATE TABLE IF NOT EXISTS items (
                    item_id TEXT PRIMARY KEY,
                    category_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    regular_price REAL NOT NULL,
                    event_price REAL NOT NULL,
                    sort_order INTEGER NOT NULL,
                    available INTEGER NOT NULL DEFAULT 1,
                    FOREIGN KEY (category_id) REFERENCES categories (category_id)
                        ON DELETE CASCADE
                )",
                [],
            )?;

            tx.execute(
                "CREATE TABLE IF NOT EXISTS modifiers (
                    modifier_id TEXT PRIMARY KEY,
                    item_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    single_selection INTEGER NOT NULL DEFAULT 1,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    FOREIGN KEY (item_id) REFERENCES items (item_id)
                        ON DELETE CASCADE
                )",
                [],
            )?;

            tx.execute(
                "CREATE TABLE IF NOT EXISTS options (
                    option_id TEXT PRIMARY KEY,
                    modifier_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    price REAL NOT NULL,
                    available INTEGER NOT NULL DEFAULT 1,
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    FOREIGN KEY (modifier_id) REFERENCES modifiers (modifier_id)
                        ON DELETE CASCADE
                )",
                [],
            )?;

            tx.execute(
                "CREATE TABLE IF NOT EXISTS discounts (
                    discount_id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    is_percentage INTEGER NOT NULL DEFAULT 1,
                    amount REAL NOT NULL,
                    available INTEGER NOT NULL DEFAULT 1,
                    sort_order INTEGER NOT NULL DEFAULT 0
                )",
                [],
            )?;

            // Create indexes
            tx.execute("CREATE INDEX IF NOT EXISTS idx_items_category ON items (category_id)", [])?;
            tx.execute("CREATE INDEX IF NOT EXISTS idx_modifiers_item ON modifiers (item_id)", [])?;
            tx.execute("CREATE INDEX IF NOT EXISTS idx_options_modifier ON options (modifier_id)", [])?;
        }
        _ => unreachable!(),
    }
    Ok(())
}
