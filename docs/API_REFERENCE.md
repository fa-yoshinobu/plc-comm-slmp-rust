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
| Word-only / dword-only random read | `read_random_words`, `read_random_dwords` |
| Extended random read | `read_random_ext` |
| Extended word-only / dword-only random read | `read_random_words_extended`, `read_random_dwords_extended` |
| Random word/dword write | `write_random_words` |
| Word-only / dword-only random write | `write_random_u16s`, `write_random_u32s` |
| Extended random word/dword write | `write_random_words_ext` |
| Extended word-only / dword-only random write | `write_random_u16s_extended`, `write_random_u32s_extended` |
| Random bit write | `write_random_bits` |
| Extended random bit write | `write_random_bits_ext` |
| Block read/write | `read_block`, `write_block` |
| Single-kind block read/write | `read_word_blocks`, `read_bit_blocks`, `write_word_blocks`, `write_bit_blocks` |
| Type name | `read_type_name` |

Extended random APIs use the 008x subcommands. Use `parse_qualified_device`
or `SlmpQualifiedDeviceAddress` for routed devices such as `U1\G0`,
`U3E0\HG0`, or `J2\SW10`. Route fields are derived from the qualified address.
Optional Z, LZ, and indirect modification uses `SlmpDeviceModification`.

## Specialized Operations

| Operation | Public API |
| --- | --- |
| Memory command words | `memory_read_words`, `memory_write_words` |
| Extend-unit command words | `extend_unit_read_words`, `extend_unit_write_words` |
| Label array access | `read_array_labels`, `write_array_labels`; use the explicit `_with_abbreviations` variants when definitions are required |
| Label random access | `read_random_labels`, `write_random_labels`; use the explicit `_with_abbreviations` variants when definitions are required |
| Remote CPU control | `remote_run` with `SlmpRemoteMode` and `SlmpRemoteClearMode`, `remote_stop`, `remote_pause`, `remote_latch_clear`, `remote_reset` |
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
| Connection options and profile descriptors | `SlmpConnectionOptions`, `plc_profile_descriptors`, `SlmpPlcProfileDescriptor`, `SlmpTransportMode`, `SlmpFrameType`, `SlmpCompatibilityMode` |
| Address parsing | `SlmpAddress::parse`, `SlmpAddress::try_parse`, `SlmpAddress::format`, `SlmpAddress::normalize`, `parse_device`, `parse_qualified_device` (all parsing requires `SlmpPlcProfile`) |
| Typed values | `read_typed`, `write_typed` |
| Named typed collections | `read_named`, `write_named`, `poll_named` |
| Single-request word/dword reads | `read_words_single_request`, `read_dwords_single_request` |
| Bit-in-word write | `write_bit_in_word` |
| Traffic counters | `traffic_stats` |

## Target Module I/O Constants

`SlmpModuleIo` provides named request-header module I/O numbers for multi-CPU
and routed CPU targets. Use these values in `SlmpTargetAddress.module_io`;
`plc_comm_slmp::SlmpTargetAddress::default()` remains the own-station route `0x03FF`.

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
