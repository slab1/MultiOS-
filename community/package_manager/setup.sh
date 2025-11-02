#!/bin/bash

# Educational Package Manager Setup Script
# ========================================
# 
# Automated setup script for MultiOS Educational Package Manager
# Installs dependencies, configures environment, and runs initial tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print colored output
print_header() {
    echo -e "${PURPLE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Check system requirements
check_requirements() {
    print_header "Checking System Requirements"
    
    local missing_deps=()
    local version_errors=()
    
    # Check Python version
    if command -v python3 &> /dev/null; then
        python_version=$(python3 -c "import sys; print('.'.join(map(str, sys.version_info[:2])))")
        if python3 -c "import sys; exit(0 if sys.version_info >= (3, 8) else 1)"; then
            print_success "Python $python_version found (>=3.8 required)"
        else
            version_errors+=("Python 3.8+ required, found $python_version")
        fi
    else
        missing_deps+=("python3")
    fi
    
    # Check pip
    if command -v pip3 &> /dev/null; then
        print_success "pip3 found"
    else
        missing_deps+=("pip3")
    fi
    
    # Check git
    if command -v git &> /dev/null; then
        print_success "git found"
    else
        print_warning "git not found (optional but recommended)"
    fi
    
    # Report missing dependencies
    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_error "Missing required dependencies: ${missing_deps[*]}"
        echo "Please install missing dependencies and rerun this script."
        exit 1
    fi
    
    if [ ${#version_errors[@]} -gt 0 ]; then
        print_error "Version errors: ${version_errors[*]}"
        echo "Please install a compatible Python version and rerun this script."
        exit 1
    fi
    
    print_success "All requirements satisfied"
}

# Install Python dependencies
install_dependencies() {
    print_header "Installing Python Dependencies"
    
    # Create virtual environment if it doesn't exist
    if [ ! -d "venv" ]; then
        print_info "Creating virtual environment..."
        python3 -m venv venv
    fi
    
    # Activate virtual environment
    print_info "Activating virtual environment..."
    source venv/bin/activate
    
    # Upgrade pip
    print_info "Upgrading pip..."
    pip install --upgrade pip
    
    # Install requirements
    if [ -f "requirements.txt" ]; then
        print_info "Installing requirements from requirements.txt..."
        pip install -r requirements.txt
    else
        print_warning "requirements.txt not found, installing minimal dependencies..."
        pip install requests PyYAML jsonschema
    fi
    
    print_success "Dependencies installed successfully"
}

# Setup directory structure
setup_directories() {
    print_header "Setting up Directory Structure"
    
    local dirs=(
        "packages"
        "packages/metadata"
        "packages/community"
        "packages/reviews"
        "packages/assets"
        "packages/test_reports"
        "cache"
        "security"
        "security/keys"
        "security/quarantine"
        "logs"
        "temp"
        "config"
    )
    
    for dir in "${dirs[@]}"; do
        mkdir -p "$dir"
        print_info "Created directory: $dir"
    done
    
    # Set appropriate permissions
    chmod 755 packages security cache logs
    
    print_success "Directory structure created"
}

# Create configuration files
setup_configuration() {
    print_header "Setting up Configuration"
    
    # Ensure config file exists
    if [ ! -f "config/config.json" ]; then
        print_error "Configuration file not found: config/config.json"
        exit 1
    fi
    
    # Make scripts executable
    chmod +x scripts/multios-pkg 2>/dev/null || true
    chmod +x tests/test_runner.py 2>/dev/null || true
    
    print_success "Configuration completed"
}

# Create sample package
create_sample_package() {
    print_header "Creating Sample Package"
    
    local sample_dir="samples/sample-math-curriculum"
    mkdir -p "$sample_dir"
    
    # Create basic structure
    mkdir -p "$sample_dir"/{src,tests,docs,curriculum,resources}
    
    # Create metadata
    cat > "$sample_dir/metadata.json" << 'EOF'
{
  "name": "sample-math-curriculum",
  "version": "1.0.0",
  "description": "Sample mathematics curriculum for grade 7 students",
  "author": "MultiOS Team",
  "email": "education@multios.edu",
  "type": "curriculum",
  "compatibility": "beginner",
  "subjects": ["Mathematics"],
  "grade_levels": ["Grade 7"],
  "prerequisites": ["Basic arithmetic"],
  "dependencies": {},
  "size": 1024000,
  "checksum": "sample-checksum",
  "created_at": "2025-11-03T05:34:19Z",
  "updated_at": "2025-11-03T05:34:19Z",
  "license": "MIT",
  "tags": ["mathematics", "grade7", "sample"]
}
EOF
    
    # Create curriculum manifest
    cat > "$sample_dir/curriculum/curriculum.yaml" << 'EOF'
learning_objectives:
  - "Understand basic algebraic concepts"
  - "Solve simple linear equations"
  - "Apply mathematics to real-world problems"

standards:
  - code: "CCSS.MATH.7.EE.4"
    description: "Use variables to represent quantities in real-world problems"

assessments:
  - type: "quiz"
    title: "Algebra Fundamentals Quiz"
    description: "Test understanding of basic algebraic concepts"

resources:
  - type: "interactive"
    title: "Equation Balance Scale"
    description: "Visual tool for understanding equation balance"
EOF
    
    # Create README
    cat > "$sample_dir/README.md" << 'EOF'
# Sample Math Curriculum

This is a sample mathematics curriculum package for educational package manager testing.

## Features

- Basic algebra concepts
- Interactive learning activities
- Assessment tools

## Usage

This package serves as an example for the educational package manager system.
EOF
    
    # Create license
    cat > "$sample_dir/LICENSE" << 'EOF'
MIT License

Copyright (c) 2025 MultiOS Team
EOF
    
    # Create requirements
    echo "# No additional dependencies required" > "$sample_dir/requirements.txt"
    
    print_success "Sample package created: $sample_dir"
}

# Run initial tests
run_tests() {
    print_header "Running Initial Tests"
    
    # Test package manager import
    if python3 -c "import src.package_manager; print('Package manager import: OK')"; then
        print_success "Package manager module import successful"
    else
        print_error "Package manager module import failed"
        return 1
    fi
    
    # Test CLI script
    if python3 scripts/multios-pkg --version 2>/dev/null; then
        print_success "CLI script test successful"
    else
        print_warning "CLI script test failed (may need manual setup)"
    fi
    
    # Test sample package creation
    if python3 src/package_manager.py validate samples/sample-math-curriculum --metadata samples/sample-math-curriculum/metadata.json; then
        print_success "Sample package validation successful"
    else
        print_warning "Sample package validation failed (check warnings above)"
    fi
    
    print_success "Initial tests completed"
}

# Create installation summary
create_summary() {
    print_header "Installation Summary"
    
    cat << EOF
${GREEN}Educational Package Manager Setup Complete!${NC}

${BLUE}Installation Details:${NC}
- Python packages installed in virtual environment
- Directory structure created
- Configuration files prepared
- Sample package created

${BLUE}Next Steps:${NC}
1. Activate the virtual environment:
   ${CYAN}source venv/bin/activate${NC}

2. Test the package manager:
   ${CYAN}python3 scripts/multios-pkg --help${NC}

3. Explore the sample package:
   ${CYAN}python3 scripts/multios-pkg validate samples/sample-math-curriculum${NC}

4. Create your first package:
   ${CYAN}python3 scripts/multios-pkg create ./my-curriculum --type curriculum${NC}

${BLUE}Documentation:${NC}
- README.md - Complete user guide
- docs/ - Detailed documentation
- samples/ - Example packages

${BLUE}Quick Commands:${NC}
- List packages: ${CYAN}python3 scripts/multios-pkg list${NC}
- Search packages: ${CYAN}python3 scripts/multios-pkg search mathematics${NC}
- Run tests: ${CYAN}python3 tests/test_runner.py samples/sample-math-curriculum.tar.gz${NC}

${BLUE}Support:${NC}
- GitHub Issues: https://github.com/multios/educational-package-manager/issues
- Documentation: https://docs.multios.edu
- Community: https://community.multios.edu

${GREEN}Happy packaging!${NC}
EOF
}

# Main installation function
main() {
    print_header "Educational Package Manager Setup"
    echo "This script will set up the MultiOS Educational Package Manager"
    echo
    
    # Ask for confirmation
    read -p "Continue with installation? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    
    echo
    
    # Run setup steps
    check_requirements
    install_dependencies
    setup_directories
    setup_configuration
    create_sample_package
    run_tests
    
    # Show summary
    create_summary
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        cat << EOF
Educational Package Manager Setup Script
========================================

Usage: $0 [options]

Options:
    --help, -h           Show this help message
    --skip-deps          Skip dependency installation
    --skip-tests         Skip running tests
    --sample-only        Create only sample package
    
This script sets up the complete Educational Package Manager environment
including dependencies, configuration, and sample packages.
EOF
        exit 0
        ;;
    --skip-deps)
        SKIP_DEPS=true
        ;;
    --skip-tests)
        SKIP_TESTS=true
        ;;
    --sample-only)
        SKIP_DEPS=true
        SKIP_TESTS=true
        ;;
esac

# Run main installation
main