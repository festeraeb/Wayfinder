# Wayfinder Windows Setup Script
# This script installs all required dependencies for development

param(
    [switch]$SkipNodeModules = $false,
    [switch]$SkipPythonSetup = $false
)

$ErrorActionPreference = "Stop"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "Wayfinder Setup Script for Windows" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Function to check if a command exists
function Test-Command {
    param([string]$Command)
    $null = Get-Command $Command -ErrorAction SilentlyContinue
    return $?
}

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow
Write-Host ""

# Check Node.js
if (Test-Command node) {
    $nodeVersion = node --version
    Write-Host "✓ Node.js found: $nodeVersion" -ForegroundColor Green
}
else {
    Write-Host "✗ Node.js not found" -ForegroundColor Red
    Write-Host "  Install from: https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}

# Check Python
if (Test-Command python) {
    $pythonVersion = python --version
    Write-Host "✓ Python found: $pythonVersion" -ForegroundColor Green
}
else {
    Write-Host "✗ Python not found" -ForegroundColor Red
    Write-Host "  Install from: https://python.org/" -ForegroundColor Yellow
    exit 1
}

# Check Rust
if (Test-Command cargo) {
    $rustVersion = cargo --version
    Write-Host "✓ Rust found: $rustVersion" -ForegroundColor Green
}
else {
    Write-Host "✗ Rust not found" -ForegroundColor Red
    Write-Host "  Install from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "All prerequisites found!" -ForegroundColor Green
Write-Host ""

# Install Node dependencies
if (-not $SkipNodeModules) {
    if (-not (Test-Path "node_modules")) {
        Write-Host "Installing Node dependencies..." -ForegroundColor Yellow
        npm install
        if ($LASTEXITCODE -ne 0) {
            Write-Host "Failed to install Node dependencies" -ForegroundColor Red
            exit 1
        }
        Write-Host "✓ Node dependencies installed" -ForegroundColor Green
    }
    else {
        Write-Host "✓ Node dependencies already installed" -ForegroundColor Green
    }
}
else {
    Write-Host "⊘ Skipping Node dependencies (--SkipNodeModules)" -ForegroundColor Yellow
}

Write-Host ""

# Setup Python virtual environment
if (-not $SkipPythonSetup) {
    if (-not (Test-Path ".venv")) {
        Write-Host "Creating Python virtual environment..." -ForegroundColor Yellow
        python -m venv .venv
        
        # Activate venv
        & ".\.venv\Scripts\Activate.ps1"
        
        # Install Python dependencies
        Write-Host "Installing Python dependencies..." -ForegroundColor Yellow
        pip install --upgrade pip setuptools wheel
        
        if (Test-Path "requirements.txt") {
            pip install -r requirements.txt
        }
        
        Write-Host "✓ Python environment setup complete" -ForegroundColor Green
    }
    else {
        Write-Host "✓ Python virtual environment already exists" -ForegroundColor Green
        Write-Host "  To activate: .\.venv\Scripts\Activate.ps1" -ForegroundColor Cyan
    }
}
else {
    Write-Host "⊘ Skipping Python setup (--SkipPythonSetup)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "Setup Complete! Ready to develop." -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. Activate Python environment (if needed):"
Write-Host "     .\.venv\Scripts\Activate.ps1" -ForegroundColor Yellow
Write-Host ""
Write-Host "  2. Start development:"
Write-Host "     npm run tauri dev" -ForegroundColor Yellow
Write-Host ""
Write-Host "  3. Or use the batch launcher:"
Write-Host "     launch-wayfinder.bat" -ForegroundColor Yellow
Write-Host ""
