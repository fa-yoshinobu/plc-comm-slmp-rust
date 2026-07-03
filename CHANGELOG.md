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
- Library: Added non-breaking SLMP specification-audit updates for manual-conformant request framing, point-limit guards, response correlation, oversized-request rejection, and PLC error diagnostics.
- Library: Exposed structured PLC error information on `SlmpError.error_info` when a non-zero end-code response carries the 9-byte error information block.
- Library: Enforced the documented iQ-F direct bit access limit of 3584 points before transport while keeping the existing 7168-point limit for non-iQ-F profiles.
- Library: Added SLMP `S` step relay device-code support for reads and profile-specific write policy enforcement.
- Library: Rejected standalone `G/HG` access on direct, random, block, and monitor-register routes; callers should use U-qualified extended access.
- Library: Rejected `G/HG` random bit writes and aligned long counter state metadata so `LCS/LCC` remain long-helper entries while using their direct bit-read route internally.
- Library: Fixed U-qualified `G/HG` extended bit reads and writes so the standalone-device guard does not reject valid `Un\Gn` / `Un\HGn` requests before transport.
- Library: Added built-in SLMP capability profiles from `plc-comm-slmp-profiles` v1.0.0 and `SlmpConnectionOptions.strict_profile` (default `true`) so high-level APIs reject profile `blocked` / `unverified` features before transport.
- Library: Added `SlmpErrorKind::ProfileFeature` and `SlmpProfileFeatureErrorInfo` so profile guard failures are distinguishable from PLC end-code errors and include profile ID, feature key, state, evidence, and the `strict_profile=false` bypass hint.
- Library: Moved direct/random point limits to the capability table for all canonical built-in Ethernet profiles, including `melsec:qcpu` and `melsec:qnu`.
- Library: Enforced capability write policies independently of `strict_profile`; `S` is read-only on iQ-R/iQ-L/MX/Q/L profiles and read-write on iQ-F.
- Library: Changed `read_named` for `TS/TC/STS/STC/CS/CC/DX/DY:BIT` to use direct bit reads instead of random word-read batching. R120PCPU live verification showed these device families accept direct bit reads but reject `0x0403` random word reads with end code `0x4032`, so batching remains limited to bit families validated on both mock and real PLC paths.
- Library: Moved Q/L profile Read Block (`0x0406`) and Write Block (`0x1406`) rejection to the capability profile guard so `strict_profile=false` can intentionally send the request and let the PLC answer.
- Docs: Documented profile-specific `S` write policy in supported-register, gotcha, latest-verification, and audit-reflection notes.
- Docs: Recorded the named-bit batching policy difference from Python, .NET, C++ minimal, and Node-RED so `TS/TC/STS/STC/CS/CC/DX/DY` stay off the random word-read path.
- Docs: Documented the Q-series Read Block (`0x0406`) and Write Block (`0x1406`) profile guard in user profiles and gotchas.
- Docs: Fixed recent device-range sample snippets.
- Docs: Cleaned up obsolete maintainer notes and normalized the root TODO.
- Release: Excluded maintainer-only files, scripts, and tests from generated source archives via `.gitattributes`.
- Tests: Added guard coverage for `S` read-only writes, `S10` parsing, standalone `G/HG` random bit write rejection, and U-qualified `G/HG` extended bit routing.
- Tests: Added canonical capability fixture comparison plus strict-profile coverage for qnudv/lcpu block/type-name guards, qnudv `strict_profile=false`, raw request bypass, iQ-F link-direct, iQ-F `U\G`, iQ-L HG, profile limits, and profile write policies.
- Tests: Added `read_named` coverage to keep the live-sensitive bit families above on the direct bit-read path.
- Tests: Added route-guard coverage for Q-series block read/write rejection through the capability profile guard.

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
- Library: Aligned standard 008x extended device specifications with the manual 11-byte Q/L and 13-byte iQ-R layouts.
- Library: Rejected oversized request data lengths before frame generation so the 16-bit request length field cannot silently wrap.
- Library: Matched 4E responses by request serial and discarded mismatched D4 responses before parsing the response payload.
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
