# POS Backend

A high-performance Point of Sale (POS) backend server built with Rust, featuring real-time updates via WebSocket and a robust SQLite database.

## Features

- 🚀 High-performance Rust implementation
- 🔒 Secure authentication and authorization
- 📱 Real-time updates via WebSocket
- 🗄️ SQLite database with connection pooling
- 🔍 Comprehensive input validation
- 📝 Detailed logging and error handling
- 🧪 Extensive test coverage
- 🔄 Automatic mDNS service discovery
- 💻 Cross-platform compatibility

## Prerequisites

- Rust (2021 edition)
- Cargo package manager
- SQLite 3

## Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/pos-backend.git
   cd pos-backend
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the server:
   ```bash
   cargo run
   ```

The server will start on port 8000 by default, and a default admin account will be created:
- PIN: "1432"
- Name: "Manny Duarte"
- Admin: true

## Project Structure

```
/POS
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library exports
│   ├── db/               # Database management
│   ├── handlers/         # API route handlers
│   ├── middleware/       # Custom middleware
│   ├── models/           # Data models
│   ├── utils/            # Utility functions
│   └── websocket/        # WebSocket implementation
└── tests/                # Integration tests
```

## API Endpoints

### Health Check
- GET /health - Server health status

### Staff Management
- POST /staff/auth - Authenticate staff
- GET /staff - List all staff
- POST /staff - Create staff
- GET /staff/{id} - Get staff details
- PUT /staff/{id} - Update staff
- DELETE /staff/{id} - Delete staff

### Catalog Management (Admin Only)
- Categories: CRUD operations at /catalog/categories
- Items: CRUD operations at /catalog/items
- Modifiers: CRUD operations at /catalog/modifiers
- Options: CRUD operations at /catalog/options
- Discounts: CRUD operations at /catalog/discounts

## WebSocket Events

The server broadcasts real-time updates for the following events:

### Staff Events
- STAFF_CREATED
- STAFF_UPDATED
- STAFF_DELETED

### Catalog Events
- CATEGORY_CREATED/UPDATED/DELETED
- ITEM_CREATED/UPDATED/DELETED
- MODIFIER_CREATED/UPDATED/DELETED
- OPTION_CREATED/UPDATED/DELETED
- DISCOUNT_CREATED/UPDATED/DELETED

## Dependencies

- actix-web (4.3) - Web framework
- actix-ws (0.2) - WebSocket support
- rusqlite (0.29) - SQLite database driver
- r2d2 (0.8) - Connection pooling
- serde (1.0) - Serialization
- tokio (1.28) - Async runtime
- uuid (1.3) - UUID generation
- mdns-sd (0.10) - mDNS service discovery

## Development

### Running Tests
```bash
cargo test
```

### Running with Logging
```bash
RUST_LOG=debug cargo run
```

### Building for Release
```bash
cargo build --release
```

## Security Features

- Admin authorization middleware
- PIN-based authentication
- Input validation on all endpoints
- Parameterized SQL queries
- Rate limiting for authentication attempts
- Secure WebSocket communication

## Database Schema

The SQLite database includes tables for:
- Staff members
- Categories
- Items
- Modifiers
- Options
- Discounts

Each table includes appropriate foreign key constraints and indexes for optimal performance.

## Contributing

1. Fork the repository
2. Create your feature branch
3. Write tests for new features
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support, please open an issue in the GitHub repository or contact the development team.
