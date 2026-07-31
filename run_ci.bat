@echo off
setlocal

echo ===================================================
echo [CI] SLMP Rust local gate
echo ===================================================

echo [1/6] Checking formatting...
cargo fmt --all --check
if %errorlevel% neq 0 exit /b %errorlevel%

echo [2/6] Running clippy...
cargo clippy --all-targets --features cli -- -D warnings
if %errorlevel% neq 0 exit /b %errorlevel%

echo [3/6] Checking rustdoc...
set "RUSTDOCFLAGS=-D warnings"
cargo doc --no-deps --all-features
if %errorlevel% neq 0 exit /b %errorlevel%
set "RUSTDOCFLAGS="

echo [4/6] Running tests...
cargo test
if %errorlevel% neq 0 exit /b %errorlevel%

echo [5/6] Checking Node crate...
cargo check -p slmp-node
if %errorlevel% neq 0 exit /b %errorlevel%

echo [6/6] Validating generated crate and isolated consumer...
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check_package_contents.ps1
if %errorlevel% neq 0 exit /b %errorlevel%

echo ===================================================
echo [SUCCESS] CI passed.
echo ===================================================
endlocal
