package handlers

import (
	"net/http"
	"time"

	"github.com/gofiber/fiber/v2"
	"pos-server/api/models"
)

// CreateTransaction creates a new transaction
func CreateTransaction(c *fiber.Ctx) error {
	var transaction models.Transaction
	if err := c.BodyParser(&transaction); err != nil {
		return c.Status(http.StatusBadRequest).JSON(fiber.Map{
			"error": "Invalid request body",
		})
	}

	// Set initial status
	transaction.Status = "created"

	// Create the transaction
	if err := transaction.Create(); err != nil {
		return c.Status(http.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to create transaction",
		})
	}

	return c.Status(http.StatusCreated).JSON(transaction)
}

// GetTransactionsForToday returns all transactions for today
func GetTransactionsForToday(c *fiber.Ctx) error {
	// Get today's date range in local timezone
	now := time.Now()
	startOfDay := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, now.Location())
	endOfDay := startOfDay.Add(24 * time.Hour)

	// Get transactions
	transactions, err := models.GetTransactionsByDateRange(startOfDay, endOfDay)
	if err != nil {
		return c.Status(http.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to get transactions",
		})
	}

	// If no transactions found, return empty array instead of null
	if transactions == nil {
		transactions = []models.Transaction{}
	}

	return c.JSON(transactions)
}

// GetTransactionsByDate gets all transactions for a specific date
func GetTransactionsByDate(c *fiber.Ctx) error {
	dateStr := c.Query("date")
	if dateStr == "" {
		dateStr = time.Now().Format("2006-01-02")
	}

	date, err := time.Parse("2006-01-02", dateStr)
	if err != nil {
		return c.Status(http.StatusBadRequest).JSON(fiber.Map{
			"error": "Invalid date format. Use YYYY-MM-DD",
		})
	}

	// Set time range for the entire day
	startDate := time.Date(date.Year(), date.Month(), date.Day(), 0, 0, 0, 0, time.Local)
	endDate := startDate.Add(24 * time.Hour)

	transactions, err := models.GetTransactionsByDateRange(startDate, endDate)
	if err != nil {
		return c.Status(http.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to get transactions",
		})
	}

	return c.JSON(transactions)
}

// GetTransactionsByDateRange gets all transactions within a date range
func GetTransactionsByDateRange(c *fiber.Ctx) error {
	startDateStr := c.Query("start_date")
	endDateStr := c.Query("end_date")

	if startDateStr == "" || endDateStr == "" {
		return c.Status(http.StatusBadRequest).JSON(fiber.Map{
			"error": "Both start_date and end_date are required",
		})
	}

	startDate, err := time.Parse("2006-01-02", startDateStr)
	if err != nil {
		return c.Status(http.StatusBadRequest).JSON(fiber.Map{
			"error": "Invalid start_date format. Use YYYY-MM-DD",
		})
	}

	endDate, err := time.Parse("2006-01-02", endDateStr)
	if err != nil {
		return c.Status(http.StatusBadRequest).JSON(fiber.Map{
			"error": "Invalid end_date format. Use YYYY-MM-DD",
		})
	}

	// Set time range
	startDate = time.Date(startDate.Year(), startDate.Month(), startDate.Day(), 0, 0, 0, 0, time.Local)
	endDate = time.Date(endDate.Year(), endDate.Month(), endDate.Day(), 0, 0, 0, 0, time.Local)
	endDate = endDate.Add(24 * time.Hour)

	transactions, err := models.GetTransactionsByDateRange(startDate, endDate)
	if err != nil {
		return c.Status(http.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to get transactions",
		})
	}

	return c.JSON(transactions)
}

// RefundTransaction marks a transaction as refunded
func RefundTransaction(c *fiber.Ctx) error {
	saleID := c.Params("sale_id")
	if saleID == "" {
		return c.Status(http.StatusBadRequest).JSON(fiber.Map{
			"error": "Sale ID is required",
		})
	}

	transaction := models.Transaction{SaleID: saleID}
	if err := transaction.Refund(); err != nil {
		return c.Status(http.StatusInternalServerError).JSON(fiber.Map{
			"error": "Failed to refund transaction",
		})
	}

	return c.JSON(fiber.Map{
		"message": "Transaction refunded successfully",
	})
}
