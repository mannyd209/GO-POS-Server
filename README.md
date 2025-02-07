# POS System

A modern Point of Sale (POS) system built with Go and SQLite, featuring real-time updates and iOS admin interface.

## Features

- PIN-based staff authentication
- Real-time order updates via WebSocket
- Local network service discovery
- Catalog management (categories, items, modifiers)
- Transaction processing and history
- Discount management
- iOS admin dashboard

## Project Structure

```
/POS
├── .gitignore              # Git ignore rules
├── README.md              # Project documentation
├── Database/             # SQLite database directory
│   └── pos.db           # Main database file
└── go-pos/              # Go backend server
    ├── api/             # API related code
    │   ├── database/    # Database management
    │   ├── discovery/   # Network discovery service
    │   ├── handlers/    # Request handlers
    │   ├── middleware/  # Middleware components
    │   └── models/      # Data models
    ├── config/          # Configuration
    ├── go.mod          # Go module definition
    └── main.go         # Application entry point
```

## Getting Started

1. Install Go 1.21 or later
2. Clone this repository
3. Navigate to the go-pos directory
4. Run `go mod download` to install dependencies
5. Start the server with `go run main.go`

The server will automatically:
- Create the SQLite database if it doesn't exist
- Start broadcasting its presence on the local network
- Listen for connections on port 8000

## iOS Admin App

The iOS admin dashboard app will automatically discover the POS server when:
1. Both devices are on the same local network
2. The POS server is running
3. The iOS app is launched

## API Documentation

See `documentation.md` in the go-pos directory for complete API documentation.

## Dependencies

- gorilla/websocket (1.5) - WebSocket support
- jmoiron/sqlx (1.3) - SQLite database driver
- google/gopacket (1.20) - Network packet processing
- go-ole/go-ole (1.2) - Windows OLE automation
- sirupsen/logrus (1.9) - Logging

## Development

### Running Tests
```bash
go test
```

### Running with Logging
```bash
go run main.go -log-level=debug
```

### Building for Release
```bash
go build -o pos-server main.go
```

## Security Features

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
