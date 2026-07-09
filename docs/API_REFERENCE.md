# SLMP Rust API Reference

This page is a user-facing index of the public Rust SLMP client surface.
Use the usage guide for examples, and this page when you need to find the
operation name for a specific SLMP command family.

The main async client type is `SlmpClient`.

## Direct And Random Device Operations

| Operation | Public API |
| --- | --- |
| Direct word read/write | `read_words_raw`, `write_words` |
| Direct bit read/write | `read_bits`, `write_bits` |
| 32-bit values | `read_dwords_raw`, `write_dwords` |
| Float32 values | `read_float32s`, `write_float32s` |
| Extended direct word read/write | `read_words_extended`, `write_words_extended` |
| Extended direct bit read/write | `read_bits_extended`, `write_bits_extended` |
| Random read | `read_random` |
| Extended random read | `read_random_ext` |
| Random word/dword write | `write_random_words` |
| Extended random word/dword write | `write_random_words_ext` |
| Random bit write | `write_random_bits` |
| Extended random bit write | `write_random_bits_ext` |
| Block read/write | `read_block`, `write_block` |
| Type name | `read_type_name` |

Extended random APIs use the 008x subcommands. Use `parse_qualified_device`
or `SlmpQualifiedDeviceAddress` with `SlmpExtensionSpec` for routed devices
such as `U1\G0`, `U3E0\HG0`, or `J2\SW10`.

## Specialized Operations

| Operation | Public API |
| --- | --- |
| Memory command words | `memory_read_words`, `memory_write_words` |
| Extend-unit command words | `extend_unit_read_words`, `extend_unit_write_words` |
| Label array access | `read_array_labels`, `write_array_labels` |
| Label random access | `read_random_labels`, `write_random_labels` |
| Remote CPU control | `remote_run`, `remote_stop`, `remote_pause`, `remote_latch_clear`, `remote_reset` |
| Remote password | `remote_password_unlock`, `remote_password_lock` |
| CPU operation state | `read_cpu_operation_state` |
| Self-diagnosis error code | `read_latest_self_diagnosis_error_code` |
| Device range catalog | `read_device_range_catalog`, `read_device_range_catalog_for_plc_profile` |
| Self-test loopback | `self_test_loopback` |

Monitor registration/cycle APIs are not part of the current Rust user-facing
client surface. CPU-buffer convenience helpers are also not separate methods;
use the extended-device `U3E0\HG...` route where the selected profile supports
CPU-buffer access.

## High-Level Helpers

| Operation | Public API |
| --- | --- |
| Connection options | `SlmpConnectionOptions`, `SlmpPlcProfile::available_connection_profiles`, `SlmpTransportMode`, `SlmpFrameType`, `SlmpCompatibilityMode` |
| Address parsing | `SlmpAddress::parse`, `SlmpAddress::parse_for_plc_profile`, `SlmpAddress::format`, `parse_device_for_plc_profile`, `parse_qualified_device` |
| Typed values | `read_typed`, `write_typed` |
| Named mixed snapshots | `read_named`, `write_named`, `poll_named` |
| Chunked word/dword reads | `read_words_single_request`, `read_words_chunked`, `read_dwords_single_request`, `read_dwords_chunked` |
| Bit-in-word write | `write_bit_in_word` |
| Trace and diagnostics | `last_request_frame`, `last_response_frame`, `traffic_stats` |

## Target Module I/O Constants

`SlmpModuleIo` provides named request-header module I/O numbers for multi-CPU
and routed CPU targets. Use these values in `SlmpTargetAddress.module_io`;
`SlmpTargetAddress::default()` remains the own-station route `0x03FF`.

| Constant | Value |
| --- | --- |
| `SlmpModuleIo::CONTROL_SYSTEM_CPU` | `0x03D0` |
| `SlmpModuleIo::STANDBY_SYSTEM_CPU` | `0x03D1` |
| `SlmpModuleIo::SYSTEM_A_CPU` | `0x03D2` |
| `SlmpModuleIo::SYSTEM_B_CPU` | `0x03D3` |
| `SlmpModuleIo::MULTIPLE_CPU_1` .. `SlmpModuleIo::MULTIPLE_CPU_4` | `0x03E0` .. `0x03E3` |
| `SlmpModuleIo::REMOTE_HEAD_1` / `SlmpModuleIo::REMOTE_HEAD_2` | `0x03E0` / `0x03E1` |
| `SlmpModuleIo::CONTROL_SYSTEM_REMOTE_HEAD` / `SlmpModuleIo::STANDBY_SYSTEM_REMOTE_HEAD` | `0x03D0` / `0x03D1` |
| `SlmpModuleIo::OWN_STATION` | `0x03FF` |
