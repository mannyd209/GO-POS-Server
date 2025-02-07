# POS Server (Go + Fiber)

A modern Point of Sale backend server built with Go and Fiber framework.

## Project Structure
```
.
├── api/
│   ├── handlers/     # HTTP request handlers
│   ├── middleware/   # Custom middleware functions
│   ├── models/       # Data models and database schemas
│   └── database/     # Database connection and migrations
├── config/          # Application configuration
├── utils/           # Utility functions and helpers
├── main.go         # Application entry point
└── README.md       # This file
```

## Requirements
- Go 1.21 or higher
- SQLite3

## Getting Started

1. Install dependencies:
```bash
go mod download
```

2. Run the server:
```bash
go run main.go
```

The server will start on port 3000 by default.

## Environment Variables
- `PORT`: Server port (default: 3000)
- `DB_PATH`: SQLite database path (default: Database/pos.db)
- `JWT_KEY`: JWT signing key for authentication
- `APP_ENV`: Application environment (development/production)
