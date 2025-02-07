package main

import (
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/mannyd209/RUST-POS-Server/api/database"
	"github.com/mannyd209/RUST-POS-Server/api/discovery"
	"github.com/mannyd209/RUST-POS-Server/api/handlers"
	"github.com/mannyd209/RUST-POS-Server/config"
)

func createDefaultData() error {
	// Create default staff members
	if err := createDefaultStaff(); err != nil {
		return err
	}

	// Create default categories
	if err := createDefaultCategories(); err != nil {
		return err
	}

	// Create default items
	if err := createDefaultItems(); err != nil {
		return err
	}

	// Create default modifiers
	modifiers := []models.Modifier{
		// Coffee modifiers (sort order 1-2)
		{
			ItemID:          getItemID("Coffee"),
			Name:           "Size",
			SingleSelection: true,
			SortOrder:      1,
		},
		{
			ItemID:          getItemID("Coffee"),
			Name:           "Add-ins",
			SingleSelection: false,
			SortOrder:      2,
		},
		// Burger modifiers (sort order 1-2)
		{
			ItemID:          getItemID("Burger"),
			Name:           "Temperature",
			SingleSelection: true,
			SortOrder:      1,
		},
		{
			ItemID:          getItemID("Burger"),
			Name:           "Toppings",
			SingleSelection: false,
			SortOrder:      2,
		},
		// Pizza modifiers (sort order 1-2)
		{
			ItemID:          getItemID("Pizza"),
			Name:           "Size",
			SingleSelection: true,
			SortOrder:      1,
		},
		{
			ItemID:          getItemID("Pizza"),
			Name:           "Toppings",
			SingleSelection: false,
			SortOrder:      2,
		},
	}

	modifierMap := make(map[string]string) // Store modifier IDs
	for _, mod := range modifiers {
		if err := mod.Create(); err != nil {
			return err
		}
		modifierMap[mod.ItemID+"_"+mod.Name] = mod.ModifierID
	}

	// Create default options
	options := []models.Option{
		// Coffee sizes (sort order 1-3)
		{
			ModifierID: modifierMap[getItemID("Coffee")+"_Size"],
			Name:      "Small",
			Price:     0.00,
			Available: true,
			SortOrder: 1,
		},
		{
			ModifierID: modifierMap[getItemID("Coffee")+"_Size"],
			Name:      "Medium",
			Price:     0.50,
			Available: true,
			SortOrder: 2,
		},
		{
			ModifierID: modifierMap[getItemID("Coffee")+"_Size"],
			Name:      "Large",
			Price:     1.00,
			Available: true,
			SortOrder: 3,
		},
		// Coffee add-ins (sort order 1-2)
		{
			ModifierID: modifierMap[getItemID("Coffee")+"_Add-ins"],
			Name:      "Extra Shot",
			Price:     0.75,
			Available: true,
			SortOrder: 1,
		},
		{
			ModifierID: modifierMap[getItemID("Coffee")+"_Add-ins"],
			Name:      "Vanilla Syrup",
			Price:     0.50,
			Available: true,
			SortOrder: 2,
		},
		// Burger temperature (sort order 1-3)
		{
			ModifierID: modifierMap[getItemID("Burger")+"_Temperature"],
			Name:      "Medium Rare",
			Price:     0.00,
			Available: true,
			SortOrder: 1,
		},
		{
			ModifierID: modifierMap[getItemID("Burger")+"_Temperature"],
			Name:      "Medium",
			Price:     0.00,
			Available: true,
			SortOrder: 2,
		},
		{
			ModifierID: modifierMap[getItemID("Burger")+"_Temperature"],
			Name:      "Well Done",
			Price:     0.00,
			Available: true,
			SortOrder: 3,
		},
		// Burger toppings (sort order 1-2)
		{
			ModifierID: modifierMap[getItemID("Burger")+"_Toppings"],
			Name:      "Cheese",
			Price:     1.00,
			Available: true,
			SortOrder: 1,
		},
		{
			ModifierID: modifierMap[getItemID("Burger")+"_Toppings"],
			Name:      "Bacon",
			Price:     2.00,
			Available: true,
			SortOrder: 2,
		},
		// Pizza sizes (sort order 1-2)
		{
			ModifierID: modifierMap[getItemID("Pizza")+"_Size"],
			Name:      "Medium",
			Price:     0.00,
			Available: true,
			SortOrder: 1,
		},
		{
			ModifierID: modifierMap[getItemID("Pizza")+"_Size"],
			Name:      "Large",
			Price:     4.00,
			Available: true,
			SortOrder: 2,
		},
		// Pizza toppings (sort order 1-3)
		{
			ModifierID: modifierMap[getItemID("Pizza")+"_Toppings"],
			Name:      "Extra Cheese",
			Price:     2.00,
			Available: true,
			SortOrder: 1,
		},
		{
			ModifierID: modifierMap[getItemID("Pizza")+"_Toppings"],
			Name:      "Pepperoni",
			Price:     2.00,
			Available: true,
			SortOrder: 2,
		},
		{
			ModifierID: modifierMap[getItemID("Pizza")+"_Toppings"],
			Name:      "Mushrooms",
			Price:     1.50,
			Available: true,
			SortOrder: 3,
		},
	}

	for _, option := range options {
		if err := option.Create(); err != nil {
			return err
		}
	}

	// Create default discounts (sort order 1-3)
	discounts := []models.Discount{
		{
			Name:         "Happy Hour",
			IsPercentage: true,
			Amount:      20.00, // 20% off
			Available:   true,
			SortOrder:   1,
		},
		{
			Name:         "Employee",
			IsPercentage: true,
			Amount:      30.00, // 30% off
			Available:   true,
			SortOrder:   2,
		},
		{
			Name:         "$5 Off",
			IsPercentage: false,
			Amount:      5.00, // $5 off
			Available:   true,
			SortOrder:   3,
		},
	}

	for _, discount := range discounts {
		if err := discount.Create(); err != nil {
			return err
		}
	}

	return nil
}

func createDefaultCategories() error {
	categories := []models.Category{
		{Name: "Drinks", SortOrder: 1},
		{Name: "Food", SortOrder: 2},
		{Name: "Desserts", SortOrder: 3},
	}

	for _, category := range categories {
		if err := category.Create(); err != nil {
			return err
		}
	}
	return nil
}

func createDefaultItems() error {
	// Get categories first
	categories, err := models.GetAllCategories()
	if err != nil {
		return err
	}

	var drinksCategoryID, foodCategoryID, dessertsCategoryID string
	for _, category := range categories {
		switch category.Name {
		case "Drinks":
			drinksCategoryID = category.CategoryID
		case "Food":
			foodCategoryID = category.CategoryID
		case "Desserts":
			dessertsCategoryID = category.CategoryID
		}
	}

	items := []models.Item{
		// Drinks (sort order 1-2)
		{
			CategoryID:   drinksCategoryID,
			Name:        "Coffee",
			RegularPrice: 3.50,
			EventPrice:   4.00,
			SortOrder:   1,
			Available:   true,
		},
		{
			CategoryID:   drinksCategoryID,
			Name:        "Iced Tea",
			RegularPrice: 3.00,
			EventPrice:   3.50,
			SortOrder:   2,
			Available:   true,
		},
		// Food (sort order 1-2)
		{
			CategoryID:   foodCategoryID,
			Name:        "Burger",
			RegularPrice: 12.00,
			EventPrice:   14.00,
			SortOrder:   1,
			Available:   true,
		},
		{
			CategoryID:   foodCategoryID,
			Name:        "Pizza",
			RegularPrice: 15.00,
			EventPrice:   17.00,
			SortOrder:   2,
			Available:   true,
		},
		// Desserts (sort order 1)
		{
			CategoryID:   dessertsCategoryID,
			Name:        "Ice Cream",
			RegularPrice: 5.00,
			EventPrice:   6.00,
			SortOrder:   1,
			Available:   true,
		},
	}

	for _, item := range items {
		if err := item.Create(); err != nil {
			return err
		}
	}
	return nil
}

func createDefaultStaff() error {
	defaultAdmin := models.Staff{
		FirstName:  "Manny",
		LastName:   "Duarte",
		PIN:        "0000",
		HourlyWage: 30.00,
		IsAdmin:    true,
	}
	if err := defaultAdmin.Create(); err != nil {
		return fmt.Errorf("error creating default admin: %v", err)
	}

	defaultStaff := models.Staff{
		FirstName:  "Kim",
		LastName:   "Duarte",
		PIN:        "1111",
		HourlyWage: 20.00,
		IsAdmin:    false,
	}
	if err := defaultStaff.Create(); err != nil {
		return fmt.Errorf("error creating default staff: %v", err)
	}

	return nil
}

func getItemID(name string) string {
	items, err := models.GetAllItems()
	if err != nil {
		log.Fatal(err)
	}
	for _, item := range items {
		if item.Name == name {
			return item.ItemID
		}
	}
	return ""
}

func isDatabaseEmpty() (bool, error) {
	tables := []string{"staff", "categories", "items", "modifiers", "options", "discounts"}
	
	for _, table := range tables {
		var count int
		err := database.DB.QueryRow("SELECT COUNT(*) FROM " + table).Scan(&count)
		if err != nil {
			return false, err
		}
		if count > 0 {
			return false, nil
		}
	}
	
	return true, nil
}

func setupRoutes(app *fiber.App) {
	// Health check route
	app.Get("/health", func(c *fiber.Ctx) error {
		return c.JSON(fiber.Map{
			"status": "healthy",
			"time":   c.Context().Time(),
		})
	})

	// Staff routes
	staff := app.Group("/staff")
	staff.Post("/auth", handlers.HandleStaffLogin)
	
	// Protected staff routes
	protected := staff.Use(middleware.Protected())
	protected.Get("/", middleware.AdminOnly(), handlers.HandleGetAllStaff)
	protected.Post("/", middleware.AdminOnly(), handlers.HandleCreateStaff)
	protected.Get("/:id", handlers.HandleGetStaff)
	protected.Put("/:id", middleware.AdminOnly(), handlers.HandleUpdateStaff)
	protected.Delete("/:id", middleware.AdminOnly(), handlers.HandleDeleteStaff)

	// Catalog routes (all protected and admin-only)
	catalog := app.Group("/catalog", middleware.Protected(), middleware.AdminOnly())
	
	// Category routes
	categories := catalog.Group("/categories")
	categories.Post("/", handlers.HandleCreateCategory)
	categories.Get("/", handlers.HandleGetAllCategories)
	categories.Get("/:id", handlers.HandleGetCategory)
	categories.Put("/:id", handlers.HandleUpdateCategory)
	categories.Delete("/:id", handlers.HandleDeleteCategory)

	// Item routes
	items := catalog.Group("/items")
	items.Post("/", handlers.HandleCreateItem)
	items.Get("/", handlers.HandleGetAllItems)
	items.Get("/category/:categoryId", handlers.HandleGetItemsByCategory)
	items.Get("/:id", handlers.HandleGetItem)
	items.Put("/:id", handlers.HandleUpdateItem)
	items.Delete("/:id", handlers.HandleDeleteItem)

	// Modifier routes
	modifiers := catalog.Group("/modifiers")
	modifiers.Post("/", handlers.HandleCreateModifier)
	modifiers.Get("/item/:itemId", handlers.HandleGetModifiersByItem)
	modifiers.Get("/:id", handlers.HandleGetModifier)
	modifiers.Put("/:id", handlers.HandleUpdateModifier)
	modifiers.Delete("/:id", handlers.HandleDeleteModifier)

	// Option routes
	options := catalog.Group("/options")
	options.Post("/", handlers.HandleCreateOption)
	options.Get("/modifier/:modifierId", handlers.HandleGetOptionsByModifier)
	options.Get("/:id", handlers.HandleGetOption)
	options.Put("/:id", handlers.HandleUpdateOption)
	options.Delete("/:id", handlers.HandleDeleteOption)

	// Discount routes
	discounts := catalog.Group("/discounts")
	discounts.Post("/", handlers.HandleCreateDiscount)
	discounts.Get("/", handlers.HandleGetAllDiscounts)
	discounts.Get("/:id", handlers.HandleGetDiscount)
	discounts.Put("/:id", handlers.HandleUpdateDiscount)
	discounts.Delete("/:id", handlers.HandleDeleteDiscount)

	// Transaction routes
	app.Post("/transactions", handlers.CreateTransaction)
	app.Get("/transactions", handlers.GetTransactionsByDate)
	app.Get("/transactions/range", handlers.GetTransactionsByDateRange)
	app.Put("/transactions/:sale_id/refund", handlers.RefundTransaction)

	// WebSocket endpoint for real-time updates
	app.Use("/ws", func(c *fiber.Ctx) error {
		if websocket.IsWebSocketUpgrade(c) {
			c.Locals("allowed", true)
			return c.Next()
		}
		return fiber.ErrUpgradeRequired
	})

	app.Get("/ws", websocket.New(func(c *websocket.Conn) {
		// WebSocket connection handler
		var (
			msg []byte
			err error
		)
		for {
			if _, msg, err = c.ReadMessage(); err != nil {
				break
			}
			if err = c.WriteMessage(websocket.TextMessage, msg); err != nil {
				break
			}
		}
	}))
}

func main() {
	// Load configuration
	cfg := config.New()

	// Initialize database
	if err := database.InitDB(cfg.DBPath); err != nil {
		log.Fatal(err)
	}
	defer database.DB.Close()

	// Create new Fiber instance
	app := fiber.New()

	// Enable CORS for all routes
	app.Use(cors.New(cors.Config{
		AllowOrigins:     "*",
		AllowMethods:     "GET,POST,PUT,DELETE",
		AllowHeaders:     "Origin, Content-Type, Accept",
		AllowCredentials: true,
	}))

	// Setup routes
	handlers.SetupRoutes(app)

	// Create discovery server
	discoveryServer := discovery.NewServer(8000)
	if err := discoveryServer.Start(); err != nil {
		log.Printf("Warning: Failed to start discovery server: %v", err)
	}
	defer discoveryServer.Stop()

	// Check if database is empty and create default data if needed
	isEmpty, err := isDatabaseEmpty()
	if err != nil {
		log.Printf("Error checking if database is empty: %v", err)
	} else if isEmpty {
		log.Println("Creating default data...")
		if err := createDefaultData(); err != nil {
			log.Printf("Error creating default data: %v", err)
		}
	} else {
		log.Println("Database already contains data, skipping default data creation")
	}

	// Handle graceful shutdown
	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)

	go func() {
		<-c
		log.Println("Gracefully shutting down...")
		_ = app.Shutdown()
	}()

	// Start server
	log.Printf("Server starting on :8000")
	if err := app.Listen(":8000"); err != nil {
		log.Fatal(err)
	}
}
