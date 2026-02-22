# Wayfinder Windows Installer Builder
# This script builds the Wayfinder application and creates Windows installers

param(
    [switch]$CleanBuild = $false,
    [switch]$SignInstallers = $false
)

$ErrorActionPreference = "Stop"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "Wayfinder Windows Installer Builder" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Get current directory
$projectRoot = Get-Location
Write-Host "Project root: $projectRoot" -ForegroundColor Gray
Write-Host ""

# Check if npm is available
if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: npm not found. Please install Node.js" -ForegroundColor Red
    exit 1
}

# Clean build if requested
if ($CleanBuild) {
    Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
    Remove-Item -Path "dist" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "src-tauri/target" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "✓ Clean complete" -ForegroundColor Green
    Write-Host ""
}

# Install/Update dependencies
Write-Host "Installing/updating Node dependencies..." -ForegroundColor Yellow
npm install
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to install dependencies" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Dependencies installed" -ForegroundColor Green
Write-Host ""

# Build the frontend
Write-Host "Building frontend..." -ForegroundColor Yellow
npm run build
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Frontend build failed" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Frontend build complete" -ForegroundColor Green
Write-Host ""

# Build Tauri application
Write-Host "Building Tauri application..." -ForegroundColor Yellow
Write-Host "This may take several minutes..." -ForegroundColor Gray
Write-Host ""

if ($SignInstallers) {
    Write-Host "Building with code signing..." -ForegroundColor Yellow
    npm run tauri build -- --ci
}
else {
    npm run tauri build
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Tauri build failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✓ Build complete!" -ForegroundColor Green
Write-Host ""

# Find and report generated installers
$msiPath = Get-ChildItem -Path "src-tauri/target/release/bundle/msi" -Filter "*.msi" -ErrorAction SilentlyContinue | Select-Object -First 1
$nsisPath = Get-ChildItem -Path "src-tauri/target/release/bundle/nsis" -Filter "*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1

Write-Host "============================================" -ForegroundColor Green
Write-Host "Build Results" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""

if ($msiPath) {
    Write-Host "MSI Installer:" -ForegroundColor Cyan
    Write-Host "  $($msiPath.FullName)" -ForegroundColor Yellow
    Write-Host "  Size: $([math]::Round($msiPath.Length / 1MB, 2)) MB" -ForegroundColor Gray
    Write-Host ""
}

if ($nsisPath) {
    Write-Host "NSIS Installer (Recommended):" -ForegroundColor Cyan
    Write-Host "  $($nsisPath.FullName)" -ForegroundColor Yellow
    Write-Host "  Size: $([math]::Round($nsisPath.Length / 1MB, 2)) MB" -ForegroundColor Gray
    Write-Host ""
}

if (-not $msiPath -and -not $nsisPath) {
    Write-Host "WARNING: No installers found" -ForegroundColor Yellow
    Write-Host "Check src-tauri/target/release/bundle/ for build artifacts" -ForegroundColor Gray
}

Write-Host ""
Write-Host "Executable location:" -ForegroundColor Cyan
$exePath = Get-ChildItem -Path "src-tauri/target/release" -Filter "*.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($exePath) {
    Write-Host "  $($exePath.FullName)" -ForegroundColor Yellow
}
else {
    Write-Host "  Check src-tauri/target/release/" -ForegroundColor Gray
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  • Test: Run the executable to verify it works"
Write-Host "  • Distribute: Share the installer with users"
Write-Host "  • Sign: For production, code-sign the installer" -ForegroundColor Yellow
Write-Host ""
