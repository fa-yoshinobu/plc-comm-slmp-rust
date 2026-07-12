@echo off
setlocal

echo ===================================================
echo [CI] SLMP Rust local gate
echo ===================================================

echo [1/4] Checking formatting...
cargo fmt --all --check
if %errorlevel% neq 0 exit /b %errorlevel%

echo [2/4] Running clippy...
cargo clippy --all-targets --features cli -- -D warnings
if %errorlevel% neq 0 exit /b %errorlevel%

echo [3/4] Running tests...
cargo test
if %errorlevel% neq 0 exit /b %errorlevel%

echo [4/4] Checking Node crate...
cargo check -p slmp-node
if %errorlevel% neq 0 exit /b %errorlevel%

echo ===================================================
echo [SUCCESS] CI passed.
echo ===================================================
endlocal
