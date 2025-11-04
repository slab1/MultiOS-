@echo off
setlocal enabledelayedexpansion

REM MultiOS Build Script for Windows
REM Handles building MultiOS for different target architectures

set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%..
set BUILD_DIR=%PROJECT_ROOT%\target
set DIST_DIR=%PROJECT_ROOT%\dist
set LOG_FILE=%PROJECT_ROOT%\build.log

REM Target configurations
set TARGETS[x86_64]=x86_64-unknown-none
set TARGETS[arm64]=aarch64-unknown-none
set TARGETS[riscv64]=riscv64gc-unknown-none

REM Default values
set TARGET=
set RELEASE=false
set CLEAN=false
set VERBOSE=false
set HELP=false

REM Parse command line arguments
:parse_args
if "%~1"=="" goto validate_args
if "%~1"=="-t" goto set_target
if "%~1"=="--target" goto set_target
if "%~1"=="-r" set RELEASE=true&shift&goto parse_args
if "%~1"=="--release" set RELEASE=true&shift&goto parse_args
if "%~1"=="-c" set CLEAN=true&shift&goto parse_args
if "%~1"=="--clean" set CLEAN=true&shift&goto parse_args
if "%~1"=="-v" set VERBOSE=true&shift&goto parse_args
if "%~1"=="--verbose" set VERBOSE=true&shift&goto parse_args
if "%~1"=="-h" set HELP=true&shift&goto parse_args
if "%~1"=="--help" set HELP=true&shift&goto parse_args
echo Unknown option: %~1
exit /b 1

:set_target
set TARGET=%~2
shift
shift
goto parse_args

:validate_args
if "%HELP%"=="true" goto show_help
if "%TARGET%"=="" echo Target architecture must be specified. Use --help for usage information.&exit /b 1

REM Check if target is valid
set TARGET_TRIPLES[x86_64]=x86_64-unknown-none
set TARGET_TRIPLES[arm64]=aarch64-unknown-none
set TARGET_TRIPLES[riscv64]=riscv64gc-unknown-none

if "!TARGETS[%TARGET%]!"=="" echo Invalid target: %TARGET%. Valid targets: x86_64, arm64, riscv64&exit /b 1

goto main

:show_help
echo MultiOS Build Script
echo.
echo Usage: %0 [OPTIONS]
echo.
echo Options:
echo     -t, --target TARGET     Target architecture (x86_64, arm64, riscv64)
echo     -r, --release          Build in release mode
echo     -c, --clean            Clean build artifacts before building
echo     -v, --verbose          Enable verbose output
echo     -h, --help             Show this help message
echo.
echo Examples:
echo     %0 --target x86_64 --release
echo     %0 --target arm64 --clean
echo     %0 --target riscv64 --verbose
echo.
echo Targets:
echo     x86_64   - Intel/AMD 64-bit processors
echo     arm64    - ARM 64-bit processors (AArch64)
echo     riscv64  - RISC-V 64-bit processors
exit /b 0

:main
echo MultiOS Build Script Starting...
echo Target: %TARGET%
echo Release: %RELEASE%

REM Setup environment
echo Setting up build environment...
if not exist "%BUILD_DIR%" mkdir "%BUILD_DIR%"
if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"
echo MultiOS Build Log - %date% %time% > "%LOG_FILE%"

REM Setup cross-compilation tools if needed
if "%TARGET%" neq "x86_64" (
    echo Setting up cross-compilation for %TARGET%...
    call :setup_cross_compilation %TARGET%
)

REM Clean build artifacts
if "%CLEAN%"=="true" (
    echo Cleaning build artifacts...
    cargo clean
    if exist "%BUILD_DIR%" rmdir /s /q "%BUILD_DIR%"
    if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
    mkdir "%BUILD_DIR%"
    mkdir "%DIST_DIR%"
)

REM Build the project
echo Building MultiOS for %TARGET%...
set CARGO_ARGS=
if "%RELEASE%"=="true" set CARGO_ARGS=%CARGO_ARGS% --release
if "%VERBOSE%"=="true" set CARGO_ARGS=%CARGO_ARGS% --verbose

REM Build workspace
echo Building workspace crates...
cargo build %CARGO_ARGS% || echo Build failed&exit /b 1

REM Build kernel specifically
echo Building kernel...
cd "%PROJECT_ROOT%\kernel"
cargo build %CARGO_ARGS% || echo Kernel build failed&exit /b 1

REM Build bootloader
echo Building bootloader...
cd "%PROJECT_ROOT%\bootloader"
cargo build %CARGO_ARGS% || echo Bootloader build failed&exit /b 1

REM Build userland
echo Building userland...
cd "%PROJECT_ROOT%\userland"
cargo build %CARGO_ARGS% || echo Userland build failed&exit /b 1

REM Package build artifacts
echo Packaging build artifacts for %TARGET%...
set TARGET_DIR=%DIST_DIR%\%TARGET%
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"

REM Create metadata file
echo MultiOS Build Information > "%TARGET_DIR%\build_info.txt"
echo ======================== >> "%TARGET_DIR%\build_info.txt"
echo Target: %TARGET% >> "%TARGET_DIR%\build_info.txt"
echo Build Type: %RELEASE% >> "%TARGET_DIR%\build_info.txt"
echo Build Date: %date% %time% >> "%TARGET_DIR%\build_info.txt"
echo Rust Version: >> "%TARGET_DIR%\build_info.txt"
rustc --version >> "%TARGET_DIR%\build_info.txt"
echo Cargo Version: >> "%TARGET_DIR%\build_info.txt"
cargo --version >> "%TARGET_DIR%\build_info.txt"

echo Build completed successfully!
echo Artifacts available in: %TARGET_DIR%
echo Build log: %LOG_FILE%
exit /b 0

:setup_cross_compilation
set TARGET_PLATFORM=%~1
echo Note: Cross-compilation tools may need to be installed manually on Windows
echo For %TARGET_PLATFORM%, you may need to install appropriate GCC toolchain
exit /b 0