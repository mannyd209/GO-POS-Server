# POS Backend Server Documentation

## Project Structure
```
go-pos/
├── Database/               # SQLite database directory
│   └── pos.db             # Main database file
├── README.md              # Project documentation
├── api/                   # API related code
│   ├── database/          # Database management
│   │   └── database.go    # Database connection and schema
│   ├── handlers/          # Request handlers
│   │   ├── staff.go       # Staff management endpoints
│   │   ├── catalog.go     # Catalog management endpoints
│   │   ├── transaction.go # Transaction management endpoints
│   │   └── discount.go    # Discount management endpoints
│   ├── middleware/        # Middleware components
│   │   └── auth.go        # Authentication middleware
│   └── models/            # Data models
│       ├── staff.go       # Staff model and methods
│       ├── catalog.go     # Catalog models (categories, items)
│       ├── transaction.go # Transaction model and methods
│       └── discount.go    # Discount model and methods
├── config/                # Configuration
│   └── config.go         # App configuration settings
├── go.mod                # Go module definition
├── go.sum                # Go module checksums
└── main.go               # Application entry point
```

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

### Authentication
- `POST /staff/auth`
  - Authenticate staff member with PIN
  - Returns session cookie for subsequent requests

### Staff Management
- `GET /staff` - List all staff
- `POST /staff` - Create new staff
- `GET /staff/:id` - Get staff details
- `PUT /staff/:id` - Update staff
- `DELETE /staff/:id` - Delete staff

### Catalog Management
- `GET /catalog/categories` - List categories
- `POST /catalog/categories` - Create category
- `GET /catalog/categories/:id` - Get category
- `PUT /catalog/categories/:id` - Update category
- `DELETE /catalog/categories/:id` - Delete category

- `GET /catalog/items` - List items
- `POST /catalog/items` - Create item
- `GET /catalog/items/:id` - Get item
- `PUT /catalog/items/:id` - Update item
- `DELETE /catalog/items/:id` - Delete item

- `GET /catalog/modifiers/item/:itemId` - List modifiers for item
- `POST /catalog/modifiers` - Create modifier
- `GET /catalog/modifiers/:id` - Get modifier
- `PUT /catalog/modifiers/:id` - Update modifier
- `DELETE /catalog/modifiers/:id` - Delete modifier

- `GET /catalog/options/modifier/:modifierId` - List options for modifier
- `POST /catalog/options` - Create option
- `GET /catalog/options/:id` - Get option
- `PUT /catalog/options/:id` - Update option
- `DELETE /catalog/options/:id` - Delete option

### Transaction Management
- `POST /transactions` - Create new transaction
- `GET /transactions` - Get today's transactions
- `GET /transactions/range` - Get transactions by date range
- `PUT /transactions/:sale_id/refund` - Mark transaction as refunded

### Discount Management
- `GET /catalog/discounts` - List discounts
- `POST /catalog/discounts` - Create discount
- `GET /catalog/discounts/:id` - Get discount
- `PUT /catalog/discounts/:id` - Update discount
- `DELETE /catalog/discounts/:id` - Delete discount

## Key Features

1. Authentication & Authorization
   - PIN-based staff authentication
   - Session management with cookies
   - Admin-only routes protection

2. Catalog Management
   - Hierarchical organization: Categories → Items → Modifiers → Options
   - Support for regular and event pricing
   - Availability toggling for items and options
   - Sorting capabilities for all catalog items

3. Transaction System
   - Unique sale IDs (format: "sale" + 6 random digits)
   - Daily sequential order numbers (resets at midnight)
   - Support for cash and card payments
   - Detailed financial tracking (subtotal, tax, tips, fees)
   - Item tracking with options and modifiers
   - Discount application
   - Cash handling (tendered amount and change)
   - Transaction status tracking (created, refunded)

4. WebSocket Support
   - Real-time updates for customer displays
   - Live order status updates
   - Staff notification system

## Dependencies
- Fiber: Web framework
- SQLite: Database
- Gorilla WebSocket: WebSocket support
- Go-SQLite3: SQLite driver

## Security Features
1. Authentication required for all non-public endpoints
2. Session-based security with secure cookies
3. Admin-only routes for sensitive operations
4. Input validation and sanitization
5. Prepared statements for SQL queries

## Error Handling
- Consistent error response format
- Detailed logging for debugging
- Graceful error recovery
- Transaction rollback on failures

## Performance Features
1. Connection pooling for database
2. Efficient query optimization
3. Proper index usage
4. Minimal memory footprint
5. Fast response times

## Development Features
1. Hot reload support
2. Structured logging
3. Easy configuration management
4. Clear code organization
5. Modular architecture
