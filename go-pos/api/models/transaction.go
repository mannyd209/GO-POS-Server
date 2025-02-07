package models

import (
	"database/sql"
	"fmt"
	"math/rand"
	"time"
	"go-pos/api/database"
)

type Transaction struct {
	SaleID         string    `json:"sale_id"`
	StaffID        string    `json:"staff_id"`
	OrderNumber    int       `json:"order_number"`
	PaymentMethod  string    `json:"payment_method"`
	Subtotal       float64   `json:"subtotal"`
	TaxAmount      float64   `json:"tax_amount"`
	TipAmount      float64   `json:"tip_amount"`
	CardFee        float64   `json:"card_fee"`
	TotalAmount    float64   `json:"total_amount"`
	TenderedAmount *float64  `json:"tendered_amount,omitempty"`
	ChangeAmount   *float64  `json:"change_amount,omitempty"`
	Status         string    `json:"status"`
	CreatedAt      time.Time `json:"created_at"`
	Items          []TransactionItem    `json:"items"`
	Discounts      []TransactionDiscount `json:"discounts"`
}

type TransactionItem struct {
	TransactionItemID string                  `json:"transaction_item_id"`
	SaleID           string                  `json:"sale_id"`
	ItemID           string                  `json:"item_id"`
	Item             Item                    `json:"item"`
	Quantity         int                     `json:"quantity"`
	UnitPrice        float64                 `json:"unit_price"`
	TotalPrice       float64                 `json:"total_price"`
	Options          []TransactionItemOption `json:"options"`
}

type TransactionItemOption struct {
	TransactionItemOptionID string  `json:"transaction_item_option_id"`
	TransactionItemID      string  `json:"transaction_item_id"`
	OptionID              string  `json:"option_id"`
	Option                Option  `json:"option"`
	Price                 float64 `json:"price"`
}

type TransactionDiscount struct {
	TransactionDiscountID string   `json:"transaction_discount_id"`
	SaleID               string   `json:"sale_id"`
	DiscountID           string   `json:"discount_id"`
	Discount             Discount `json:"discount"`
	Amount               float64  `json:"amount"`
}

type TransactionSummary struct {
	TotalTransactions     int     `json:"total_transactions"`
	TotalCashSales       float64 `json:"total_cash_sales"`
	TotalCardSales       float64 `json:"total_card_sales"`
	TotalTax             float64 `json:"total_tax"`
	TotalTips            float64 `json:"total_tips"`
	TotalDiscounts       float64 `json:"total_discounts"`
	TotalCardTransactions int     `json:"total_card_transactions"`
	TotalCashTransactions int     `json:"total_cash_transactions"`
	TotalGrossSales      float64 `json:"total_gross_sales"`
	TotalNetSales        float64 `json:"total_net_sales"`
}

// generateSaleID generates a unique sale ID in the format "sale" + 6 random digits
func generateSaleID() (string, error) {
	for i := 0; i < 100; i++ { // Try up to 100 times
		saleID := fmt.Sprintf("sale%06d", rand.Intn(1000000))
		
		// Check if this sale ID already exists
		var exists bool
		err := database.DB.QueryRow("SELECT EXISTS(SELECT 1 FROM transactions WHERE sale_id = ?)", saleID).Scan(&exists)
		if err != nil {
			return "", err
		}
		
		if !exists {
			return saleID, nil
		}
	}
	
	return "", fmt.Errorf("failed to generate unique sale ID after 100 attempts")
}

// getNextOrderNumber gets the next order number for the current day
func getNextOrderNumber() (int, error) {
	// Get the current date in the local timezone
	now := time.Now()
	startOfDay := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, now.Location())
	endOfDay := startOfDay.Add(24 * time.Hour)

	// Get the highest order number for today
	var maxOrderNumber sql.NullInt64
	err := database.DB.QueryRow(`
		SELECT MAX(order_number) 
		FROM transactions 
		WHERE created_at >= ? AND created_at < ?`,
		startOfDay, endOfDay,
	).Scan(&maxOrderNumber)

	if err != nil {
		return 0, err
	}

	if !maxOrderNumber.Valid {
		return 1, nil // Start with 1 if no orders today
	}

	nextOrderNumber := int(maxOrderNumber.Int64) + 1
	if nextOrderNumber > 99 {
		nextOrderNumber = 1 // Reset to 1 if we exceed 99
	}

	return nextOrderNumber, nil
}

// Create inserts a new transaction into the database
func (t *Transaction) Create() error {
	tx, err := database.DB.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Generate sale ID and order number
	t.SaleID, err = generateSaleID()
	if err != nil {
		return err
	}

	t.OrderNumber, err = getNextOrderNumber()
	if err != nil {
		return err
	}

	// Set creation time
	t.CreatedAt = time.Now()

	// Insert the transaction
	_, err = tx.Exec(`
		INSERT INTO transactions (
			sale_id, staff_id, order_number, payment_method, 
			subtotal, tax_amount, tip_amount, card_fee, 
			total_amount, tendered_amount, change_amount, status,
			created_at
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
		t.SaleID, t.StaffID, t.OrderNumber, t.PaymentMethod,
		t.Subtotal, t.TaxAmount, t.TipAmount, t.CardFee,
		t.TotalAmount, t.TenderedAmount, t.ChangeAmount, t.Status,
		t.CreatedAt,
	)
	if err != nil {
		return err
	}

	// Insert transaction items
	for i := range t.Items {
		item := &t.Items[i]
		item.TransactionItemID = fmt.Sprintf("titem%06d", rand.Intn(1000000))
		item.SaleID = t.SaleID

		_, err = tx.Exec(`
			INSERT INTO transaction_items (
				transaction_item_id, sale_id, item_id,
				quantity, unit_price, total_price
			) VALUES (?, ?, ?, ?, ?, ?)`,
			item.TransactionItemID, item.SaleID, item.ItemID,
			item.Quantity, item.UnitPrice, item.TotalPrice,
		)
		if err != nil {
			return err
		}

		// Insert item options
		for j := range item.Options {
			option := &item.Options[j]
			option.TransactionItemOptionID = fmt.Sprintf("toption%06d", rand.Intn(1000000))
			option.TransactionItemID = item.TransactionItemID

			_, err = tx.Exec(`
				INSERT INTO transaction_item_options (
					transaction_item_option_id, transaction_item_id,
					option_id, price
				) VALUES (?, ?, ?, ?)`,
				option.TransactionItemOptionID, option.TransactionItemID,
				option.OptionID, option.Price,
			)
			if err != nil {
				return err
			}
		}
	}

	// Insert transaction discounts
	for i := range t.Discounts {
		discount := &t.Discounts[i]
		discount.TransactionDiscountID = fmt.Sprintf("tdiscount%06d", rand.Intn(1000000))
		discount.SaleID = t.SaleID

		_, err = tx.Exec(`
			INSERT INTO transaction_discounts (
				transaction_discount_id, sale_id,
				discount_id, amount
			) VALUES (?, ?, ?, ?)`,
			discount.TransactionDiscountID, discount.SaleID,
			discount.DiscountID, discount.Amount,
		)
		if err != nil {
			return err
		}
	}

	return tx.Commit()
}

// GetByDateRange retrieves transactions within a date range
func GetTransactionsByDateRange(startDate, endDate time.Time) ([]Transaction, error) {
	rows, err := database.DB.Query(`
		SELECT 
			t.sale_id, t.staff_id, t.order_number, t.payment_method,
			t.subtotal, t.tax_amount, t.tip_amount, t.card_fee,
			t.total_amount, t.tendered_amount, t.change_amount,
			t.status, t.created_at
		FROM transactions t
		WHERE t.created_at >= ? AND t.created_at < ?
		ORDER BY t.created_at DESC`,
		startDate, endDate,
	)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var transactions []Transaction
	for rows.Next() {
		var t Transaction
		err := rows.Scan(
			&t.SaleID, &t.StaffID, &t.OrderNumber, &t.PaymentMethod,
			&t.Subtotal, &t.TaxAmount, &t.TipAmount, &t.CardFee,
			&t.TotalAmount, &t.TenderedAmount, &t.ChangeAmount,
			&t.Status, &t.CreatedAt,
		)
		if err != nil {
			return nil, err
		}

		// Load items with their details
		itemRows, err := database.DB.Query(`
			SELECT 
				ti.transaction_item_id, ti.sale_id, ti.item_id,
				ti.quantity, ti.unit_price, ti.total_price,
				COALESCE(i.name, ''), COALESCE(i.category_id, ''),
				COALESCE(i.regular_price, 0), COALESCE(i.event_price, 0),
				COALESCE(i.sort_order, 0), COALESCE(i.available, 0)
			FROM transaction_items ti
			LEFT JOIN items i ON ti.item_id = i.item_id
			WHERE ti.sale_id = ?`,
			t.SaleID,
		)
		if err != nil {
			return nil, err
		}
		defer itemRows.Close()

		for itemRows.Next() {
			var item TransactionItem
			err := itemRows.Scan(
				&item.TransactionItemID, &item.SaleID, &item.ItemID,
				&item.Quantity, &item.UnitPrice, &item.TotalPrice,
				&item.Item.Name, &item.Item.CategoryID, &item.Item.RegularPrice,
				&item.Item.EventPrice, &item.Item.SortOrder, &item.Item.Available,
			)
			if err != nil {
				return nil, err
			}

			// Load options for this item with their details
			optionRows, err := database.DB.Query(`
				SELECT 
					tio.transaction_item_option_id, tio.transaction_item_id,
					tio.option_id, tio.price,
					COALESCE(o.modifier_id, ''), COALESCE(o.name, ''),
					COALESCE(o.price, 0), COALESCE(o.available, 0),
					COALESCE(o.sort_order, 0)
				FROM transaction_item_options tio
				LEFT JOIN options o ON tio.option_id = o.option_id
				WHERE tio.transaction_item_id = ?`,
				item.TransactionItemID,
			)
			if err != nil {
				return nil, err
			}
			defer optionRows.Close()

			for optionRows.Next() {
				var option TransactionItemOption
				err := optionRows.Scan(
					&option.TransactionItemOptionID, &option.TransactionItemID,
					&option.OptionID, &option.Price,
					&option.Option.ModifierID, &option.Option.Name,
					&option.Option.Price, &option.Option.Available,
					&option.Option.SortOrder,
				)
				if err != nil {
					return nil, err
				}
				item.Options = append(item.Options, option)
			}

			t.Items = append(t.Items, item)
		}

		// Load discounts with their details
		discountRows, err := database.DB.Query(`
			SELECT 
				td.transaction_discount_id, td.sale_id,
				td.discount_id, td.amount,
				COALESCE(d.name, ''), COALESCE(d.is_percentage, 0),
				COALESCE(d.amount, 0), COALESCE(d.available, 0),
				COALESCE(d.sort_order, 0)
			FROM transaction_discounts td
			LEFT JOIN discounts d ON td.discount_id = d.discount_id
			WHERE td.sale_id = ?`,
			t.SaleID,
		)
		if err != nil {
			return nil, err
		}
		defer discountRows.Close()

		for discountRows.Next() {
			var discount TransactionDiscount
			err := discountRows.Scan(
				&discount.TransactionDiscountID, &discount.SaleID,
				&discount.DiscountID, &discount.Amount,
				&discount.Discount.Name, &discount.Discount.IsPercentage,
				&discount.Discount.Amount, &discount.Discount.Available,
				&discount.Discount.SortOrder,
			)
			if err != nil {
				return nil, err
			}
			t.Discounts = append(t.Discounts, discount)
		}

		transactions = append(transactions, t)
	}

	return transactions, nil
}

func GetTransactionSummary(startDate, endDate time.Time) (*TransactionSummary, error) {
	query := `
		SELECT 
			COUNT(*) as total_transactions,
			SUM(CASE WHEN payment_method = 'cash' THEN total_amount ELSE 0 END) as total_cash_sales,
			SUM(CASE WHEN payment_method != 'cash' THEN total_amount ELSE 0 END) as total_card_sales,
			SUM(tax_amount) as total_tax,
			SUM(tip_amount) as total_tips,
			COUNT(CASE WHEN payment_method = 'cash' THEN 1 END) as total_cash_transactions,
			COUNT(CASE WHEN payment_method != 'cash' THEN 1 END) as total_card_transactions,
			SUM(total_amount) as total_gross_sales,
			SUM(subtotal) as total_net_sales,
			COALESCE((
				SELECT SUM(d.amount)
				FROM transaction_discounts td
				JOIN transactions t2 ON td.sale_id = t2.sale_id
				JOIN discounts d ON td.discount_id = d.discount_id
				WHERE t2.created_at >= ? AND t2.created_at < ?
			), 0) as total_discounts
		FROM transactions
		WHERE created_at >= ? AND created_at < ?
	`

	summary := &TransactionSummary{}
	err := database.DB.QueryRow(query, startDate, endDate, startDate, endDate).Scan(
		&summary.TotalTransactions,
		&summary.TotalCashSales,
		&summary.TotalCardSales,
		&summary.TotalTax,
		&summary.TotalTips,
		&summary.TotalCashTransactions,
		&summary.TotalCardTransactions,
		&summary.TotalGrossSales,
		&summary.TotalNetSales,
		&summary.TotalDiscounts,
	)
	if err != nil {
		return nil, err
	}

	return summary, nil
}

// loadItems loads all items for a transaction
func (t *Transaction) loadItems() error {
	rows, err := database.DB.Query(`
		SELECT 
			ti.transaction_item_id, ti.sale_id, ti.item_id,
			ti.quantity, ti.unit_price, ti.total_price,
			i.name, i.regular_price, i.event_price
		FROM transaction_items ti
		JOIN items i ON ti.item_id = i.item_id
		WHERE ti.sale_id = ?`,
		t.SaleID,
	)
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var item TransactionItem
		err := rows.Scan(
			&item.TransactionItemID, &item.SaleID, &item.ItemID,
			&item.Quantity, &item.UnitPrice, &item.TotalPrice,
			&item.Item.Name, &item.Item.RegularPrice, &item.Item.EventPrice,
		)
		if err != nil {
			return err
		}

		// Load options for this item
		if err := t.loadItemOptions(&item); err != nil {
			return err
		}

		t.Items = append(t.Items, item)
	}

	return nil
}

// loadItemOptions loads all options for a transaction item
func (t *Transaction) loadItemOptions(item *TransactionItem) error {
	rows, err := database.DB.Query(`
		SELECT 
			tio.transaction_item_option_id, tio.transaction_item_id,
			tio.option_id, tio.price,
			o.name
		FROM transaction_item_options tio
		JOIN options o ON tio.option_id = o.option_id
		WHERE tio.transaction_item_id = ?`,
		item.TransactionItemID,
	)
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var option TransactionItemOption
		err := rows.Scan(
			&option.TransactionItemOptionID, &option.TransactionItemID,
			&option.OptionID, &option.Price,
			&option.Option.Name,
		)
		if err != nil {
			return err
		}

		item.Options = append(item.Options, option)
	}

	return nil
}

// loadDiscounts loads all discounts for a transaction
func (t *Transaction) loadDiscounts() error {
	rows, err := database.DB.Query(`
		SELECT 
			td.transaction_discount_id, td.sale_id,
			td.discount_id, td.amount,
			d.name, d.is_percentage
		FROM transaction_discounts td
		JOIN discounts d ON td.discount_id = d.discount_id
		WHERE td.sale_id = ?`,
		t.SaleID,
	)
	if err != nil {
		return err
	}
	defer rows.Close()

	for rows.Next() {
		var discount TransactionDiscount
		err := rows.Scan(
			&discount.TransactionDiscountID, &discount.SaleID,
			&discount.DiscountID, &discount.Amount,
			&discount.Discount.Name, &discount.Discount.IsPercentage,
		)
		if err != nil {
			return err
		}

		t.Discounts = append(t.Discounts, discount)
	}

	return nil
}

// Refund marks a transaction as refunded
func (t *Transaction) Refund() error {
	_, err := database.DB.Exec(
		"UPDATE transactions SET status = 'refunded' WHERE sale_id = ?",
		t.SaleID,
	)
	return err
}
