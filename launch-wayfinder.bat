@echo off
REM Wayfinder Application Launcher for Windows
REM This script launches the Wayfinder desktop application

setlocal enabledelayedexpansion

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0
cd /d "%SCRIPT_DIR%"

REM Check if Node is installed
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Node.js is not installed or not in PATH
    echo Please install Node.js from https://nodejs.org/
    echo.
    pause
    exit /b 1
)

REM Check if Python is installed
where python >nul 2>nul
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Python is not installed or not in PATH
    echo Please install Python from https://python.org/
    echo.
    pause
    exit /b 1
)

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo.
    echo ERROR: Rust is not installed
    echo Please install Rust from https://rustup.rs/
    echo.
    pause
    exit /b 1
)

echo.
echo ============================================
echo Wayfinder Application Launcher
echo ============================================
echo.
echo Starting development server...
echo Node version: 
node --version
echo Python version: 
python --version
echo Rust version:
cargo --version
echo.

REM Install dependencies if node_modules doesn't exist
if not exist "node_modules" (
    echo Installing Node dependencies...
    call npm install
    if %errorlevel% neq 0 (
        echo ERROR: Failed to install dependencies
        pause
        exit /b 1
    )
)

REM Launch Tauri dev server
echo.
echo Launching Wayfinder...
echo Press Ctrl+C to stop
echo.
call npm run tauri dev

endlocal
