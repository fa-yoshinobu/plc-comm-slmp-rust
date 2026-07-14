# Rust SLMP quality-overhaul decision and acceptance record

This maintainer record maps the approved workspace decisions to the Rust implementation. It is not a user guide. Breaking compatibility is intentional where it conflicts with an explicit, profile-safe, single-request contract.

The optional Node binding follows the same rule:
`normalizeAddress(address, plcProfile)` requires the exact canonical PLC
profile label. There is no profile-free compatibility overload.

## Per-decision target contracts

Each item uses the following evidence boxes. A box may be checked only after the repository contains the implementation and executable evidence.

### D-001 — Destination port is required

- Scope: connection options, executable examples, verification and benchmark tools.
- Target: callers must supply a port in `1..=65535`; zero and omission are never replaced with `1025` or `1035`.
- Compatibility: constructors and runnable configurations that relied on a port default break.
- Acceptance: constructor rejects zero, all runnable endpoint definitions require a port, and no socket is opened for invalid input.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-002 — Transport is required

- Scope: connection options and all runnable endpoint inputs.
- Target: callers explicitly choose TCP or UDP; omission and unknown text do not become TCP.
- Compatibility: callers and configurations that relied on TCP defaulting break.
- Acceptance: constructor has a non-optional `SlmpTransportMode`, CLI/environment/JSON inputs reject omission, and examples state the requirement.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-003 — Communication timeout defaults to 3 seconds

- Scope: TCP and UDP connection options.
- Target: omitted timeout is exactly 3 seconds; explicit zero is rejected before communication.
- Compatibility: the previous timeout value, if different, changes.
- Acceptance: option-construction test asserts 3 seconds and connect validation rejects zero. Benchmark configuration omission also selects 3000 ms; its intentional timeout-failure scenario may still set a shorter explicit duration.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-004 — Complete target route is required

- Scope: connection constructor, environment examples, JSON polling, monitor, verifier, and benchmark.
- Target: network, station, module I/O, and multidrop are supplied as one explicit target; `SlmpTargetAddress::default()` is permitted only when the caller deliberately passes it.
- Compatibility: omitted routes no longer become the own-station route.
- Acceptance: constructor requires `SlmpTargetAddress`; runtime parsers require either a named target or all four values and reject partial input.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-005 — Monitoring timer defaults to 4 seconds

- Scope: all normal requests.
- Target: omitted monitoring timer is `0x0010` in 250 ms units.
- Compatibility: previous defaults, if different, change.
- Acceptance: option-construction and golden-frame tests assert `0x0010`.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-006 — Strict profile validation is standard and the bypass is internal

- Scope: capability guards.
- Target: normal users cannot disable strict profile validation; maintainers can still perform controlled internal evidence work.
- Compatibility: public `strict_profile` mutation is removed.
- Acceptance: field is crate-private, normal APIs enforce profile guards, and public docs do not advertise the bypass.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-009 — TCP keepalive defaults to 30 seconds

- Scope: TCP transport only.
- Target: keepalive idle defaults to 30 seconds; UDP ignores it; explicit zero is invalid when enabled.
- Compatibility: TCP socket behavior changes on platforms that support the option.
- Acceptance: option test asserts 30 seconds and TCP setup rejects an enabled zero duration.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-011 / D-012 — Raw subcommand and payload are required

- Scope: the single generic maintainer raw-command API.
- Target: `raw_command(command, subcommand, payload)` requires all three values, including an explicitly empty payload.
- Compatibility: omitted subcommand/payload calls no longer compile.
- Acceptance: public signature has three non-optional arguments and raw command frame tests cover explicit values.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-013 — 4E serial is automatic, unique, and validated

- Scope: 4E request serialization and response matching.
- Target: serial numbers are allocated inside the per-client request lock, wrap as 16-bit values, and must match the response.
- Compatibility: callers do not supply serial values.
- Acceptance: concurrent-call test observes unique sequential serials with no pipelining and mismatch tests reject wrong responses.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-018 — Series is profile-derived

- Scope: normal and remote-password operations.
- Target: frame, compatibility, address series, and password representation derive from the required canonical PLC profile.
- Compatibility: per-request series overrides are not available in semantic APIs.
- Acceptance: profile fixtures produce their expected frame/subcommand/password forms without a user series argument.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-019 / D-020 — Unused random categories may be omitted

- Scope: random read and word/dword random write, including Extended Device forms.
- Target: category-specific methods omit unused lists; an aggregate request with every category empty fails before transport.
- Compatibility: callers may migrate from explicit empty placeholders to category-specific methods.
- Acceptance: specialized word/dword methods exist and an all-empty test records zero requests.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-021 / D-022 — Unused block categories may be omitted

- Scope: word/bit block read and write.
- Target: category-specific methods omit the unused category; an aggregate with no blocks fails before transport.
- Compatibility: callers may migrate from explicit empty placeholders to category-specific methods.
- Acceptance: specialized word/bit methods exist and all-empty read/write fail before transport.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-023 / D-037 — Mixed blocks are never silently split

- Scope: block read and write.
- Target: one API call emits one mixed-block request; the library does not split it into word and bit requests.
- Compatibility: `split_mixed_blocks` behavior is removed.
- Acceptance: no public split flag exists and mixed-block tests observe one request.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-024 / D-025 / D-026 — Remote RUN/PAUSE intent is explicit

- Scope: remote RUN and PAUSE.
- Target: `SlmpRemoteMode` is required for RUN/PAUSE and `SlmpRemoteClearMode` is additionally required for RUN.
- Compatibility: calls relying on normal/no-clear defaults no longer compile.
- Acceptance: public signatures require enums and wire tests cover every clear mode and both operation modes.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-027 / D-028 — Remote RESET is fixed and does not wait for a normal response

- Scope: remote RESET.
- Target: the semantic API accepts no subcommand or response flag; it sends command `0x1006`, subcommand `0x0000`, payload `0x0001`, then completes after send.
- Compatibility: configurable reset arguments are removed.
- Acceptance: captured wire data matches the fixed request and the test server need not send a response.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-029 — Remote-password series is profile-derived

- Scope: remote password lock/unlock.
- Target: callers provide only the password; payload form derives from the connection profile.
- Compatibility: series overrides are removed from the semantic API.
- Acceptance: profile-specific fixtures validate fixed/variable payload forms with no series argument.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-030 — Label abbreviations are optional but discoverable

- Scope: random label read/write.
- Target: standard methods send zero abbreviations; explicitly named `*_with_abbreviations` methods expose the protocol feature.
- Compatibility: the standard method no longer takes an empty abbreviation argument.
- Acceptance: both standard and explicit variants are present and wire tests distinguish zero and nonzero abbreviation counts.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-031 / D-032 / D-033 — Module and long-timer operands are required

- Scope: Extend Unit and long timer/retentive timer helpers.
- Target: module number, head number, and multi-point count are explicit; head zero is accepted only when explicitly supplied, while the count must be positive.
- Validation: long-timer counts are multiplied by four with checked arithmetic and must fit both the protocol word-count field and the active profile's one-request direct-word limit; truncation or wraparound never creates a smaller request.
- Compatibility: calls relying on module zero, head zero, or one-point defaults break.
- Acceptance: signatures require the operands and range/zero-count checks run before transport.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-034 — One generic maintainer raw-command surface

- Scope: raw command access.
- Target: command-specific raw wrappers are not public; semantic methods generate fixed/validated payloads and `raw_command` is the sole arbitrary-command escape hatch.
- Compatibility: callers of command-specific raw wrappers migrate to semantic APIs or the explicit generic raw command.
- Acceptance: public-surface scan finds no command-specific raw payload wrappers.
- Maintainer diagnostics: `last_request_frame`, `last_response_frame`, `RawSlmpDeviceAddress`, and `encode_raw_device_spec` support controlled frame investigation and are intentionally omitted from ordinary user documentation.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-035 / D-036 — No automatic splitting or chunked API

- Scope: continuous word/dword reads and writes, named reads/polls, and named writes.
- Target: one call emits at most one request; counts above the profile limit fail before transport; no chunk helper is public.
- Compatibility: chunked APIs and automatic-split options are removed.
- Acceptance: public-surface scan finds no chunk methods; limit tests observe zero transport requests for 961 words / 481 dwords; named reads/polls and writes emit one random request or reject the complete operation before transport.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-038 — No localized end-code manual text

- Scope: error model and public exports.
- Target: preserve numeric end code, stable key, command/subcommand, and classification without embedded localized manual prose or language selection.
- Compatibility: message/language helpers are removed.
- Acceptance: public-surface scan finds no localized message API and tests retain deterministic numeric/key information for known and unknown codes.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-041 — Every public Rust address conversion requires a profile

- Scope: parse, try-parse, normalize, format, named normalization, and qualified parsing.
- Target: textual device conversion cannot choose radix or family support without an explicit profile; formatting uses the profile bound to the semantic address.
- Compatibility: profile-free overloads are removed.
- Acceptance: iQ-F `X10` becomes numeric 8, iQ-R `X10` becomes numeric 16, iQ-F `X1A` fails, and round trips preserve canonical text.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-042 — Extended Device normal APIs are semantic

- Scope: continuous/random Extended Device access.
- Target: qualified addresses derive extension/direct-memory fields; only typed `IndexZ`, `IndexLz`, and `Indirect` modifiers are exposed to normal callers.
- Compatibility: public `SlmpExtensionSpec` arguments are removed.
- Acceptance: public-surface scan finds no public spec, qualified routes encode expected fields, and invalid modifier combinations fail before transport.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### D-047 — Semantic addresses are profile-bound; raw addresses are separate

- Scope: `SlmpDeviceAddress`, qualified addresses, client entry points, and maintainer encoding.
- Target: semantic addresses immutably retain their profile, device code, and wire number; read-only `plc_profile()`, `code()`, and `number()` accessors expose those values. A client rejects a mismatched profile before transport; profile-free wire numbers use `RawSlmpDeviceAddress` only with the raw encoder.
- Compatibility: `SlmpDeviceAddress::new(code, number)` becomes `new(code, number, profile)`. Direct field reads migrate to accessors, and direct field mutation is no longer possible; callers construct a new semantic address when any component changes.
- Acceptance: mismatch test records zero requests, matching iQ-F/iQ-R radix tests pass, normal APIs do not accept the raw address type, and a compile-fail doctest proves the wire number cannot be mutated after construction.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

### SLMP-C05 — Strict typed scalar writes

- Scope: `write_typed`, named/CLI scalar parsing, and `SlmpValue` conversion.
- Target: each dtype accepts only its exact `SlmpValue` variant; float values are finite and textual numbers are range checked before transport.
- Compatibility: cross-type coercion, integer truncation, float saturation, and unknown Boolean text are rejected.
- Acceptance: (1) U/S/D/L/F/BIT mismatches fail with request count 0; (2) all integer boundaries use checked conversion; (3) non-finite F and ambiguous Boolean text fail.
- [x] Implementation completed.
- [x] Tests added for every acceptance condition.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed; ordinary `cargo package` stopped only at its uncommitted-change guard.

### SLMP-C06 — Send-only RESET invalidates transport

- Scope: TCP/UDP `expect_response=false` exchange used by Remote RESET.
- Target: after the RESET frame is sent, the client remains closed and cannot consume a delayed 3E response.
- Compatibility: the same client cannot issue a later request; callers create a new connection and verify PLC state.
- Acceptance: (1) fixed RESET frame is sent once; (2) the next request fails closed without another send; (3) TCP and UDP transports are not restored.
- [x] Implementation completed.
- [x] Regression test added for the public RESET path.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed.

### SLMP-C11 — Ambiguous write ranges are rejected

- Scope: regular/Extended random word and bit writes and block writes.
- Target: duplicate addresses and overlapping word/DWord/block spans fail before transport.
- Compatibility: last-writer-wins requests that were previously encoded are rejected.
- Acceptance: (1) duplicate word/bit entries fail; (2) word-to-DWord and DWord-to-DWord overlap fails; (3) block overlap fails; (4) qualified route identity participates in Extended Device comparison.
- [x] Implementation completed.
- [x] Tests added for every range category.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed.

### SLMP-C26 — Qualified wire fields are not publicly constructible

- Scope: `SlmpQualifiedDeviceAddress`.
- Target: semantic constructors derive private extension/direct-memory fields and read-only accessors expose them; raw contradictory struct literals are unavailable.
- Compatibility: public field initialization and direct field reads migrate to constructors/accessors; `with_modification` returns `Result`.
- Acceptance: (1) compile-fail coverage prevents a raw struct literal; (2) U/J constructors derive the expected fields; (3) invalid modification combinations fail before transport.
- [x] Implementation completed.
- [x] Compile-fail and runtime coverage added or updated.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed.

### SLMP-C27 — Long-timer typed reads preserve profile binding

- Scope: the long-timer fast path in `read_typed`.
- Target: the supplied semantic address profile must equal the client profile before its number is reused by a specialized route.
- Compatibility: cross-profile long-timer addresses are rejected instead of silently rebound.
- Acceptance: a mismatched LTN address fails before transport with request count 0.
- [x] Implementation completed.
- [x] Regression test added.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed.

### SLMP-C28 — Named writes are single-request-or-reject

- Scope: `write_named`.
- Target: one compatible random-write family is emitted as one request; mixed bit/word families and implicit bit-in-word RMW are rejected before transport.
- Compatibility: callers explicitly sequence multiple families and call `write_bit_in_word` when RMW is intended.
- Acceptance: (1) multiple same-family entries produce one request; (2) mixed families send zero requests; (3) bit-in-word sends zero requests.
- [x] Implementation completed.
- [x] Request-count regression tests added.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed.

### SLMP-C29 — Node normalization rejects base-only profiles

- Scope: `slmp-node` `normalizeAddress`.
- Target: only canonical, connection-selectable profiles are accepted, matching the .NET contract.
- Compatibility: `melsec:qcpu` is rejected; use its concrete module profile.
- Acceptance: canonical concrete profile succeeds and base-only `melsec:qcpu` fails.
- [x] Implementation completed.
- [x] Node crate unit test updated.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed.

### SLMP-C30 — LZ index is limited to LZ0/LZ1

- Scope: qualified address modification construction and final wire resolution.
- Target: LZ index 0 and 1 are accepted; index 2 or greater fails before transport.
- Compatibility: invalid `IndexLz(2..=255)` operands no longer reach the PLC.
- Acceptance: (1) LZ1 derives the correct flag/index; (2) LZ2 fails in the semantic constructor; (3) final resolver retains fail-closed validation.
- [x] Implementation completed.
- [x] Boundary regression coverage added.
- [x] Local CI, Node tests, doctests, Clippy, and dirty-tree package verification passed.

## Batch verification and independent review

- [x] Relevant formatting, Clippy, unit, integration, doctest, example, and package checks passed; exact commands/results recorded below.
- [x] Codex self-review completed against the actual diff, public API surface, validation order, errors, transport state, timeout/cancellation behavior, tests, examples, generated documentation, packaging, and cross-language contract.
- [x] Claude source review completed by the user through Claude CLI and recorded in the workspace disposition file.
- [x] Codex accepted and resolved Rust findings 5/6/11/26/27/28/29/30 and reran all affected checks.
- [x] Required live-PLC checks passed, or each unavailable check has an explicit release disposition.
- [x] Final acceptance criteria verified and this Rust batch marked complete.

### Transport cancellation safety

- Scope: TCP and UDP request futures.
- Target: the socket is owned by the in-flight exchange and restored to the client only after a complete successful exchange. Internal timeout, transport failure, parse-boundary failure, or external future cancellation leaves the client closed.
- Compatibility: after cancellation, callers must create/connect a new client instead of reusing a potentially contaminated socket.
- Acceptance: external-timeout test sends one request, cancels before response, and proves a second request is rejected without another send; UDP delayed-response coverage proves the same closed-state rule after receive timeout.
- [x] Implementation completed.
- [x] Acceptance tests/checks completed.
- [x] Documentation and migration note completed.

## Verification evidence

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
- `cargo test --all-features`: PASS, 124 tests including doctests; zero failed/ignored.
- `cargo doc --no-deps --all-features`: PASS.
- `cargo package --allow-dirty`: PASS; 61 files, package verification compiled successfully.
- `config_polling --dry-run` and `multi_plc_monitor --dry-run`: PASS with explicit port, transport, target, profile, and dtype; no PLC communication.
- Public-surface/stale-name scan: no public chunk helpers, public `SlmpExtensionSpec`, localized end-code message API, public strict-profile bypass, response-optional raw request, or stale environment endpoint defaults.
- `git diff --check`: PASS; line-ending conversion notices only, no whitespace errors.

Local mock servers and golden frames provide API, validation, serialization, response matching, cancellation invalidation, UDP invalidation, and request-count evidence. No live PLC communication was authorized or performed.

## Live-PLC disposition

The contract changes in this record are API-shape, pre-transport validation, deterministic frame construction, or host transport-state behavior and therefore do not currently require a physical PLC to establish implementation correctness. Existing profile/device compatibility evidence is not upgraded by this work. Any later physical compatibility check remains `unverified` until separately proposed and explicitly authorized.

## Claude review package status

The approved decisions, repository diff, and final local results were reviewed in the two user-authorized SLMP Claude batches. Canonical results and dispositions are preserved under `D:\APP\Close\instructions`.

## 2026-07-12 D-128 through D-132 delta

### D-128 — Public monitor APIs

- Scope: `register_monitor_devices`, `register_monitor_devices_ext`, and `run_monitor_cycle`.
- Target: Word/DWord registration and cycles are typed single requests with explicit, nonzero, profile-bounded cycle counts; no implicit registration, retry, split, or fallback occurs.
- Compatibility: this is additive. Running before PLC registration sends one request and exposes the PLC result.
- Acceptance: exact commands, qualified subcommand, zero/over-limit rejection, three cycles, PLC NG, and response-size mismatch are covered.

### D-129 — Exact self-test echo

- Scope: `self_test_loopback`.
- Target: 1–960 ASCII `0-9/A-F`; declared length, exact response size, and echo must all match.
- Compatibility: malformed echoes previously accepted through prefix decoding now fail.
- Acceptance: valid, invalid input, declared-length, trailing-data, and payload mismatch cases are covered.

### D-131 — Clear Error semantic API

- Scope: `clear_error`.
- Target: one fixed `0x1617/0x0000` empty-payload request with normal error propagation.
- Compatibility: additive replacement for maintainer raw-command use.
- Acceptance: exact request shape and single-request boundary are covered.

### D-132 — HG target ownership

- Scope: qualified Extended Device HG writes.
- Target: preserve the connection target exactly; never derive it from `U3En`, retry another CPU, or read back automatically. Cross-CPU reads remain allowed.
- Compatibility: applications create/select the client target explicitly when a write must reach another CPU.
- Acceptance: Own Station remains `0x03FF` for `U3E1\HG`; only an explicitly CPU No.2 client emits `0x03E1`.

- [x] Local implementation and regression tests completed.
- [x] Formatting, Clippy, full tests, doctests, Node crate, and local CI passed.
- [x] User API, migration, changelog, and shared target guidance updated.
- [x] Claude review of this delta completed through `CLAUDE-SLMP-20260712-02`; all findings were dispositioned and affected checks rerun.
- [x] New public-API verification completed through deterministic regression coverage and the approved D-128/D-129/D-131 live checks.
- [x] D-132 Extend Unit versus HG physical-area classification completed and recorded in the closed cross-implementation comparison.

## NR-006: Lifetime traffic-statistics reference contract

Scope: Rust reference implementation and cross-language contract for the next release.

Target contract: `traffic_stats().await` returns a client-lifetime immutable snapshot. A request
and its full frame bytes count only after a complete transport send succeeds. A complete received
TCP response counts after assembly in the selected frame format; a UDP datagram counts on receipt.
Both count before serial, end-code, or payload validation. Unrecognized TCP subheaders, partial
sends/receives, and pre-send failures do not count. Close/reconnect does not reset counters.

Acceptance criteria:

- [x] Existing Rust implementation and executable tests match the target contract.
- [x] User API and usage documentation state the counter meaning and lifetime.
- [x] Live PLC verification is unnecessary because deterministic transports observe every boundary.
- [ ] Final next-release package and cross-language API comparison completed.

## QREV-20260714-002: Correlate responses with the complete request target

Scope: Rust TCP and UDP transports for both 3E and 4E binary frames.

Target contract: after a structurally complete response frame is received, its network, station,
module I/O, and multidrop fields must exactly match the target encoded in the request. A complete
foreign-route frame is discarded and the same request continues waiting. A malformed frame is a
protocol failure and invalidates the in-flight transport. For 4E, both the target and serial number
must match before the response is accepted.

Compatibility impact: responses from a route other than the caller-selected route are no longer
returned as successful PLC results. No public API signature changes.

Acceptance criteria:

1. TCP and UDP tests cover every target-field mismatch independently for both 3E and 4E, followed
   by a matching response that is returned from the original request.
2. A malformed response remains a protocol error and the failed transport cannot be reused.
3. 4E requires both the complete target and the serial number to match.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Formatting, Clippy, unit/integration tests, docs, Node binding, and package checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency.
- [x] Claude source review completed and findings recorded in `D:\APP\claude_review_findings_20260714.md`.
- [x] Codex resolved or dispositioned every Claude finding and reran affected checks.
- [x] Live-PLC verification is not required; deterministic TCP/UDP peers exercise the wire contract.
- [x] Documentation, migration notes, changelog, and generated API reference agree.
- [x] Final acceptance criteria verified and the item marked complete.

## QREV-20260714-003: Use one absolute deadline for each request

Scope: Rust TCP and UDP send/receive handling, including discarded foreign-route and wrong-serial
responses.

Target contract: one absolute deadline begins before the request send and bounds the complete
exchange. Every send and receive uses only the remaining time. Discarding any otherwise valid
foreign-route or wrong-serial frame never restarts or extends the deadline. Timeout, cancellation,
I/O failure, or malformed framing leaves the transport closed so partial or delayed data cannot
satisfy a later request.

Compatibility impact: traffic that continually injects unrelated valid frames can no longer keep a
request alive beyond its configured timeout. No public API signature changes.

Acceptance criteria:

1. Deterministic TCP and UDP wrong-serial and saturated foreign-route floods cannot extend a
   request past its configured deadline, allowing only scheduler tolerance asserted by the tests.
2. A matching response received before the same deadline succeeds after unrelated frames.
3. Send and complete response assembly share the same deadline; it is not restarted per read.
4. Timeout and cancellation isolation regression tests prove that a later request cannot consume
   bytes from the failed exchange, including a clean exchange through a newly connected session.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Formatting, Clippy, unit/integration tests, docs, Node binding, and package checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency.
- [x] Claude source review completed and findings recorded in `D:\APP\claude_review_findings_20260714.md`.
- [x] Codex resolved or dispositioned every Claude finding and reran affected checks.
- [x] Live-PLC verification is not required; deterministic TCP/UDP peers exercise the timing contract.
- [x] Documentation, migration notes, changelog, and generated API reference agree.
- [x] Final acceptance criteria verified and the item marked complete.

## PROFILE-20260714-RJ71EN71: Canonical MX-R Ethernet-unit profile

Scope: profile import tooling, Rust profile/default/capability/range metadata, user profile listing,
and downstream exhaustive matching behavior.

Target contract: `melsec:mx-r:rj71en71` is a connectable 4E/iQ-R profile based on
`melsec:mx-r`. Runtime metadata and checked-in fixtures agree field-for-field with canonical
profile release `v2.1.0`; the default importer cannot silently replace them with an older release.
Public profile and error-kind enums are non-exhaustive so later canonical additions do not create
another downstream exhaustive-match break.

Compatibility impact: the profile and `SlmpErrorKind::Timeout` are new public enum values.
`SlmpPlcProfile` and `SlmpErrorKind` are now non-exhaustive, so downstream exhaustive matches must
add a wildcard arm.

Acceptance criteria:

1. `update_slmp_profile_jsons.ps1 -FailIfChanged` uses `v2.1.0` by default and reports both
   checked-in fixtures unchanged.
2. Parsing, defaults, base profile, range profile, selected catalog identity, and model label are
   directly executable and equal the canonical profile.
3. Every canonical profile's frame, compatibility mode, subcommands, verified models, feature
   state/source, limit max/weighted max/over-end-code/source, write policy, and display label are
   compared by the parity test.
4. `docs/PROFILES.md`, API documentation, and the changelog expose the new profile and breaking
   enum-match migration.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the canonical fixture and public API surface.
- [x] Claude source review completed; findings are recorded in `D:\APP\claude_review_findings_20260714.md`.
- [x] Codex resolved or dispositioned every Claude finding and reran affected checks.
- [x] Live-PLC checks are not required for import pinning and deterministic metadata parity; no communication was performed.
- [x] Documentation, migration notes, changelog, and API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## Claude source review disposition — 2026-07-14

The authorized review scope and canonical findings are recorded in
`D:\APP\claude_review_findings_20260714.md`. Codex checked each Rust and applicable cross-library
finding against the actual source before changing it.

| Finding | Codex disposition | Resolution/evidence |
| --- | --- | --- |
| F-X1 | Accepted | Default profile import Ref is `v2.1.0`; `-FailIfChanged` reports both fixtures unchanged. |
| F-X2 | Accepted | Added the `melsec:mx-r:rj71en71` row to `docs/PROFILES.md`. |
| F-X5 | Accepted | Changelog now classifies the profile as a Library addition and records enum-match compatibility; public evolving enums are non-exhaustive. |
| R-1 | Accepted | TCP and UDP check the absolute deadline before and after every send/receive/discard boundary; a no-sleep saturated flood test covers both transports. |
| R-2 | Accepted | The saturated test uses serial-matching foreign routes, asserts timeout/Rx accounting/closed transport, then proves a fresh session succeeds. |
| R-3 | Duplicate of F-X2 | Resolved by the canonical profile table row. |
| R-4 | Duplicate of F-X1 | Resolved by the fixed `v2.1.0` importer Ref and drift check. |
| R-5 | Accepted | All request I/O timeout phases return `SlmpErrorKind::Timeout`; `is_timeout()` is public. Oversized durations remain General validation failures. |
| R-6 | Accepted | `SLMP_TIMEOUT_MS` now states that it is the absolute request-exchange deadline. |
| R-7 | Accepted | MX-R and MX-F no longer share incorrect feature evidence; MX-R and MX-R/RJ71EN71 use canonical live/spec sources. |
| R-8 | Accepted | Parity now includes sources, over-end-codes, subcommands, verified models, defaults, and labels. It exposed and corrected older iQ-R Ethernet-unit/MX-F evidence-source and iQ-F end-code drift; direct construction also caught and corrected the MX-R/RJ71EN71 address-profile normalization. |
| R-9 | Accepted | The 100 ms deadline regression lower bound is 90 ms; the 80 ms saturated case permits only 10 ms early tolerance. |
| R-10 | Accepted | Nonzero 4E reserved response bytes are a structural error; deterministic TCP and UDP tests prove transport invalidation. |
| R-11 | Accepted | `SlmpPlcProfile` is non-exhaustive and the required wildcard-match migration is in the changelog. |

- [x] Every applicable Claude finding has an explicit technical disposition.
- [x] Full post-disposition CI, package, and diff verification rerun.
- [x] No live PLC communication, commit, push, PR, or publication was performed.

Post-disposition evidence: the canonical `v2.1.0` fixture drift check, Rust formatting, Clippy with
all targets and CLI features, all unit/integration/doc tests, Node crate check, crate packaging and
package compilation, and `git diff --check` passed. The saturated TCP/UDP regression also passed
five consecutive focused runs. Final Codex diff review additionally aligned short UDP datagrams
with the malformed-response classification and added a lower-bound assertion to the segmented TCP
deadline test; the resulting 12-case response-correlation target passed five consecutive runs and
the complete CI/package gates were rerun afterward.
