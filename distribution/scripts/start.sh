#!/bin/bash

# MultiOS Developer Portal Startup Script
# This script starts both the frontend and backend servers

echo "🚀 Starting MultiOS Developer Portal..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo -e "${RED}Error: package.json not found. Please run this script from the project root directory.${NC}"
    exit 1
fi

# Function to check if a port is in use
check_port() {
    if lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Check if ports are already in use
if check_port 5173; then
    echo -e "${YELLOW}Warning: Port 5173 is already in use. Frontend might not start properly.${NC}"
fi

if check_port 3001; then
    echo -e "${YELLOW}Warning: Port 3001 is already in use. Backend might not start properly.${NC}"
fi

# Install frontend dependencies if needed
if [ ! -d "node_modules" ]; then
    echo -e "${BLUE}Installing frontend dependencies...${NC}"
    pnpm install
fi

# Install backend dependencies if needed
if [ ! -d "server/node_modules" ]; then
    echo -e "${BLUE}Installing backend dependencies...${NC}"
    cd server && pnpm install && cd ..
fi

# Function to kill background processes on exit
cleanup() {
    echo -e "\n${YELLOW}Shutting down servers...${NC}"
    kill $(jobs -p) 2>/dev/null
    exit 0
}

# Set up trap to cleanup on script exit
trap cleanup SIGINT SIGTERM

# Start backend server in background
echo -e "${GREEN}Starting backend server on port 3001...${NC}"
cd server
pnpm dev &
BACKEND_PID=$!
cd ..

# Wait a moment for backend to start
sleep 2

# Start frontend development server
echo -e "${GREEN}Starting frontend development server on port 5173...${NC}"
echo -e "${BLUE}🌐 Frontend URL: http://localhost:5173${NC}"
echo -e "${BLUE}🔧 Backend API: http://localhost:3001${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop both servers${NC}"
echo ""

# Run frontend (this will block until stopped)
pnpm dev

# Wait for frontend to be stopped
wait