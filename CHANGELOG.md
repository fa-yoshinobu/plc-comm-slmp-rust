# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Entry labels**

- `Release`: Package/version metadata and publishing preparation.
- `Library`: Runtime behavior, public API, protocol handling, or validation in the distributed library.
- `Docs`: README, user guides, generated API docs, or other documentation-only changes.
- `Samples`: Examples, sample flows, sample scripts, or sample applications.
- `Tests`: Test suites, test fixtures, golden vectors, or verification data.
- `Tooling`: Developer/operator command-line tools and helper utilities.
- `CI`: Release checks, workflow scripts, or automation-only changes.

## [Unreleased]

### Changed
- Library: Added SLMP `S` step relay device-code support for reads and rejected writes to `S` as read-only.
- Library: Rejected standalone `G/HG` access on direct, random, block, and monitor-register routes; callers should use U-qualified extended access.
- Library: Rejected `G/HG` random bit writes and aligned long counter state metadata so `LCS/LCC` remain long-helper entries while using their direct bit-read route internally.
- Library: Changed `read_named` for `TS/TC/STS/STC/CS/CC/DX/DY:BIT` to use direct bit reads instead of random word-read batching. R120PCPU live verification showed these device families accept direct bit reads but reject `0x0403` random word reads with end code `0x4032`, so batching remains limited to bit families validated on both mock and real PLC paths.
- Docs: Documented `S` as a read-only bit device in supported-register, gotcha, latest-verification, and audit-reflection notes.
- Docs: Recorded the named-bit batching policy difference from Python, .NET, C++ minimal, and Node-RED so `TS/TC/STS/STC/CS/CC/DX/DY` stay off the random word-read path.
- Docs: Fixed recent device-range sample snippets.
- Tests: Added guard coverage for `S` read-only writes, `S10` parsing, and standalone `G/HG` random bit write rejection.
- Tests: Added `read_named` coverage to keep the live-sensitive bit families above on the direct bit-read path.

## [1.1.1] - 2026-06-29

### Changed
- Release: Bumped crate metadata to `1.1.1` and the `slmp-node` workspace crate to `1.0.2`.
- Docs: Documented explicit named-address dtype requirements in user docs and crate-level examples.
- Samples: Updated named-helper examples to use explicit dtype suffixes.

## [1.1.0] - 2026-06-29

### Changed
- Release: Bumped crate metadata to `1.1.0`.
- Library: Made named-address parsing and typed read/write helpers require explicit dtype suffixes such as `:U`, `:S`, `:D`, `:L`, `:F`, or `:BIT`; bare devices no longer default to `U`, `BIT`, or long-timer `D`.

### Removed
- Library: Removed the embedded localized SLMP end-code message list from the public crate; numeric end codes, deterministic `slmp_end_code_xxxx` keys, and remote-password classification remain available.

### Fixed
- Library: `BIT_IN_WORD` now requires an explicit `.0` through `.F` bit index instead of treating a missing bit index as bit 0.
- Library: Made the typed-read `U` branch explicit so future unsupported dtypes cannot fall through to `U16`.
- Tooling: `slmp_verify_client` now rejects invalid numeric CLI input for ports, targets, counts, write values, block values, label byte values, and label array options instead of silently defaulting or narrowing them.
- Tooling: `slmp_verify_client` now rejects out-of-range values before converting them to `u8`, `u16`, or `u32`.
- Tooling: `slmp_verify_client` now accepts only `0` or `1` for bit write values.

### Tests
- Tests: Added coverage for rejecting bit-in-word named addresses without an explicit bit index.
- Tests: Updated high-level address parser and shared-spec vectors for explicit dtype requirements.
- Tests: Added `slmp_verify_client` coverage for invalid target numbers, invalid count values, and out-of-range label byte values.

## [1.0.1] - 2026-06-25

### Changed
- Release: Bumped crate metadata to `1.0.1`.
- Samples: Examples now require `SLMP_PLC_PROFILE`; they no longer default to `melsec:iq-r` when the PLC profile environment variable is omitted.
- Tooling: `slmp_bench_client` now requires `--plc-profile`; it no longer defaults to `melsec:iq-r`.

### Removed
- Samples: Removed the legacy `SLMP_PLC_FAMILY` and `SLMP_plc_profile` environment-variable aliases from the examples. Use the exact canonical `SLMP_PLC_PROFILE` name instead.

## [1.0.0] - 2026-06-24

### Changed
- Release: Bumped crate metadata, lockfile metadata, and the `slmp-node` workspace crate to `1.0.0` for the first stable release line.

### Fixed
- Library: Reject `remote_run` clear modes outside `0`, `1`, and `2` before building the SLMP request payload.
- Library: Validate named-target `network`, `station`, `module_io`, and `multidrop` fields before narrowing to `u8` or `u16`, preventing silent wraparound for out-of-range values.
