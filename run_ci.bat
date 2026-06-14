@echo off
setlocal

echo ===================================================
echo [CI] SLMP Rust local gate
echo ===================================================

echo [1/5] Checking formatting...
cargo fmt --all --check
if %errorlevel% neq 0 exit /b %errorlevel%

echo [2/5] Running clippy...
cargo clippy --all-targets --features cli -- -D warnings
if %errorlevel% neq 0 exit /b %errorlevel%

echo [3/5] Running tests...
cargo test
if %errorlevel% neq 0 exit /b %errorlevel%

echo [4/5] Checking Node crate...
cargo check -p slmp-node
if %errorlevel% neq 0 exit /b %errorlevel%

echo [5/5] Building verify client...
cargo build --features cli --bin slmp_verify_client
if %errorlevel% neq 0 exit /b %errorlevel%

echo ===================================================
echo [SUCCESS] CI passed.
echo ===================================================
endlocal
