#!/bin/bash

echo "🚀 Educational App Store - Setup Script"
echo "========================================"

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL is not installed. Please install PostgreSQL first."
    exit 1
fi

# Create database
echo "📦 Creating database..."
createdb educational_app_store 2>/dev/null || echo "Database already exists"

# Install backend dependencies
echo "📦 Installing backend dependencies..."
cd backend && npm install

# Install frontend dependencies  
echo "📦 Installing frontend dependencies..."
cd ../ && pnpm install

echo ""
echo "✅ Setup completed!"
echo ""
echo "📝 Next steps:"
echo "1. Run database migration: cd backend && npm run db:migrate"
echo "2. Seed database: cd backend && npm run db:seed"
echo "3. Start backend: cd backend && npm run dev"
echo "4. Start frontend: npm run dev"
echo ""
echo "🔐 Default admin login:"
echo "Email: admin@edustore.com"
echo "Password: admin123"
echo ""
echo "🔐 Sample developer login:"
echo "Email: developer@edustore.com"
echo "Password: dev123"