# PERF VALIDATION MEMO

Date: 2026-06-11
Target repo: `plc-comm-slmp-rust`
Target PLC: iQ-R at `192.168.250.100`

## Done

- Implemented `read_named` plain `BIT` batching in `src/helpers.rs`.
  - Plain bit devices are read through containing word random reads when safe.
  - Public API, `read_typed`, and `read_bits` are unchanged.
  - Fallback direct bit reads remain for unsupported/special devices.
- Added Phase 3 mock tests in `tests/read_named_bit_batching.rs`.
  - Same-word bit sharing.
  - Representative bit patterns.
  - iQ-F X/Y octal boundary.
  - Mixed bit device kinds.
  - All supported plain bit batching codes.
  - `0xFF` random-read chunk split.
  - Long counter state direct-bit fallback.
  - Mixed `BIT`, `BIT_IN_WORD`, word, and dword reads.
- Android app was rebuilt with the updated Rust native library and installed/launched on device `GCC22X04302400CN`.

## Local Verification

- `cargo fmt --all --check`: passed.
- `cargo test`: passed.
- `cargo clippy --lib --bins --tests --features cli -- -D warnings`: passed.
- `cargo clippy --all-targets --features cli -- -D warnings`: passed after the user allowed minimal `examples/` lint fixes.
- `cargo build --features cli --bin slmp_verify_client`: passed.
- Android `cargo test --manifest-path rust-core/Cargo.toml --all-targets`: passed.
- iOS `cargo check --manifest-path rust/melsec-io-core-ffi/Cargo.toml`: passed.
- Android native `.so` build through `rust-build.bat`: passed for `armeabi-v7a`, `arm64-v8a`, and `x86_64`.
- Android `assembleDebug`: passed.
- Android install/launch: passed.

## Live PLC Results

- TCP `192.168.250.100:1025`
  - `slmp_verify_client read D0 1`: passed.
  - `read_named` 64 plain bits `M1000..M1063`: `requests_delta=1`, `tx_delta=51`, `rx_delta=25`, `values=64`.
- UDP `192.168.250.100:1027`
  - `slmp_verify_client read D0 1`: passed.
  - `read_named` 64 plain bits `M1000..M1063`: `requests_delta=1`, `tx_delta=51`, `rx_delta=25`, `values=64`.
- UDP `192.168.250.100:1035`
  - failed with `os error 10054`; this was the wrong UDP port for this PLC.

## App Overlay

- Android screenshot after rebuild/install showed block page `M0 - M63` at `500ms`.
- Overlay reading: `3.3 r`, `118B/65B`, `135MB`, `CPU 7.9%`.
- Screenshot file: `D:\_github_plc\PlcIoChecker_Android\android-screen-now.png`.
- Phase 0 before-change reference: iQ-R, 64 bit page, 500ms, about `120 req/s`.

## Known Hardware Conditions

- Full `device_range_sample_compare` against current iQ-R hardware reports X0/X1 bit-block write/readback mismatches.
- User confirmed X0 area has an input unit, so readback not matching written values is expected hardware behavior.
- With X excluded:
  - TCP/1025 `device_range_sample_compare`: passed with `passed=342`, `bit_blocks_failed=0`.
  - UDP/1027 `device_range_sample_compare`: passed with `passed=342`, `bit_blocks_failed=0`.

## Separate Known Issue

- `route_validation_compare` currently fails one case on both TCP/1025 and UDP/1027:
  - `write_block_roundtrip`
  - SLMP error `end_code=0xC05B`, command `0x1406`, subcommand `0x0002`
  - This is the mixed word+bit block-write route.
- Direct checks showed:
  - word-only `block-write`: passed.
  - bit-only `block-write`: passed.
  - mixed word+bit `block-write`: failed with `0xC05B`.
- Existing repo tests/docs already treat `0xC05B` as not retryable for mixed block writes.
- User confirmed this should be handled as a separate known issue, not as a required improvement for this performance task.
- This failure is unrelated to the `read_named` plain-bit batching change.
- iOS app install/launch was not performed in this Windows environment. Rust FFI check passed.

## Completion Status

- Performance objective is verified on real PLC for the main path: 64 plain bit `read_named` collapsed to one wire request.
- Android bridge build/install was completed and overlay shows low request rate.
- The current `0xC05B` mixed block-write behavior is recorded as unrelated and not required for this performance improvement.
