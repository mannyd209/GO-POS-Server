# POS Backend Server Documentation

## Overview
The POS (Point of Sale) Backend Server is a robust Go-based application that provides a complete backend solution for point of sale systems. It features automatic service discovery, real-time updates, and a comprehensive API for managing sales, inventory, staff, and more.

## Key Features
- **Local Network Discovery**: Automatic service discovery using Zeroconf/mDNS/Bonjour
- **Real-time Updates**: WebSocket support for live transaction updates
- **SQLite Database**: Lightweight, serverless database with full ACID compliance
- **RESTful API**: Complete API for all POS operations
- **Modular Architecture**: Clean separation of concerns for easy maintenance
- **Automatic Data Creation**: Default data generation for new installations

## Project Structure
```
go-pos/
├── Database/               # SQLite database directory
│   └── pos.db             # Main database file
├── README.md              # Project documentation
├── api/                   # API related code
│   ├── database/          # Database management
│   │   └── database.go    # Database connection and schema
│   ├── discovery/         # Service discovery
│   │   └── discovery.go   # Zeroconf/mDNS implementation
│   ├── handlers/          # Request handlers
│   │   ├── staff.go       # Staff management endpoints
│   │   ├── category.go    # Category management endpoints
│   │   ├── item.go        # Item management endpoints
│   │   ├── modifier.go    # Modifier management endpoints
│   │   ├── option.go      # Option management endpoints
│   │   ├── discount.go    # Discount management endpoints
│   │   └── transaction.go # Transaction management endpoints
│   ├── middleware/        # Middleware components
│   │   └── auth.go        # Authentication middleware
│   ├── models/            # Data models
│   │   ├── staff.go       # Staff model and methods
│   │   ├── category.go    # Category model and methods
│   │   ├── item.go        # Item model and methods
│   │   ├── modifier.go    # Modifier model and methods
│   │   ├── option.go      # Option model and methods
│   │   ├── discount.go    # Discount model and methods
│   │   └── transaction.go # Transaction model and methods
│   └── utils/             # Utility functions
├── config/                # Configuration
│   └── config.go         # App configuration settings
├── go.mod                # Go module definition
├── go.sum                # Go module checksums
└── main.go               # Application entry point
```

## Configuration
The server uses environment variables for configuration, with sensible defaults:

```go
type Config struct {
    Port     string // Default: "8000"
    DBPath   string // Default: "Database/pos.db"
    AppEnv   string // Default: "development"
}
```

Environment variables:
- `PORT`: Server port (default: 8000)
- `DB_PATH`: Database file path (default: Database/pos.db)
- `APP_ENV`: Application environment (default: development)

## Service Discovery

The POS server uses Zeroconf/mDNS for automatic service discovery on local networks. The service is advertised as `_pos-server._tcp` on all available network interfaces to ensure maximum discoverability.

### Service Advertisement
- Service type: `_pos-server._tcp`
- Domain: `local.`
- Port: 8000
- TXT records: `version=1.0.0`

The service is broadcast on all available network interfaces (e.g., Wi-Fi, Ethernet, localhost) to ensure clients can discover the server regardless of their network connection method. This means you may see multiple service advertisements when browsing, which is normal and expected behavior.

### Client Discovery
Clients can discover the server using standard mDNS browsing for the `_pos-server._tcp.local.` service. The server will be automatically discovered as long as the client is on the same local network.

## Database Storage

The server uses SQLite as its database engine, with the database file stored in the `Database/pos.db` location. The database is persistent across server restarts, ensuring all transaction data, staff information, and catalog items are preserved.

### Database Features
- **Persistent Storage**: All data is stored on disk and preserved across server restarts
- **ACID Compliance**: Full ACID (Atomicity, Consistency, Isolation, Durability) compliance
- **Concurrent Access**: Supports multiple simultaneous read/write operations
- **Automatic Schema**: Tables are automatically created if they don't exist

## Database Schema

### Staff Table
```sql
CREATE TABLE staff (
    staff_id TEXT PRIMARY KEY,
    pin TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    hourly_wage REAL NOT NULL,
    is_admin INTEGER NOT NULL DEFAULT 0
);
```

### Categories Table
```sql
CREATE TABLE categories (
    category_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);
```

### Items Table
```sql
CREATE TABLE items (
    item_id TEXT PRIMARY KEY,
    category_id TEXT REFERENCES categories(category_id),
    name TEXT NOT NULL,
    regular_price REAL NOT NULL,
    event_price REAL NOT NULL,
    sort_order INTEGER NOT NULL,
    available INTEGER NOT NULL DEFAULT 1
);
```

### Modifiers Table
```sql
CREATE TABLE modifiers (
    modifier_id TEXT PRIMARY KEY,
    item_id TEXT REFERENCES items(item_id),
    name TEXT NOT NULL,
    single_selection INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0
);
```

### Options Table
```sql
CREATE TABLE options (
    option_id TEXT PRIMARY KEY,
    modifier_id TEXT REFERENCES modifiers(modifier_id),
    name TEXT NOT NULL,
    price REAL NOT NULL,
    available INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0
);
```

### Discounts Table
```sql
CREATE TABLE discounts (
    discount_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    is_percentage INTEGER NOT NULL DEFAULT 1,
    amount REAL NOT NULL,
    available INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0
);
```

### Transactions Table
```sql
CREATE TABLE transactions (
    sale_id TEXT PRIMARY KEY,
    staff_id TEXT REFERENCES staff(staff_id),
    order_number INTEGER NOT NULL,
    payment_method TEXT NOT NULL,
    subtotal REAL NOT NULL,
    tax_amount REAL NOT NULL,
    tip_amount REAL NOT NULL,
    card_fee REAL NOT NULL,
    total_amount REAL NOT NULL,
    tendered_amount REAL,
    change_amount REAL,
    status TEXT NOT NULL DEFAULT 'created',
    created_at DATETIME NOT NULL
);
```

### Transaction Items Table
```sql
CREATE TABLE transaction_items (
    transaction_item_id TEXT PRIMARY KEY,
    sale_id TEXT REFERENCES transactions(sale_id),
    item_id TEXT REFERENCES items(item_id),
    quantity INTEGER NOT NULL,
    unit_price REAL NOT NULL,
    total_price REAL NOT NULL
);
```

### Transaction Item Options Table
```sql
CREATE TABLE transaction_item_options (
    transaction_item_option_id TEXT PRIMARY KEY,
    transaction_item_id TEXT REFERENCES transaction_items(transaction_item_id),
    option_id TEXT REFERENCES options(option_id),
    price REAL NOT NULL
);
```

### Transaction Discounts Table
```sql
CREATE TABLE transaction_discounts (
    transaction_discount_id TEXT PRIMARY KEY,
    sale_id TEXT REFERENCES transactions(sale_id),
    discount_id TEXT REFERENCES discounts(discount_id),
    amount REAL NOT NULL
);
```

## API Endpoints

### Staff Management
- `GET /staff` - List all staff
  - Returns: Array of staff members
  - Query params: 
    - `active`: Filter by active status (0/1)
    - `sort`: Sort field (name, id)
    - `order`: Sort order (asc/desc)

- `POST /staff` - Create new staff
  - Body: Staff object
  - Required fields: first_name, last_name, pin, hourly_wage
  - Returns: Created staff object

- `GET /staff/:id` - Get staff details
  - Returns: Staff object
  - Error: 404 if not found

- `PUT /staff/:id` - Update staff
  - Body: Staff object
  - Returns: Updated staff object
  - Error: 404 if not found

- `DELETE /staff/:id` - Delete staff
  - Returns: 204 No Content
  - Error: 404 if not found

### Catalog Management

#### Categories
- `GET /catalog/categories` - List categories
  - Returns: Array of categories
  - Query params:
    - `sort`: Sort field (name, sort_order)
    - `order`: Sort order (asc/desc)

- `POST /catalog/categories` - Create category
  - Body: Category object
  - Required fields: name, sort_order
  - Returns: Created category object

- `GET /catalog/categories/:id` - Get category
  - Returns: Category object with items
  - Error: 404 if not found

- `PUT /catalog/categories/:id` - Update category
  - Body: Category object
  - Returns: Updated category object
  - Error: 404 if not found

- `DELETE /catalog/categories/:id` - Delete category
  - Returns: 204 No Content
  - Error: 404 if not found, 400 if has items

#### Items
- `GET /catalog/items` - List items
  - Returns: Array of items
  - Query params:
    - `category`: Filter by category_id
    - `available`: Filter by availability (0/1)
    - `sort`: Sort field (name, price, sort_order)
    - `order`: Sort order (asc/desc)

- `POST /catalog/items` - Create item
  - Body: Item object
  - Required fields: name, category_id, regular_price, event_price, sort_order
  - Returns: Created item object

- `GET /catalog/items/:id` - Get item
  - Returns: Item object with modifiers
  - Error: 404 if not found

- `PUT /catalog/items/:id` - Update item
  - Body: Item object
  - Returns: Updated item object
  - Error: 404 if not found

- `DELETE /catalog/items/:id` - Delete item
  - Returns: 204 No Content
  - Error: 404 if not found

#### Modifiers
- `GET /catalog/modifiers/item/:itemId` - List modifiers for item
  - Returns: Array of modifiers with options
  - Error: 404 if item not found

- `POST /catalog/modifiers` - Create modifier
  - Body: Modifier object
  - Required fields: name, item_id, single_selection
  - Returns: Created modifier object

- `GET /catalog/modifiers/:id` - Get modifier
  - Returns: Modifier object with options
  - Error: 404 if not found

- `PUT /catalog/modifiers/:id` - Update modifier
  - Body: Modifier object
  - Returns: Updated modifier object
  - Error: 404 if not found

- `DELETE /catalog/modifiers/:id` - Delete modifier
  - Returns: 204 No Content
  - Error: 404 if not found

#### Options
- `GET /catalog/options/modifier/:modifierId` - List options for modifier
  - Returns: Array of options
  - Error: 404 if modifier not found

- `POST /catalog/options` - Create option
  - Body: Option object
  - Required fields: name, modifier_id, price
  - Returns: Created option object

- `GET /catalog/options/:id` - Get option
  - Returns: Option object
  - Error: 404 if not found

- `PUT /catalog/options/:id` - Update option
  - Body: Option object
  - Returns: Updated option object
  - Error: 404 if not found

- `DELETE /catalog/options/:id` - Delete option
  - Returns: 204 No Content
  - Error: 404 if not found

### Transaction Management
- `POST /transactions` - Create new transaction
  - Body: Transaction object with items and options
  - Required fields: staff_id, items array
  - Returns: Created transaction object

- `GET /transactions` - Get today's transactions
  - Returns: Array of transactions
  - Query params:
    - `status`: Filter by status
    - `staff_id`: Filter by staff
    - `sort`: Sort field (created_at, total_amount)
    - `order`: Sort order (asc/desc)

- `GET /transactions/range` - Get transactions by date range
  - Query params:
    - `start_date`: Start date (YYYY-MM-DD)
    - `end_date`: End date (YYYY-MM-DD)
    - `status`: Filter by status
    - `staff_id`: Filter by staff
  - Returns: Array of transactions

- `GET /transactions/summary` - Get transaction summary for a date
  - Query params:
    - `date`: Date (YYYY-MM-DD)
  - Returns: Transaction summary object

- `PUT /transactions/:sale_id/refund` - Mark transaction as refunded
  - Returns: Updated transaction object
  - Error: 404 if not found

### Transaction Summary
The server provides a comprehensive transaction summary endpoint that calculates various metrics for a given date:

```
GET /transactions/summary?date=YYYY-MM-DD
```

Response format:
```json
{
    "total_transactions": 1,
    "total_cash_sales": 32.50,
    "total_card_sales": 0.00,
    "total_tax": 2.50,
    "total_tips": 5.00,
    "total_discounts": 0.00,
    "total_card_transactions": 0,
    "total_cash_transactions": 1,
    "total_gross_sales": 32.50,
    "total_net_sales": 25.00
}
```

The summary includes:
- Transaction counts (total, cash, card)
- Sales amounts (cash, card, gross, net)
- Additional amounts (tax, tips, discounts)

### WebSocket Endpoints
- `WS /ws/transactions` - Real-time transaction updates
  - Events:
    - `transaction_created`: New transaction created
    - `transaction_updated`: Transaction status changed
    - `transaction_refunded`: Transaction refunded

### Discount Management
- `GET /catalog/discounts` - List discounts
  - Returns: Array of discounts
  - Query params:
    - `available`: Filter by availability (0/1)
    - `sort`: Sort field (name, amount)
    - `order`: Sort order (asc/desc)

- `POST /catalog/discounts` - Create discount
  - Body: Discount object
  - Required fields: name, amount, is_percentage
  - Returns: Created discount object

- `GET /catalog/discounts/:id` - Get discount
  - Returns: Discount object
  - Error: 404 if not found

- `PUT /catalog/discounts/:id` - Update discount
  - Body: Discount object
  - Returns: Updated discount object
  - Error: 404 if not found

- `DELETE /catalog/discounts/:id` - Delete discount
  - Returns: 204 No Content
  - Error: 404 if not found

## Error Handling
All API endpoints follow a consistent error response format:
```json
{
    "error": true,
    "message": "Error description",
    "code": "ERROR_CODE"
}
```

Common HTTP status codes:
- 200: Success
- 201: Created
- 204: No Content
- 400: Bad Request
- 404: Not Found
- 500: Internal Server Error

## Database Management
The server automatically:
- Creates the database file if it doesn't exist
- Creates all required tables
- Creates default data for new installations
- Handles database migrations (when implemented)

## Security Considerations
- All database queries use prepared statements to prevent SQL injection
- Input validation is performed on all API endpoints
- PIN values are hashed before storage
- CORS is enabled for cross-origin requests
- WebSocket connections are validated

## Dependencies
Major dependencies used:
- `github.com/gofiber/fiber/v2`: Fast HTTP framework
- `github.com/gofiber/websocket/v2`: WebSocket support
- `github.com/mattn/go-sqlite3`: SQLite driver
- `github.com/grandcat/zeroconf`: Service discovery

## Getting Started
1. Clone the repository
2. Install Go 1.23.6 or later
3. Run `go mod tidy` to install dependencies
4. Start the server: `go run main.go`

The server will:
1. Create the database and tables if they don't exist
2. Start broadcasting its presence on the local network
3. Listen for HTTP requests on port 8000
4. Create default data if the database is empty
