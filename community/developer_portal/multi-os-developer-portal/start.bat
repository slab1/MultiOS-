@echo off
echo 🚀 Starting MultiOS Developer Portal...

REM Check if we're in the right directory
if not exist "package.json" (
    echo Error: package.json not found. Please run this script from the project root directory.
    pause
    exit /b 1
)

REM Install frontend dependencies if needed
if not exist "node_modules" (
    echo Installing frontend dependencies...
    call pnpm install
)

REM Install backend dependencies if needed
if not exist "server\node_modules" (
    echo Installing backend dependencies...
    cd server
    call pnpm install
    cd ..
)

REM Start backend server in background
echo Starting backend server on port 3001...
cd server
start "MultiOS Backend" cmd /k pnpm dev
cd ..

REM Wait a moment for backend to start
timeout /t 3 /nobreak >nul

REM Start frontend development server
echo Starting frontend development server on port 5173...
echo 🌐 Frontend URL: http://localhost:5173
echo 🔧 Backend API: http://localhost:3001
echo Press Ctrl+C to stop the frontend server
echo.

REM Run frontend
call pnpm dev

pause