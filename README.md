# POS Server (Go + Fiber)

A modern Point of Sale backend server built with Go and Fiber framework, featuring automatic service discovery, persistent data storage, and real-time updates.

## Features
- **Service Discovery**: Automatic server discovery using Zeroconf/mDNS
- **Persistent Storage**: SQLite database with full ACID compliance
- **Real-time Updates**: WebSocket support for live transaction updates
- **Transaction Analytics**: Daily transaction summaries and reports
- **Session Management**: Secure session-based authentication
- **RESTful API**: Complete API for all POS operations

## Project Structure
```
.
├── api/
│   ├── handlers/     # HTTP request handlers
│   ├── middleware/   # Custom middleware functions
│   ├── models/       # Data models and database schemas
│   ├── database/     # Database connection and migrations
│   └── discovery/    # Service discovery implementation
├── Database/        # SQLite database storage
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

2. Create database directory:
```bash
mkdir -p Database
```

3. Run the server:
```bash
go run main.go
```

The server will start on port 8000 by default and will be discoverable on your local network as `_pos-server._tcp.local.`

## Environment Variables
- `PORT`: Server port (default: 8000)
- `DB_PATH`: SQLite database path (default: Database/pos.db)
- `APP_ENV`: Application environment (development/production)

## Key Endpoints

### Authentication
The server uses PIN-based authentication for staff members. On first run, it creates a default admin account:
- Name: Manny
- PIN: 0000
- Role: Admin

To authenticate:
```bash
curl -X POST -H "Content-Type: application/json" \
  -d '{"pin":"0000"}' \
  http://localhost:8000/staff/auth
```

### Transactions
```
POST /transactions           # Create transaction
GET /transactions           # List transactions by date
GET /transactions/summary   # Get daily transaction summary
PUT /transactions/:id/refund # Refund transaction
```

### Real-time Updates
```
WS /ws/transactions        # WebSocket for live updates
```

## Development

### Running Tests
```bash
go test ./...
```

### Building for Production
```bash
go build -o pos-server
```

## API Documentation
For detailed API documentation, see [documentation.md](documentation.md)
