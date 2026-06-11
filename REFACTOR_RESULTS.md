# Refactor Results

Date: 2026-06-11

## Summary

- Moved self-independent client helper logic from `src/client.rs` into the private `src/client_rules.rs` module.
- Added characterization coverage for bit unpacking and long timer word decoding.
- Added private module unit tests for moved classification, guard, conversion, and CPU state decode helpers.
- Added CI gates for `cargo fmt --all --check` and `cargo clippy --all-targets --features cli -- -D warnings`.

## Moved Helpers

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

Skipped helpers: none.

## Verification

All commands completed successfully after the refactor:

- `cargo test`
- `cargo build --features cli --bin slmp_verify_client`
- `cargo check -p slmp-node`
- `cargo doc --no-deps -p plc-comm-slmp-rust`
- `cargo fmt --all --check`
- `cargo clippy --all-targets --features cli -- -D warnings`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --manifest-path ../PlcIoChecker_Android/rust-core/Cargo.toml --all-targets`
- `cargo check --manifest-path ../PlcIoChecker_iOS/rust/melsec-io-core-ffi/Cargo.toml`

## API And Scope Checks

- Public `pub use` exports in `src/lib.rs` are unchanged.
- `src/lib.rs` only adds private `mod client_rules;`.
- Existing tests, shared vectors, docs, examples, changelog, and `crates/slmp-node` were not modified.
- No crate version, MSRV, edition, dependency, or wire protocol changes were made.

## Deferred Ideas

- Keep frame building/parsing extraction as a future design-only topic because it would require threading compatibility mode and frame type through private signatures.
- Keep `error_codes.rs` table consolidation or codegen as a future design-only topic because it is high-volume data movement with little benefit for this refactor.
