@echo off
REM BACKit-onStellar Monorepo Setup Script

echo ================================
echo BACKit-onStellar Setup
echo ================================
echo.

REM Check if pnpm is installed
where pnpm >nul 2>nul
if %errorlevel% neq 0 (
    echo [ERROR] pnpm is not installed!
    echo Please install pnpm first: npm install -g pnpm
    pause
    exit /b 1
)

echo [1/3] Installing dependencies...
call pnpm install

if %errorlevel% neq 0 (
    echo [ERROR] Failed to install dependencies
    pause
    exit /b 1
)

echo.
echo [2/3] Verifying workspace structure...
if not exist "packages\frontend" (
    echo [ERROR] Frontend package not found!
    exit /b 1
)
if not exist "packages\backend" (
    echo [ERROR] Backend package not found!
    exit /b 1
)
if not exist "packages\contracts" (
    echo [ERROR] Contracts package not found!
    exit /b 1
)

echo.
echo [3/3] Testing dev command...
echo Note: This will show placeholder messages as packages are not yet configured
timeout /t 2 >nul
call pnpm run dev

echo.
echo ================================
echo Setup Complete!
echo ================================
echo.
echo Next steps:
echo 1. Configure each package with the required dependencies
echo 2. Run 'pnpm dev' to start all packages in development mode
echo 3. See individual package READMEs for specific setup instructions
echo.
pause
