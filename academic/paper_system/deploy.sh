#!/bin/bash

# Academic Paper System Deployment Script
# This script sets up and starts both the backend and frontend services

set -e

echo "🚀 Starting Academic Paper System Deployment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================${NC}"
}

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    print_error "Node.js is not installed. Please install Node.js 18+ and try again."
    exit 1
fi

# Check if MongoDB is running
if ! pgrep -x "mongod" > /dev/null; then
    print_warning "MongoDB is not running. Please start MongoDB before proceeding."
    read -p "Do you want to start MongoDB? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_status "Starting MongoDB..."
        if command -v brew &> /dev/null; then
            brew services start mongodb/brew/mongodb-community
        elif command -v systemctl &> /dev/null; then
            sudo systemctl start mongod
        else
            print_error "Could not start MongoDB automatically. Please start it manually."
        fi
    fi
fi

# Function to install dependencies and start service
start_service() {
    local service_name=$1
    local service_path=$2
    local start_command=$3
    
    print_header "Setting up $service_name"
    
    cd "$service_path"
    
    # Check if package.json exists
    if [[ ! -f "package.json" ]]; then
        print_error "package.json not found in $service_path"
        return 1
    fi
    
    # Install dependencies
    print_status "Installing $service_name dependencies..."
    if command -v pnpm &> /dev/null; then
        pnpm install --frozen-lockfile
    elif command -v yarn &> /dev/null; then
        yarn install --frozen-lockfile
    else
        npm install
    fi
    
    # Copy environment file if it doesn't exist
    if [[ -f ".env.example" && ! -f ".env" ]]; then
        print_status "Creating environment configuration..."
        cp .env.example .env
        print_warning "Please review and update the .env file with your configuration"
    fi
    
    # Start the service in background
    print_status "Starting $service_name..."
    if [[ "$start_command" == "background" ]]; then
        nohup npm run dev > ../logs/${service_name}.log 2>&1 &
        echo $! > ../logs/${service_name}.pid
        print_status "$service_name started with PID: $(cat ../logs/${service_name}.pid)"
    else
        eval "$start_command"
    fi
}

# Create logs directory
mkdir -p logs

# Start Backend Service
print_header "Starting Backend Service"
cd backend
start_service "Backend" "$(pwd)" "npm run dev"

# Wait a moment for backend to start
sleep 5

# Start Frontend Service
print_header "Starting Frontend Service"
cd ../frontend/academic-platform
start_service "Frontend" "$(pwd)" "background"

# Wait for frontend to start
sleep 10

# Final status check
print_header "Deployment Status"

# Check if backend is running
if curl -s http://localhost:5000/api/health > /dev/null; then
    print_status "✅ Backend service is running on http://localhost:5000"
else
    print_error "❌ Backend service is not responding"
fi

# Check if frontend is running (basic check)
if pgrep -f "vite" > /dev/null; then
    print_status "✅ Frontend service is running"
else
    print_error "❌ Frontend service is not running"
fi

print_header "Deployment Complete!"
echo ""
print_status "Frontend: http://localhost:3000"
print_status "Backend API: http://localhost:5000"
print_status "API Health Check: http://localhost:5000/api/health"
echo ""

# Display demo credentials
print_header "Demo Credentials"
cat << EOF
${YELLOW}Researcher:${NC} researcher@os.academic.edu / password123
${YELLOW}Reviewer:${NC}  reviewer@os.academic.edu / password123
${YELLOW}Editor:${NC}     editor@os.academic.edu / password123
${YELLOW}Admin:${NC}      admin@os.academic.edu / password123
EOF

echo ""
print_status "To stop the services, run: ./stop-services.sh"

# Create stop script
cat > stop-services.sh << 'EOF'
#!/bin/bash
echo "🛑 Stopping Academic Paper System services..."

if [[ -f "logs/backend.pid" ]]; then
    kill $(cat logs/backend.pid) 2>/dev/null && echo "Backend stopped" || echo "Backend was not running"
    rm -f logs/backend.pid
fi

if pgrep -f "vite" > /dev/null; then
    pkill -f "vite" && echo "Frontend stopped" || echo "Frontend was not running"
fi

echo "All services stopped."
EOF

chmod +x stop-services.sh

echo ""
print_status "Services started successfully! 🎉"
print_status "Check the logs/ directory for service logs if needed."
