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

- Library: Restored the canonical Q-series device-range runtime probes for QCPU, LCPU, QnU, QnUDV, and their Ethernet-unit profiles. QCPU probes `Z15`; all affected profiles locate the `ZR` boundary and derive `R`, while only PLC end-code responses mean an unreadable candidate and all local, timeout, transport, lifecycle, and protocol failures propagate without a partial catalog.

## [5.0.0] - 2026-08-07

- Library: Named polling now prepares and validates its immutable Random Read payload and compact decode indexes once per stream, then reuses them for every FIFO-controlled cycle without changing timing, cancellation, close, or error behavior.
- Library: Typed command decoders now parse an internal response-frame view; only public raw/trace/error/owned result surfaces materialize bytes. Extended Random and Monitor payloads use a validated exact-size two-pass encoder with one final payload allocation and no per-device encoded buffers.
- Tests: Added allocation/encoding counters and regressions for one-time polling preparation, compact indexed decode, typed/raw response ownership, and exact-size Extended payload construction.
- Library: Prepare and fully validate each general request payload once, then pass its validated bytes and calculated data length to a private frame builder; wire frames, validation order, 4E serial allocation, and transport behavior are unchanged.
- Tests: Count general-request payload validation, preserve frame/serial/statistics state on boundary and malformed monitor-registration rejection, and prove the prepared builder does not revalidate.
- Docs: Made controlled write examples attempt restoration before propagating readback failures, made remote-password examples validate before unlock and attempt re-lock after read failure, separated Clear Error as an explicit state-changing maintenance action, and documented manual reconciliation after restoration failures or outcome-unknown operations.
- Tests: Added compile-checked documentation contract tests for cleanup ordering, password re-lock, controlled extended writes, and Clear Error separation.
- Library: Structured nonzero-end-code responses now require their embedded route, command, and subcommand to match the active request. A mismatch is malformed, retires the transport, and makes a transmitted state change outcome-unknown; matching trailing error data is retained in `SlmpErrorInfo.extra`.
- BREAKING: Contiguous Direct, Random, Monitor-registration, Block, and applicable Extended Device operations now reject a request whose consumed device span exceeds the selected 24-bit or 32-bit wire address field before request-counter mutation or transport. Packed word access to bit devices consumes 16 device numbers per word, ordinary DWord/Float32 access consumes two word devices per value, and bit blocks consume 16 bit devices per block point. Direct `LTN`/`LSTN` status blocks consume one logical device per four wire words, while native Random/Monitor DWord entries for `LTN`/`LSTN`/`LCN`/`LZ` consume one device number. No configured PLC device-range limit is inferred.
- Library: `write_bit_in_word` now requires a writable word-device target and validates it before its read request, so an invalid target sends neither half of the explicit read-modify-write sequence.
- BREAKING: Typed and named semantic units are now exact: `BIT` and every bit-unit operation require a bit device, while numeric/string dtypes require a word device. Explicit low-level word APIs remain available for packed 16-bit access to bit devices.
- BREAKING: `read_named` and `poll_named` reject `LTN`/`LSTN` current values and `LTS`/`LTC`/`LSTS`/`LSTC` states before transport because they require the Direct long-timer route. Use `read_typed` or an explicit long-timer helper.
- Library: Route validation now derives Random Read/Write, Block Read/Write, and LZ coverage from the canonical profile. Q/L Ethernet-unit profiles keep Random and Block coverage while base Q/L profiles omit Block; generic Random validation no longer parses or sends LZ.
- Library: A fully correlated, protocol-checked, command-decoded success or PLC end code now remains definitive when `close` races or the local deadline is observed after decode. The affected transport is still retired; incomplete active operations retain conservative closed, timeout, or outcome-unknown classification.
- Library: TCP and UDP connection establishment now uses one monotonic absolute deadline covering IPv4 DNS, all selected TCP candidates, UDP bind/connect, TCP socket configuration, and final client adoption. Late results are discarded; deadline expiry is Timeout while pre-deadline candidate exhaustion remains Transport.
- Library: Shared-client operations now use one FIFO admission order with one wire transaction at a time. Cancelling a waiting future sends nothing, `close` immediately invalidates active and queued work for the exact connection generation, and separate clients remain independent.
- Library: One monotonic request deadline now covers send, complete TCP/UDP receive, correlation, protocol parsing, and command-specific payload decoding. Timeout, close, cancellation, transport, and malformed-response paths retire the affected transport without resend.
- Library: State-changing commands now return machine-readable `OutcomeUnknown` with a structured `SlmpOutcomeUnknownReason` after any failure that occurs once transmission may have started. PLC end codes remain definitive PLC results.
- Library: `write_bit_in_word` remains an explicit non-atomic two-command RMW, but its read and write now occupy one uninterrupted FIFO client turn.
- Tests: Added deterministic FIFO, waiting-cancellation, close-generation, independent-client, RMW ordering, PLC-NG reuse, malformed-write outcome, and pairwise error taxonomy coverage.
- Release: Aligned artifact roles so the registry package contains consumer runtime, native API metadata, license, README, and ecosystem-native examples where applicable while excluding repository tests and maintainer tooling; the GitHub source archive retains tracked non-hardware validation and maintainer inputs.
- Library: Audited every live API that accepts a profile-bound address and the explicit-profile live catalog assertion: exact canonical identity, including unit-specific profiles, must equal the client profile before request construction or transport activity.
- Tests: Profile-mismatch coverage verifies rejection without reducing unit profiles to their base family.
- Library: Device-range catalogs now use only canonical profile rules and the single required SD-register window. Communication failures are propagated without converting PLC errors into inferred address limits or hidden boundary probes.
- Tests: Added device-range catalog coverage proving canonical values use one SD read and unknown ranges remain unknown without runtime probing.
- Library: Audited every direct, extended, random, named, typed, and bit-in-word write entry: individual bit values remain Rust `bool` values with no numeric, string, truthy, or compatibility entry point. Packed bit-block words remain a separate `u16` wire-level API.
- Docs: README documentation links now include the shared Performance and Choosing a Language pages, and package registry metadata was expanded for discoverability. No functional change.
- Library: Replaced post-1.85 let-chain syntax with equivalent Rust 1.85 control flow so the published crate now compiles at its declared minimum Rust version.
- CI: Added an explicit all-target/all-feature gate for the published crate on Rust 1.85; the non-published N-API workspace binding retains its independent dependency toolchain.
- CI: Generate and extract the actual `.crate`, reject repository tests and maintainer tooling, build every declared Cargo example and rustdoc from the extracted artifact, and compile a separate path consumer using only that package.
- CI: Build current-worktree source archives through an isolated temporary Git index so tracked modifications, non-ignored untracked files, and tracked deletions are all validated without changing the maintainer's real index.
- Docs: Documented Rust 1.85 as the declared minimum supported compiler in the getting-started guide.

### BREAKING

- Library: `write_bit_in_word` and new `write_bit_in_word_extended` cover Direct and qualified U module-buffer / J link-direct complete-word routes, prevalidate the immutable route, own one FIFO turn, and use one absolute post-admission deadline for the mandatory read followed by write. The write is sent even when the bit is unchanged; the pair is not PLC-atomic, never retries, and a possibly transmitted unconfirmed write uses the outcome-unknown error contract.
- Library: `SlmpErrorKind` adds `Cancelled`, `Closed`, `NotConnected`, `Transport`, `MalformedResponse`, and `OutcomeUnknown`; transport and response failures previously grouped under `General` are now distinct. State-changing post-send failures expose `outcome_unknown_reason` and must not be automatically retried.
- Library: TCP and UDP connections are now IPv4-only. IPv6 literals are rejected before socket creation, hostname results are filtered to IPv4 without IPv6 fallback, and callers using IPv6 must migrate to IPv4.
- Library: Array label lengths now use the SLMP bit/byte logical-length contract and two-byte wire padding. Zero logical lengths, non-exact array write buffers, and zero or odd random-label write buffers are rejected before transport.
- Library: Requests that exceed the 16-bit SLMP data-length field or one complete UDP datagram are rejected before frame publication, send, and 4E serial allocation. Oversized label aggregates now return `SlmpError` deterministically.
- Library: Direct and extended bit reads now reject trailing packed bytes and non-binary used nibbles that were previously tolerated.

### Fixed

- Library: Array label reads now accept the documented six-bit/two-byte response shape and reject count, unit, logical-length, truncation, and trailing-data mismatches. Random label reads reject zero or odd result lengths while preserving unknown data type IDs and spare values.
- Library: Enforced command-payload limits of 65,529 bytes over TCP, 65,492 bytes for UDP 3E, and 65,488 bytes for UDP 4E without truncation or automatic splitting.
- Library: Packed bit responses now require exactly `ceil(points / 2)` bytes and require every used nibble to be `0` or `1`.

### Tests

- Tests: Added deterministic delayed-resolver, shared-candidate-deadline, remaining-time success, literal-DNS-bypass, transport-classification, and configured-loopback coverage for connection establishment.
- Tests: Added IPv6-literal rejection and local TCP/UDP hostname coverage for IPv4-only resolution.
- Tests: Added bit and byte boundary vectors, pre-transport validation checks, and malformed label-response coverage.
- Tests: Added TCP/UDP 3E/4E payload boundaries, frame/serial/stat preservation on rejection, and aggregate limits for all four label builders.
- Tests: Added packed-bit regressions for short and trailing data, invalid high/low nibbles, and odd-point padding.

## [4.0.1] - 2026-07-29

- Release: Bumped the main crate and lockfile metadata to `4.0.1`; the non-published `slmp-node` workspace crate remains `1.0.3`.
- Release: GitHub Release drafts now prepend this version's changelog section to generated notes and repair a missing section on workflow reruns.

### BREAKING

- Library: Raw device-spec encoders now return `Result` and reject addresses that exceed the selected legacy or link-direct wire field instead of truncating high bits. Callers must propagate or handle the encoding error.

### Fixed

- Library: Decimal and hexadecimal device parsers reject signs and embedded whitespace, and J-direct network numbers use the stable decimal `0..255` contract.
- Library: Long-timer helpers validate device-family support and the selected wire width before transport.
- Library: LZ DWord helpers and long-timer/long-retentive-timer result generation now validate the complete `u32` device span with checked arithmetic. Overflow is reported as `SlmpError` instead of panicking in checked builds or wrapping to a different address in release builds; a valid single point at `u32::MAX` remains accepted.
- Library: J link-direct extended random read/write and monitor registration now use Q/L subcommands and bit-value encoding; requests mixing J and iQ-R entry layouts are rejected before transport, and qualified-device span validation uses the correct representation width.
- Library: Profile device-range upper bounds are not used as transport send guards.

### Tests

- Tests: Added pre-transport LZ, long-timer family, strict parser, J-network, and boundary coverage, including identical debug/release behavior at `u32::MAX`.

## [4.0.0] - 2026-07-17

- Release: Bumped the main crate and lockfile metadata to `4.0.0`; the `slmp-node` workspace crate remains `1.0.3`.

- Tests: Pinned the existing `traffic_stats()` byte-count and mismatched-response contract used for parity in the other SLMP implementations.
- Library: TCP and UDP now accept a response only when all four target-route fields match the request; 4E responses must also match the request serial. Complete foreign responses are discarded while malformed frames still invalidate the transport.
- Library: Each request now has one absolute timeout covering send, response assembly, and any discarded foreign-route or wrong-serial frames; unrelated responses cannot restart the timeout.
- Library: Added `melsec:mx-r:rj71en71` (`SlmpPlcProfile::MxRRj71En71`) for MX-R through RJ71EN71 connections and pinned profile imports to `v2.1.0`.
- Library: Deadline expiry is now machine-identifiable as `SlmpErrorKind::Timeout` and through `SlmpError::is_timeout()`.
- Breaking: `SlmpErrorKind` adds `Timeout` and is now non-exhaustive; downstream exhaustive matches must include a wildcard arm.
- Breaking: `SlmpPlcProfile` is now non-exhaustive so future canonical profile additions do not repeatedly break downstream exhaustive matches; downstream matches must include a wildcard arm.
- Tests: Added deterministic TCP/UDP 3E/4E route-correlation, wrong-serial flood, within-deadline success, malformed-frame, and transport-isolation coverage.
- Tooling: Refreshed canonical SLMP profile fixtures for 2026-07-14, including `melsec:mx-r:rj71en71` and its device-range rules.

## [3.1.0] - 2026-07-13

- Library: Added public monitor registration/cycle and fixed Clear Error semantic APIs; each call sends exactly one request without registration fallback or retry.
- Library: Monitor cycle expected counts must total at least one and stay within the selected profile's monitor-registration limit.
- Library: Self-test loopback now rejects declared-length, actual-length, trailing-data, and echo mismatches.
- Docs: Clarified explicit monitor counts and that `U3En\HG` never changes or retries the user-selected request target.
- Tooling: Removed cross-repository verification artifacts and their dependent interactive wrapper. Independent cross-implementation checks are not part of this repository's package or release gate.
- Library: Long-timer and long-retentive-timer helpers reject zero, one-request-limit overflow, arithmetic overflow, and `u16` truncation of the required point count before transport.
- Library: Typed writes require the exact matching `SlmpValue` variant and finite float values; CLI/named scalar parsing rejects range overflow instead of truncating or saturating.
- Library: Random and block writes reject duplicate or overlapping device spans before transport, including qualified Extended Device writes.
- Library: Remote RESET closes the transport after its send-only exchange so a delayed 3E response cannot satisfy a later request.
- Library: `write_named` batches one compatible random-write family into exactly one request and rejects mixed families or implicit bit-in-word read-modify-write sequences.
- Library: `read_named` and each `poll_named` cycle accept only entries that fit one random-read request; direct/block/long-timer fallback routes are rejected before transport.
- Node binding: `normalizeAddress` rejects abstract base-only profiles as well as non-canonical labels.

### BREAKING
- Library: Qualified-address wire fields are private and exposed read-only through validated semantic constructors; `SlmpQualifiedDeviceAddress` no longer implements `serde::Deserialize`, because deserialization could bypass those constructors. Use `parse_qualified_device` or the validated semantic constructors. LZ index modification accepts only LZ0 and LZ1.
- Node binding: `normalizeAddress` now requires the exact canonical PLC profile label as its second argument. Address radix and supported device families are never inferred by the binding.
- Library: `SlmpConnectionOptions::new` now requires destination port, transport, complete target route, and canonical PLC profile. Port zero is rejected; no destination or transport is inferred.
- Library: Address parsing, formatting, normalization, qualified-device parsing, and numeric semantic address construction are profile-bound. `SlmpDeviceAddress` now keeps its profile, code, and wire number in private immutable fields exposed through read-only accessors; a semantic address from another profile is rejected before transport use.
- Library: Removed public automatic chunking, mixed-block splitting, localized end-code messages, public strict-profile bypass, and the response-optional raw request surface. `raw_command` always requires command, subcommand, and payload and always returns the response bytes.
- Library: Extended Device operations derive wire fields from the qualified semantic address. Normal APIs no longer accept `SlmpExtensionSpec`; typed Z, LZ, and indirect modifiers remain explicit, and `with_modification` now returns `Result` after validating the combination.
- Library: Remote RUN and PAUSE require typed mode values; Remote RUN also requires a typed clear mode. Remote RESET has fixed protocol data and no response-mode argument.
- Library: Random and block aggregate APIs provide category-specific methods so unused categories can be omitted, while an all-empty aggregate request is rejected before transport.
- Library: `read_named` no longer splits an oversized random-read batch into multiple requests. It reports the selected profile limit before sending, preventing a multi-time snapshot from being returned as one result.
- Library: Standard label APIs no longer accept an abbreviation table. Use the explicitly named `*_with_abbreviations` variants when abbreviations are required.
- Samples/Tooling: Executable environment, JSON, monitor, benchmark, and verification inputs require port, transport, target route, and profile instead of supplying runnable endpoint defaults. Named reads require an explicit dtype.
- Tooling: The interactive read/write walk now requires explicit `--host` and `--port`; it no longer supplies a runnable PLC endpoint.

### Added
- Library: Added `SlmpPlcProfileDescriptor` and crate-root `plc_profile_descriptors()` for canonical SLMP profile metadata.

### Changed
- Library: The default communication timeout is 3 seconds, the monitoring timer is 4 seconds (`0x0010`), and TCP keepalive idle time is 30 seconds.
- Tooling: The benchmark client's omitted operation/communication timeout now uses the same 3000 ms default instead of 2000 ms.
- Library: 4E serial numbers are assigned under the client request lock and matched against responses. Timeout, receive failure, or external Rust future cancellation invalidates the in-flight TCP/UDP socket so partial or delayed data cannot satisfy a later request.
- Docs: Updated migration, routing, single-request consistency, profile-bound address, Extended Device, remote-control, and executable-example guidance for the new contract.

- Release: Bumped the main crate metadata to `3.1.0`; the `slmp-node` workspace crate remains `1.0.3`.
- Tooling: Pinned canonical SLMP profile imports to published profile tag `v2.0.0`.

### Tests
- Tests: Added concurrent request serialization/serial-number coverage, UDP timeout invalidation, external-cancellation invalidation, profile-mismatch pre-transport rejection (including long timers), aggregate empty-input rejection, strict typed write values, duplicate/overlap rejection, typed Extended Device modifiers and LZ bounds, RESET transport invalidation, single-request named writes, explicit remote-control wire values, and approved connection default checks.

## [3.0.0] - 2026-07-10

### Changed
- Release: Bumped the main crate metadata to `3.0.0`; the `slmp-node` workspace crate remains `1.0.3`.
- Packaging: Included LICENSE and CHANGELOG.md in the published crate.
- Docs: Replaced relative README links with absolute URLs so they resolve on package registry pages.

### BREAKING
- Library: Breaking: `SlmpConnectionOptions::new` and `set_plc_profile` now return `Result` for invalid connection profiles instead of panicking.
- Migration: Handle the returned `Result` with `?` or explicit matching. Update callers that mutate options after construction to unwrap or propagate construction errors before accessing fields.

### Added
- Library: Added `SlmpPlcProfile::available_connection_profiles()` for selectors that must exclude the base-only `melsec:qcpu` profile.

### Changed
- Library: Renamed the internal device-range model-label helper to clarify that it is not a canonical profile identifier.
- Docs: Updated connection examples and profile guidance for the recoverable error API.

## [2.0.0] - 2026-07-06

### BREAKING
- Release: Renamed the crates.io package and standardized the Rust import path.

| Old crate/use | New crate/use |
| --- | --- |
| `plc-comm-slmp-rust` | `plc-comm-slmp` |
| `use plc_comm_slmp_rust::...` | `use plc_comm_slmp::...` |

- Library: Removed short `SlmpModuleIo` aliases in favor of the canonical module I/O vocabulary.

| Removed name | Use instead |
| --- | --- |
| `CONTROL_CPU`, `CONNECTED_CPU`, `DEFAULT` | `OWN_STATION` |
| `ACTIVE_CPU` | `CONTROL_SYSTEM_CPU` |
| `STANDBY_CPU` | `STANDBY_SYSTEM_CPU` |
| `TYPE_A_CPU` | `SYSTEM_A_CPU` |
| `TYPE_B_CPU` | `SYSTEM_B_CPU` |
| `CPU1` / `CPU_1` to `CPU4` / `CPU_4` | `MULTIPLE_CPU_1` to `MULTIPLE_CPU_4` |
| `SELF-CPU1` to `SELF-CPU4` | `SELF-MULTIPLE-CPU-1` to `SELF-MULTIPLE-CPU-4` |

### Changed
- Release: Bumped package metadata to `2.0.0`.
- Library: Added `SlmpModuleIo` named constants for multi-CPU target routing while keeping the default own-station target unchanged.
- Library: Added extended random read/write APIs and synced the embedded SLMP capability fixture to `plc-comm-slmp-profiles` `v1.2.2`.
- Docs: Updated README, Getting Started, docs.rs links, and release duplicate checks for `plc-comm-slmp`.
- Tooling: Changed the canonical profile update script default ref to `v1.2.2`.

## [1.2.0] - 2026-07-05

### Changed
- Release: Bumped package metadata to `1.2.0`.
- Tooling: Normalized line-ending handling in the canonical profile JSON update script so `-SourceRoot` runs no longer report false changes.
- Library: Synced the embedded SLMP capability fixture to `plc-comm-slmp-profiles` `v1.2.1`, including `display_name` labels and Ethernet unit profiles for RJ71EN71, LJ71E71-100, and QJ71E71-100 variants.
- Library: Added `SlmpPlcProfile::display_name()` as the public UI-label helper while keeping stored PLC profile values canonical.
- Docs: Documented the profile display-name helper and canonical-ID storage guidance.
- Tests: Added canonical fixture parity coverage for profile `display_name` values.
- Samples: Added read-only Rust `multi_plc_monitor` and `config_polling` operational recipes with dry-run validation, reconnect backoff, and JSON config.
- Library: Added non-breaking SLMP specification-audit updates for manual-conformant request framing, point-limit guards, response correlation, oversized-request rejection, and PLC error diagnostics.
- Library: Exposed structured PLC error information on `SlmpError.error_info` when a non-zero end-code response carries the 9-byte error information block.
- Library: Enforced the documented iQ-F direct bit access limit of 3584 points before transport while keeping the existing 7168-point limit for non-iQ-F profiles.
- Library: Refreshed the built-in capability table to `plc-comm-slmp-profiles` `v1.1.0` with explicit 008x extended random/monitor limit keys.
- Library: Added SLMP `S` step relay device-code support for reads and profile-specific write policy enforcement.
- Tooling: Changed the canonical profile update script default ref from `v1.0.0` to `v1.1.0`.
- Library: Rejected standalone `G/HG` access on direct, random, block, and monitor-register routes; callers should use U-qualified extended access.
- Library: Rejected `G/HG` random bit writes and aligned long counter state metadata so `LCS/LCC` remain long-helper entries while using their direct bit-read route internally.
- Library: Fixed U-qualified `G/HG` extended bit reads and writes so the standalone-device guard does not reject valid `Un\Gn` / `Un\HGn` requests before transport.
- Library: Added built-in SLMP capability profiles from `plc-comm-slmp-profiles` v1.0.0 and `SlmpConnectionOptions.strict_profile` (default `true`) so high-level APIs reject profile `blocked` / `unverified` features before transport.
- Library: Added `SlmpErrorKind::ProfileFeature` and `SlmpProfileFeatureErrorInfo` so profile guard failures are distinguishable from PLC end-code errors and include profile ID, feature key, state, evidence, and the `strict_profile=false` bypass hint.
- Library: Moved direct/random point limits to the capability table for all canonical built-in Ethernet profiles, including `melsec:qcpu` and `melsec:qnu`.
- Library: Added canonical weighted random-word write limits for `melsec:iq-l` and `melsec:iq-f`, so mixed word/dword random writes are guarded before transport.
- Library: Enforced capability write policies independently of `strict_profile`; `S` is read-only on iQ-R/iQ-L/MX/Q/L profiles and read-write on iQ-F.
- Library: Rejected profile-unsupported device families before transport while leaving device address upper-bound checks to application/live-probe code.
- Library: Changed `read_named` for `TS/TC/STS/STC/CS/CC/DX/DY:BIT` to use direct bit reads instead of random word-read batching. R120PCPU live verification showed these device families accept direct bit reads but reject `0x0403` random word reads with end code `0x4032`, so batching remains limited to bit families validated on both mock and real PLC paths.
- Library: Changed `read_named` random-read batching to use each profile's canonical `random_read_word.max` instead of a fixed 96-entry batch size.
- Library: Moved Q/L profile Read Block (`0x0406`) and Write Block (`0x1406`) rejection to the capability profile guard so `strict_profile=false` can intentionally send the request and let the PLC answer.
- Docs: Documented profile-specific `S` write policy in supported-register, gotcha, latest-verification, and audit-reflection notes.
- Docs: Recorded the named-bit batching policy difference from Python, .NET, C++ minimal, and Node-RED so `TS/TC/STS/STC/CS/CC/DX/DY` stay off the random word-read path.
- Docs: Documented the Q-series Read Block (`0x0406`) and Write Block (`0x1406`) profile guard in user profiles and gotchas.
- Docs: Removed duplicated SLMP supported-register and device-range user pages and linked users to the shared SLMP Profile Reference.
- Docs: Added a Usage Guide example showing how to read `SlmpError.end_code` and structured `error_info`.
- Docs: Added Usage Guide examples for `U...` module access, `U...HG` CPU-buffer access, and `J...` link direct extended devices.
- Docs: Removed the manual page-navigation block from Getting Started and rely on site navigation instead.
- Docs: Moved shared SLMP gotcha items to the common troubleshooting page and kept Gotchas focused on Rust-specific behavior.
- Docs: Slimmed Gotchas to library-specific items and moved shared setup/end-code symptoms to the PLC Setup Guide.
- Docs: Standardized the Gotchas page structure with KV Host Link so library-specific caveats have the same destination across protocols.
- Docs: Merged runnable example recipes into the Usage Guide and removed the standalone Recipes page.
- Docs: Fixed recent device-range sample snippets.
- Docs: Cleaned up obsolete maintainer notes and normalized the root TODO.
- Release: Excluded maintainer-only files, scripts, and tests from generated source archives via `.gitattributes`.
- Tooling: Changed the canonical profile update script default ref from `main` to fixed tag `v1.0.0`; `SLMP_PROFILES_REF` can still override it.
- Tests: Added guard coverage for `S` read-only writes, `S10` parsing, standalone `G/HG` random bit write rejection, and U-qualified `G/HG` extended bit routing.
- Tests: Added canonical capability fixture comparison plus strict-profile coverage for qnudv/lcpu block/type-name guards, qnudv `strict_profile=false`, raw request bypass, iQ-F link-direct, iQ-F `U\G`, iQ-L HG, profile limits, and profile write policies.
- Tests: Added `read_named` coverage to keep the live-sensitive bit families above on the direct bit-read path.
- Tests: Added `read_named` batching coverage proving Q/L-compatible profiles can use a larger random-read batch while iQ-R remains split at 96.
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

### Tests
- Tests: Added coverage for rejecting bit-in-word named addresses without an explicit bit index.
- Tests: Updated high-level address parser tests for explicit dtype requirements.

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
