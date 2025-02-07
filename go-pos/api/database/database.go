package database

import (
	"database/sql"
	"fmt"
	"log"
	"os"
	"path/filepath"

	_ "github.com/mattn/go-sqlite3"
)

var DB *sql.DB

func InitDB(dbPath string) error {
	// Ensure database directory exists
	dbDir := filepath.Dir(dbPath)
	if err := os.MkdirAll(dbDir, 0755); err != nil {
		return fmt.Errorf("failed to create database directory: %v", err)
	}

	var err error
	DB, err = sql.Open("sqlite3", dbPath)
	if err != nil {
		return fmt.Errorf("failed to open database: %v", err)
	}

	if err = DB.Ping(); err != nil {
		return fmt.Errorf("failed to ping database: %v", err)
	}

	fmt.Println("Connected to database:", dbPath)
	createTables()
	return nil
}

func createTables() {
	createStaffTable := `
	CREATE TABLE IF NOT EXISTS staff (
		staff_id TEXT PRIMARY KEY,
		pin TEXT NOT NULL,
		first_name TEXT NOT NULL,
		last_name TEXT NOT NULL,
		hourly_wage REAL NOT NULL,
		is_admin INTEGER NOT NULL DEFAULT 0
	);`

	createCategoriesTable := `
	CREATE TABLE IF NOT EXISTS categories (
		category_id TEXT PRIMARY KEY,
		name TEXT NOT NULL,
		sort_order INTEGER NOT NULL
	);`

	createItemsTable := `
	CREATE TABLE IF NOT EXISTS items (
		item_id TEXT PRIMARY KEY,
		category_id TEXT NOT NULL,
		name TEXT NOT NULL,
		regular_price REAL NOT NULL,
		event_price REAL NOT NULL,
		sort_order INTEGER NOT NULL,
		available INTEGER NOT NULL DEFAULT 1,
		FOREIGN KEY (category_id) REFERENCES categories(category_id)
	);`

	createModifiersTable := `
	CREATE TABLE IF NOT EXISTS modifiers (
		modifier_id TEXT PRIMARY KEY,
		item_id TEXT NOT NULL,
		name TEXT NOT NULL,
		single_selection INTEGER NOT NULL DEFAULT 1,
		sort_order INTEGER NOT NULL DEFAULT 0,
		FOREIGN KEY (item_id) REFERENCES items(item_id)
	);`

	createOptionsTable := `
	CREATE TABLE IF NOT EXISTS options (
		option_id TEXT PRIMARY KEY,
		modifier_id TEXT NOT NULL,
		name TEXT NOT NULL,
		price REAL NOT NULL,
		available INTEGER NOT NULL DEFAULT 1,
		sort_order INTEGER NOT NULL DEFAULT 0,
		FOREIGN KEY (modifier_id) REFERENCES modifiers(modifier_id)
	);`

	createDiscountsTable := `
	CREATE TABLE IF NOT EXISTS discounts (
		discount_id TEXT PRIMARY KEY,
		name TEXT NOT NULL,
		is_percentage INTEGER NOT NULL DEFAULT 1,
		amount REAL NOT NULL,
		available INTEGER NOT NULL DEFAULT 1,
		sort_order INTEGER NOT NULL DEFAULT 0
	);`

	createTransactionsTable := `
	CREATE TABLE IF NOT EXISTS transactions (
		sale_id TEXT PRIMARY KEY,
		staff_id TEXT NOT NULL,
		order_number INTEGER NOT NULL,
		payment_method TEXT NOT NULL CHECK(payment_method IN ('cash', 'card')),
		subtotal REAL NOT NULL,
		tax_amount REAL NOT NULL,
		tip_amount REAL NOT NULL DEFAULT 0,
		card_fee REAL NOT NULL DEFAULT 0,
		total_amount REAL NOT NULL,
		tendered_amount REAL,
		change_amount REAL,
		status TEXT NOT NULL CHECK(status IN ('created', 'refunded')) DEFAULT 'created',
		created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
		FOREIGN KEY (staff_id) REFERENCES staff(staff_id)
	);`

	createTransactionItemsTable := `
	CREATE TABLE IF NOT EXISTS transaction_items (
		transaction_item_id TEXT PRIMARY KEY,
		sale_id TEXT NOT NULL,
		item_id TEXT NOT NULL,
		quantity INTEGER NOT NULL,
		unit_price REAL NOT NULL,
		total_price REAL NOT NULL,
		FOREIGN KEY (sale_id) REFERENCES transactions(sale_id),
		FOREIGN KEY (item_id) REFERENCES items(item_id)
	);`

	createTransactionItemOptionsTable := `
	CREATE TABLE IF NOT EXISTS transaction_item_options (
		transaction_item_option_id TEXT PRIMARY KEY,
		transaction_item_id TEXT NOT NULL,
		option_id TEXT NOT NULL,
		price REAL NOT NULL,
		FOREIGN KEY (transaction_item_id) REFERENCES transaction_items(transaction_item_id),
		FOREIGN KEY (option_id) REFERENCES options(option_id)
	);`

	createTransactionDiscountsTable := `
	CREATE TABLE IF NOT EXISTS transaction_discounts (
		transaction_discount_id TEXT PRIMARY KEY,
		sale_id TEXT NOT NULL,
		discount_id TEXT NOT NULL,
		amount REAL NOT NULL,
		FOREIGN KEY (sale_id) REFERENCES transactions(sale_id),
		FOREIGN KEY (discount_id) REFERENCES discounts(discount_id)
	);`

	// Create indexes for better query performance
	createIndexes := []string{
		"CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions(created_at);",
		"CREATE INDEX IF NOT EXISTS idx_transactions_staff_id ON transactions(staff_id);",
		"CREATE INDEX IF NOT EXISTS idx_transaction_items_sale_id ON transaction_items(sale_id);",
		"CREATE INDEX IF NOT EXISTS idx_transaction_item_options_transaction_item_id ON transaction_item_options(transaction_item_id);",
		"CREATE INDEX IF NOT EXISTS idx_transaction_discounts_sale_id ON transaction_discounts(sale_id);",
	}

	// Execute the table creation
	tables := []string{
		createStaffTable,
		createCategoriesTable,
		createItemsTable,
		createModifiersTable,
		createOptionsTable,
		createDiscountsTable,
		createTransactionsTable,
		createTransactionItemsTable,
		createTransactionItemOptionsTable,
		createTransactionDiscountsTable,
	}

	for _, table := range tables {
		if _, err := DB.Exec(table); err != nil {
			log.Fatal(err)
		}
	}

	// Create indexes
	for _, index := range createIndexes {
		if _, err := DB.Exec(index); err != nil {
			log.Fatal(err)
		}
	}

	fmt.Println("Database tables created successfully")
}
