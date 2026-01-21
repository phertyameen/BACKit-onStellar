@echo off
REM Quick verification script for BACKit-onStellar monorepo setup

echo ========================================
echo BACKit-onStellar Setup Verification
echo ========================================
echo.

echo [Step 1/5] Checking pnpm installation...
where pnpm >nul 2>nul
if %errorlevel% neq 0 (
    echo [FAIL] pnpm is not installed!
    echo Please install: npm install -g pnpm
    pause
    exit /b 1
)
echo [PASS] pnpm is installed
echo.

echo [Step 2/5] Verifying workspace structure...
if not exist "packages\frontend" (
    echo [FAIL] Frontend package not found!
    exit /b 1
)
if not exist "packages\backend" (
    echo [FAIL] Backend package not found!
    exit /b 1
)
if not exist "packages\contracts" (
    echo [FAIL] Contracts package not found!
    exit /b 1
)
echo [PASS] All 3 packages exist
echo   - packages\frontend
echo   - packages\backend
echo   - packages\contracts
echo.

echo [Step 3/5] Checking configuration files...
if not exist "pnpm-workspace.yaml" (
    echo [FAIL] pnpm-workspace.yaml not found!
    exit /b 1
)
if not exist "turbo.json" (
    echo [FAIL] turbo.json not found!
    exit /b 1
)
if not exist ".gitignore" (
    echo [FAIL] .gitignore not found!
    exit /b 1
)
if not exist ".nvmrc" (
    echo [FAIL] .nvmrc not found!
    exit /b 1
)
echo [PASS] All configuration files present
echo.

echo [Step 4/5] Testing pnpm install...
call pnpm install
if %errorlevel% neq 0 (
    echo [FAIL] pnpm install failed!
    pause
    exit /b 1
)
echo [PASS] pnpm install successful
echo.

echo [Step 5/5] Listing workspace packages...
call pnpm ls -r --depth=-1
echo.

echo ========================================
echo Verification Summary
echo ========================================
echo [PASS] pnpm workspace configured correctly
echo [PASS] All 3 packages detected
echo [PASS] Dependencies installed
echo [PASS] Turborepo ready
echo.
echo Next: Run 'pnpm dev' to test (will show expected errors)
echo ========================================
echo.
pause
