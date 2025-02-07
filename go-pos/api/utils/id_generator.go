package utils

import (
	"database/sql"
	"fmt"
	"math/rand"
	"strings"
	"time"
)

// generateRandomNumbers generates a 6-digit random number as string
func generateRandomNumbers() string {
	rand.Seed(time.Now().UnixNano())
	return fmt.Sprintf("%06d", rand.Intn(1000000))
}

// GenerateID creates an ID with the given prefix and 6 random numbers
func GenerateID(db *sql.DB, prefix string, table string) (string, error) {
	for i := 0; i < 100; i++ { // Try up to 100 times to generate a unique ID
		id := strings.ToLower(prefix) + generateRandomNumbers()
		
		// Check if ID exists in the specified table
		var exists bool
		var idColumn string
		switch table {
		case "staff":
			idColumn = "staff_id"
		case "categories":
			idColumn = "category_id"
		case "items":
			idColumn = "item_id"
		case "modifiers":
			idColumn = "modifier_id"
		case "options":
			idColumn = "option_id"
		case "discounts":
			idColumn = "discount_id"
		default:
			return "", fmt.Errorf("unknown table: %s", table)
		}
		
		query := fmt.Sprintf("SELECT 1 FROM %s WHERE %s = ? LIMIT 1", table, idColumn)
		err := db.QueryRow(query, id).Scan(&exists)
		
		if err == sql.ErrNoRows {
			// ID doesn't exist, we can use it
			return id, nil
		} else if err != nil {
			return "", fmt.Errorf("error checking ID existence: %v", err)
		}
		// If we get here, the ID exists, try again
	}
	
	return "", fmt.Errorf("failed to generate unique ID after 100 attempts")
}

// GenerateStaffID creates a staff ID using firstName and random numbers
func GenerateStaffID(db *sql.DB, firstName string) (string, error) {
	return GenerateID(db, firstName, "staff")
}

// GenerateItemID creates an item ID
func GenerateItemID(db *sql.DB) (string, error) {
	return GenerateID(db, "item", "items")
}

// GenerateCategoryID creates a category ID
func GenerateCategoryID(db *sql.DB) (string, error) {
	return GenerateID(db, "category", "categories")
}

// GenerateModifierID creates a modifier ID
func GenerateModifierID(db *sql.DB) (string, error) {
	return GenerateID(db, "modifier", "modifiers")
}

// GenerateOptionID creates an option ID
func GenerateOptionID(db *sql.DB) (string, error) {
	return GenerateID(db, "option", "options")
}

// GenerateDiscountID creates a discount ID
func GenerateDiscountID(db *sql.DB) (string, error) {
	return GenerateID(db, "discount", "discounts")
}
