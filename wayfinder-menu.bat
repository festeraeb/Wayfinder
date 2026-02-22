@echo off
REM Wayfinder Quick Start Menu for Windows
REM Use this to set up and launch Wayfinder

setlocal enabledelayedexpansion

:menu
cls
echo.
echo ============================================
echo Wayfinder - Quick Start Menu
echo ============================================
echo.
echo 1. Setup (Install dependencies)
echo 2. Launch Development Mode
echo 3. Build Installers
echo 4. Open Project Folder
echo 5. View Documentation
echo 6. Exit
echo.
set /p choice="Enter your choice (1-6): "

if "%choice%"=="1" goto setup
if "%choice%"=="2" goto launch
if "%choice%"=="3" goto build
if "%choice%"=="4" goto openfolder
if "%choice%"=="5" goto docs
if "%choice%"=="6" goto exit

echo Invalid choice. Please try again.
timeout /t 2 /nobreak
goto menu

:setup
cls
echo.
echo Running setup...
echo.
powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\setup-windows.ps1'"
if %errorlevel% equ 0 (
    echo.
    echo Setup completed successfully!
) else (
    echo.
    echo Setup failed. Check the error messages above.
)
pause
goto menu

:launch
cls
echo.
echo Launching Wayfinder development server...
echo (Close the Tauri window to stop)
echo.
call launch-wayfinder.bat
goto menu

:build
cls
echo.
echo Building installers...
echo This will take several minutes...
echo.
powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\build-installer.ps1'"
pause
goto menu

:openfolder
start explorer.exe "%cd%"
goto menu

:docs
cls
echo.
echo Available Documentation:
echo.
if exist "TAURI_SETUP.md" (
    echo * TAURI_SETUP.md - Detailed setup guide
)
if exist "README.md" (
    echo * README.md - Project overview
)
if exist "IMPLEMENTATION_SUMMARY.md" (
    echo * IMPLEMENTATION_SUMMARY.md - Implementation details
)
echo.
pause
goto menu

:exit
endlocal
exit /b 0
