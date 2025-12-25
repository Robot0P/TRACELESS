@echo off
REM Traceless Windows Build Script
REM Windows 构建脚本

echo ========================================
echo   Traceless Build Script for Windows
echo ========================================
echo.

REM 检查 Node.js
where node >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Node.js is not installed.
    echo Please install Node.js from https://nodejs.org/
    exit /b 1
)
echo [OK] Node.js found

REM 检查 Rust
where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo [ERROR] Rust is not installed.
    echo Please install Rust from https://rustup.rs/
    exit /b 1
)
echo [OK] Rust found

REM 安装依赖
echo.
echo [*] Installing dependencies...
call npm ci
if %errorlevel% neq 0 (
    call npm install
)

REM 构建应用
echo.
echo [*] Building Traceless for Windows...
call npm run tauri build

if %errorlevel% neq 0 (
    echo [ERROR] Build failed!
    exit /b 1
)

echo.
echo [OK] Build completed!
echo.
echo Build artifacts:
echo   MSI: src-tauri\target\release\bundle\msi\
echo   EXE: src-tauri\target\release\bundle\nsis\
echo.

dir /s /b src-tauri\target\release\bundle\msi\*.msi 2>nul
dir /s /b src-tauri\target\release\bundle\nsis\*.exe 2>nul

echo.
echo Done!
pause
