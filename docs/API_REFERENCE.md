# SLMP Rust API Reference

This page is a user-facing index of the public Rust SLMP client surface.
Use the usage guide for examples, and this page when you need to find the
operation name for a specific SLMP command family.

The main async client type is `SlmpClient`.

All TCP and UDP connections are IPv4-only. `SlmpConnectionOptions` accepts an
IPv4 literal or a hostname with an IPv4 result. IPv6 literals and hostnames
without an IPv4 result are rejected without IPv6 fallback.

`SlmpConnectionOptions::timeout` bounds complete connection establishment with
one monotonic absolute deadline. For TCP this includes IPv4 DNS, every selected
address candidate, no-delay/keepalive configuration, and adoption. For UDP it
includes IPv4 DNS, bind, connect, and adoption. IPv4 literals bypass DNS.
Expiry is `SlmpErrorKind::Timeout`; candidate or socket failure completed before
expiry is `SlmpErrorKind::Transport`. A late resolver or socket result is never
adopted. The same timeout value separately bounds each already-connected SLMP
request from its first send through response decoding.

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

Every individual bit-write entry accepts Rust `bool` values only. This includes
direct, extended, random, typed, named, and bit-in-word operations; no numeric
or string compatibility API is exposed. Packed bit-block words are a distinct
wire-level input and remain `u16` values.

All contiguous device operations validate the complete consumed span against
the selected 24-bit Q/L-compatible or 32-bit iQ-R wire address field before
transport. A J-qualified link-direct request always remains a 24-bit Q/L layout,
even on an iQ-R client; other iQ-R Extended Device layouts use 32 bits. Ordinary
word devices consume one address per word and two per
DWord/Float32 value; packed word access to bit devices consumes 16 addresses per
word, and a bit-block point consumes 16 bit addresses. Direct long-timer status
blocks consume one `LTN`/`LSTN` address per four returned words. Random,
monitor-registration, Block, and applicable Extended Device routes follow the
same route-specific widths. This is wire-field validation, not a PLC
device-range policy check.

Semantic unit validation is exact. Bit-unit direct, extended, random, block,
typed, and named APIs accept only bit devices. Typed/named `BIT` accepts only a
bit device, while numeric/string dtypes accept only word devices. Explicit
low-level word APIs may still access a bit device as one packed 16-bit word.
Word-device bit access uses `.n` or `write_bit_in_word`; no implicit mask,
read-modify-write, or route fallback is performed. `write_bit_in_word` requires a
word device and preflights writability before its read request, so an invalid or
non-writable target sends neither half of the explicit two-request sequence.

Extended random APIs use the 008x subcommands. Use `parse_qualified_device`
or `SlmpQualifiedDeviceAddress` for routed devices such as `U1\G0`,
`U3E0\HG0`, or `J2\SW10`. Route fields are derived from the qualified address.
Optional Z, LZ, and indirect modification uses `SlmpDeviceModification`.

## Specialized Operations

| Operation | Public API |
| --- | --- |
| Memory command words | `memory_read_words`, `memory_write_words` |
| Extend-unit command words | `extend_unit_read_words`, `extend_unit_write_words` |
| Monitor registration/cycle | `register_monitor_devices`, `register_monitor_devices_ext`, `run_monitor_cycle` |
| Label array access | `read_array_labels`, `write_array_labels`; use the explicit `_with_abbreviations` variants when definitions are required |
| Label random access | `read_random_labels`, `write_random_labels`; use the explicit `_with_abbreviations` variants when definitions are required |
| Remote CPU control | `remote_run` with `SlmpRemoteMode` and `SlmpRemoteClearMode`, `remote_stop`, `remote_pause`, `remote_latch_clear`, `remote_reset` |
| Remote password | `remote_password_unlock`, `remote_password_lock` |
| CPU operation state | `read_cpu_operation_state` |
| Self-diagnosis error code | `read_latest_self_diagnosis_error_code` |
| Device range catalog | `read_device_range_catalog`, `read_device_range_catalog_for_plc_profile` |
| Self-test loopback | `self_test_loopback` |
| Clear PLC error | `clear_error` |

Device-range catalog calls read one canonical SD-register window. QCPU, LCPU,
QnU, and QnUDV address profiles then use the canonical runtime probes for `Z`
(QCPU only), `ZR`, and derived `R`. A probe's nonzero PLC end code means that
candidate address is unreadable; timeout, transport, protocol, lifecycle, and
local-validation failures are returned without a partial catalog.

Array label `unit_specification` is `0` for a logical bit count and `1` for a
logical byte count. Both forms occupy whole two-byte wire units: bit counts use
`ceil(array_data_length / 16) * 2` bytes and byte counts use
`ceil(array_data_length / 2) * 2` bytes. The logical length must be positive,
and `write_array_labels` requires the exact padded buffer length. Random label
read and write data lengths must also be positive and even. Read responses must
match the requested count and, for array labels, each requested unit and
logical length; malformed or trailing data returns `SlmpError`.

## Request payload limits

TCP command payloads are limited to 65,529 bytes. UDP command payloads are limited to 65,492 bytes
for 3E and 65,488 bytes for 4E so the complete frame fits one datagram. Oversized requests return
`SlmpError` before send, request-frame publication, or 4E serial allocation and are never truncated
or split automatically. Label builders enforce their aggregate size; their largest
protocol-representable even payload is 65,528 bytes.

CPU-buffer convenience helpers are not separate methods; use the
extended-device `U3E0\HG...` route where the selected profile supports
CPU-buffer access.

## High-Level Helpers

| Operation | Public API |
| --- | --- |
| Connection options and profile descriptors | `SlmpConnectionOptions`, `plc_profile_descriptors`, `SlmpPlcProfileDescriptor`, `SlmpTransportMode`, `SlmpFrameType`, `SlmpCompatibilityMode` |
| Address parsing | `SlmpAddress::parse`, `SlmpAddress::try_parse`, `SlmpAddress::format`, `SlmpAddress::normalize`, `parse_device`, `parse_qualified_device` (all parsing requires `SlmpPlcProfile`) |
| Typed values | `read_typed`, `write_typed` |
| Named typed collections | `read_named`, `write_named`, `poll_named` (one random request per call/cycle or pre-transport rejection; a polling stream prepares its immutable request and compact decode indexes once; Direct long-timer routes are excluded) |
| Single-request word/dword reads | `read_words_single_request`, `read_dwords_single_request` |
| Bit-in-word write | `write_bit_in_word` (direct) and `write_bit_in_word_extended` (qualified U/J route); explicit non-atomic RMW under one FIFO turn and one post-admission deadline |
| Traffic counters | `traffic_stats` |
| Errors and timeout classification | `SlmpError`, `SlmpErrorKind`, `SlmpOutcomeUnknownReason`, `SlmpError::is_timeout`, `SlmpError::is_outcome_unknown` |

`SlmpErrorInfo` exposes the correlated error route, command, and subcommand.
Its `raw` field is the required nine-byte prefix; `extra` retains all following
error-data bytes. A prefix that identifies another request is malformed and
invalidates the transport instead of becoming a definitive PLC end code.

One client connection admits ordinary operations in FIFO order and permits one wire transaction at
a time. Its absolute request deadline spans send, receive, correlation, parse, and payload decode.
`close` invalidates incomplete active and queued work for that exact connection. A complete,
correlated, protocol-checked, command-decoded success or PLC end code remains definitive even if a
concurrent `close` or a deadline observed only after decode wins the later transport-state race;
the affected transport is still retired. Separate client instances are independent.
`raw_command` uses the supplied `SlmpCommand` to apply the same conservative state-changing outcome
classification as semantic APIs.

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
