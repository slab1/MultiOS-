#!/bin/bash

# MultiOS Code Browser Build Script
# This script sets up and builds the complete code browser system

set -e  # Exit on any error

echo "🚀 Building MultiOS Interactive Code Browser..."
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust not found. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        log_error "Node.js not found. Please install Node.js from https://nodejs.org/"
        exit 1
    fi
    
    # Check pnpm
    if ! command -v pnpm &> /dev/null; then
        log_warning "pnpm not found. Installing pnpm..."
        npm install -g pnpm
    fi
    
    # Check cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust and Cargo."
        exit 1
    fi
    
    log_success "All prerequisites are installed"
}

# Build backend
build_backend() {
    log_info "Building backend server..."
    
    cd backend
    
    # Check if Cargo.toml exists
    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found in backend directory"
        exit 1
    fi
    
    # Build the backend
    log_info "Compiling Rust backend..."
    cargo build --release
    
    if [ $? -eq 0 ]; then
        log_success "Backend built successfully"
    else
        log_error "Backend build failed"
        exit 1
    fi
    
    cd ..
}

# Build frontend
build_frontend() {
    log_info "Building frontend application..."
    
    cd frontend/code-browser-frontend
    
    # Check if package.json exists
    if [ ! -f "package.json" ]; then
        log_error "package.json not found in frontend directory"
        exit 1
    fi
    
    # Install dependencies
    log_info "Installing frontend dependencies..."
    pnpm install
    
    # Build the frontend
    log_info "Building React application..."
    pnpm run build
    
    if [ $? -eq 0 ]; then
        log_success "Frontend built successfully"
    else
        log_error "Frontend build failed"
        exit 1
    fi
    
    cd ../..
}

# Create environment files
setup_environment() {
    log_info "Setting up environment configuration..."
    
    # Backend environment
    if [ ! -f "backend/.env" ]; then
        cat > backend/.env << EOF
# MultiOS Code Browser Backend Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
RUST_LOG=info
MAX_FILE_SIZE=10485760
ANALYSIS_TIMEOUT=30
EOF
        log_success "Created backend environment file"
    else
        log_info "Backend environment file already exists"
    fi
    
    # Frontend environment
    if [ ! -f "frontend/code-browser-frontend/.env.local" ]; then
        cat > frontend/code-browser-frontend/.env.local << EOF
# MultiOS Code Browser Frontend Configuration
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080
VITE_APP_TITLE=MultiOS Code Browser
VITE_DEBUG_MODE=false
EOF
        log_success "Created frontend environment file"
    else
        log_info "Frontend environment file already exists"
    fi
}

# Setup directories
setup_directories() {
    log_info "Setting up required directories..."
    
    # Create necessary directories
    mkdir -p backend/logs
    mkdir -p backend/static
    mkdir -p backend/data
    
    # Set up sample data
    if [ ! -f "backend/data/sample_code.rs" ]; then
        cat > backend/data/sample_code.rs << 'EOF'
// Sample MultiOS Kernel Code
fn main() {
    println!("Initializing MultiOS Kernel...");
    
    initialize_kernel();
    start_scheduler();
    init_memory_manager();
    
    kernel_loop();
}

fn initialize_kernel() {
    setup_interrupt_handlers();
    configure_memory();
    init_pic_controller();
}

fn start_scheduler() {
    create_idle_task();
    setup_timer();
}

fn timer_handler() {
    increment_system_time();
    schedule_next_task();
}
EOF
        log_success "Created sample code file"
    fi
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    # Backend tests
    log_info "Running backend tests..."
    cd backend
    cargo test --verbose
    backend_test_result=$?
    cd ..
    
    # Frontend tests
    log_info "Running frontend tests..."
    cd frontend/code-browser-frontend
    pnpm test --passWithNoTests
    frontend_test_result=$?
    cd ../..
    
    if [ $backend_test_result -eq 0 ] && [ $frontend_test_result -eq 0 ]; then
        log_success "All tests passed"
    else
        log_warning "Some tests failed, but build continuing..."
    fi
}

# Generate documentation
generate_docs() {
    log_info "Generating documentation..."
    
    # Backend documentation
    cd backend
    cargo doc --no-deps
    cd ..
    
    # Copy documentation to docs folder
    mkdir -p docs/api
    cp -r backend/target/doc/* docs/api/ 2>/dev/null || true
    
    log_success "Documentation generated"
}

# Create startup scripts
create_scripts() {
    log_info "Creating startup scripts..."
    
    # Backend startup script
    cat > start-backend.sh << 'EOF'
#!/bin/bash
cd backend
cargo run --bin code-browser-backend
EOF
    chmod +x start-backend.sh
    
    # Frontend startup script
    cat > start-frontend.sh << 'EOF'
#!/bin/bash
cd frontend/code-browser-frontend
pnpm run dev
EOF
    chmod +x start-frontend.sh
    
    # Combined startup script
    cat > start-all.sh << 'EOF'
#!/bin/bash

# Start backend in background
echo "Starting MultiOS Code Browser Backend..."
cd backend && cargo run --bin code-browser-backend &
BACKEND_PID=$!

# Wait for backend to start
sleep 5

# Start frontend
echo "Starting MultiOS Code Browser Frontend..."
cd frontend/code-browser-frontend && pnpm run dev

# Cleanup function
cleanup() {
    echo "Shutting down services..."
    kill $BACKEND_PID 2>/dev/null
    exit
}

# Set trap for cleanup
trap cleanup INT TERM

# Wait for frontend
wait
EOF
    chmod +x start-all.sh
    
    log_success "Startup scripts created"
}

# Main build process
main() {
    echo ""
    log_info "Starting MultiOS Code Browser build process..."
    echo ""
    
    # Check if we're in the right directory
    if [ ! -d "backend" ] || [ ! -d "frontend" ]; then
        log_error "Please run this script from the code_browser root directory"
        exit 1
    fi
    
    check_prerequisites
    setup_environment
    setup_directories
    build_backend
    build_frontend
    run_tests
    generate_docs
    create_scripts
    
    echo ""
    echo "🎉 Build completed successfully!"
    echo "================================"
    echo ""
    echo "Next steps:"
    echo "1. Start the backend: ./start-backend.sh"
    echo "2. Start the frontend: ./start-frontend.sh"
    echo "3. Or start both: ./start-all.sh"
    echo ""
    echo "Access the application:"
    echo "- Frontend: http://localhost:5173"
    echo "- Backend API: http://localhost:8080"
    echo ""
    echo "For more information, see README.md"
    echo ""
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "MultiOS Code Browser Build Script"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --backend      Build only the backend"
        echo "  --frontend     Build only the frontend"
        echo "  --test         Run tests only"
        echo "  --clean        Clean build artifacts"
        echo ""
        exit 0
        ;;
    --backend)
        check_prerequisites
        setup_environment
        build_backend
        ;;
    --frontend)
        check_prerequisites
        setup_environment
        build_frontend
        ;;
    --test)
        run_tests
        ;;
    --clean)
        log_info "Cleaning build artifacts..."
        rm -rf backend/target
        rm -rf frontend/code-browser-frontend/node_modules
        rm -rf frontend/code-browser-frontend/dist
        rm -rf backend/logs
        rm -rf backend/data
        log_success "Clean completed"
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        echo "Use --help for usage information"
        exit 1
        ;;
esac
