# Development History

Last consolidated: 2026-06-11

This document preserves the useful content that used to live in temporary
refactor and performance memo files. Keep this file as the durable engineering
record for the SLMP communication crate.

## Library Contracts

- Preserve the public API unless an API migration is explicitly approved.
- Preserve existing frame strings and protocol behavior.
- Keep Android and iOS FFI consumers compatible.
- Do not add silent compatibility fallbacks for invalid data.
- Prefer characterization tests before moving protocol logic.

## Refactoring History

### Client Rule Extraction

Completed work:

- Moved self-independent client helper logic from `src/client.rs` into private
  `src/client_rules.rs`.
- Added `mod client_rules;` privately from `src/lib.rs`.
- Kept the public API unchanged.

Moved helpers:

- `map_plc_family_to_range_family`
- `validate_non_empty_u16_count`
- `validate_u16_count`
- `validate_direct_bit_read`
- `validate_direct_bit_write`
- `validate_direct_word_read`
- `validate_direct_word_write`
- `validate_direct_dword_read`
- `validate_direct_dword_write`
- `validate_random_read_devices`
- `validate_random_write_word_devices`
- `is_long_timer_state_device`
- `requires_random_bit_write`
- `is_long_current_value_device`
- `is_dword_only_scalar_device`
- `is_random_dword_only_read_device`
- `validate_no_lcs_lcc_block_read`
- `validate_no_lcs_lcc_block_write`
- `unpack_bit_values`
- `parse_long_timer_words`
- `decode_cpu_operation_state`

Skipped helpers:

- None in that extraction pass.

Effect:

- `src/client.rs` became more focused on client orchestration.
- Rule-level behavior became easier to test directly.
- The crate avoided an API churn while improving internal shape.

### Added Characterization Tests

Covered behavior includes:

- Bit unpacking.
- Long timer word decoding.
- PLC family and range-family classification.
- Validation guards.
- Conversion helpers.
- CPU operation-state decoding.

Observed verification:

- `cargo test` passed.
- CLI build passed.
- Node check passed.
- Rust doc build passed.
- `cargo fmt` passed.
- `cargo clippy` with CLI features passed.
- `cargo clippy --all-targets` passed.
- Android rust-core tests passed.
- iOS FFI cargo check passed.

## Performance History

### `read_named` Plain BIT Batching

Completed work:

- Optimized `read_named` for plain BIT devices in `src/helpers.rs`.
- Plain bit devices now read through containing word random reads when it is
  safe to do so.
- Direct bit-read fallback remains available for cases that should not be
  batched.
- Public API stayed unchanged.

Added tests:

- Same-word sharing.
- Bit patterns.
- iQ-F X/Y octal boundary behavior.
- Mixed device kinds.
- Supported device codes.
- `0xFF` split behavior.
- Long counter state fallback.
- Mixed `BIT`, `BIT_IN_WORD`, word, and dword reads.

Effect:

- 64 plain bits can collapse to one wire request when they are batchable.
- This is especially important for app block pages with many BIT devices.

### Live PLC Validation

Hardware:

- MELSEC iQ-R at `192.168.250.100`.

TCP validation:

- TCP port 1025.
- `read D0 1` succeeded.
- `read_named` for 64 plain bits `M1000..M1063` produced:
  - `requests_delta=1`
  - `tx_delta=51`
  - `rx_delta=25`
  - `values=64`

UDP validation:

- UDP port 1027 succeeded with the same 64-bit read behavior.
- UDP port 1035 failed with `os error 10054`; this was treated as a wrong-port
  configuration, not a crate regression.

App overlay after rebuild:

- M0-M63, 500 ms polling interval.
- Overlay showed about `3.3 r`.
- Traffic showed about `118B/65B`.
- Memory showed about `135MB`.
- CPU showed about `7.9%`.

Phase 0 reference before the batching change:

- iQ-R, 64-bit page, 500 ms polling interval.
- About 120 requests per second.

Known hardware note:

- X0 area input unit caused a write/read mismatch in a device-range sample
  comparison.
- Excluding X, `device_range_sample_compare` passed 342 cases for TCP/UDP.

Separate known issue:

- Mixed word+bit block-write issue `0xC05B`, command `0x1406`, subcommand
  `0x0002`, is tracked separately and is unrelated to the plain BIT batching
  optimization.

## Work Intentionally Not Done

- No public API migration.
- No frame builder/parser extraction in the completed refactor pass.
- No `error_codes.rs` table consolidation or code generation.
- No binary mobile FFI envelope work was done in this crate during the mobile
  FFI envelope measurement pass because the measured mobile cost was below the
  optimization gate.
- iOS app install/launch was not performed in the Windows validation
  environment, although Rust FFI checks passed.

## Future Notes

- Frame building/parsing extraction remains a reasonable future refactor, but
  it should be preceded by golden frame tests.
- `error_codes.rs` table consolidation can be considered if the table grows or
  becomes hard to audit manually.
- Keep Android and iOS bridge checks in the validation list when changing shared
  FFI files.
