@echo off
chcp 65001 >nul
title Productivity App - 启动中...

echo [1/2] 检查 Rust 环境...
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo [错误] 未找到 Cargo，请先安装 Rust
    echo 访问: https://rustup.rs/
    pause
    exit /b 1
)

echo [2/2] 启动应用...
echo.

cargo tauri dev

if %errorlevel% neq 0 (
    echo.
    echo [错误] 应用启动失败
    pause
    exit /b 1
)

pause
