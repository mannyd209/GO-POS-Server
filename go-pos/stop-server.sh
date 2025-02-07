#!/bin/bash

echo "Stopping all processes on port 8000..."

# Find and kill process using port 8000
PID=$(lsof -ti:8000)
if [ ! -z "$PID" ]; then
    echo "Killing process $PID..."
    kill -9 $PID
fi

# Kill any go run processes
pkill -f "go run main.go"

# Only flush mDNS if --flush flag is provided
if [ "$1" == "--flush" ]; then
    echo "Flushing mDNS cache (requires sudo)..."
    sudo killall -HUP mDNSResponder
fi

echo "Server stopped successfully!"
