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

The approved decisions, repository diff, and final local results were reviewed in the two user-authorized SLMP Claude batches. Canonical results and dispositions are preserved in the archived workspace instruction records.

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
- [x] Final next-release package and cross-language API comparison completed. Evidence: the `v4.0.0`
  tag equals repository HEAD, the GitHub Release and crates.io `plc-comm-slmp` `4.0.0` crate are public,
  tag-commit checks passed, and the final five-implementation source/API comparison was completed
  on 2026-07-18.

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
- [x] Claude source review completed and findings preserved in the archived workspace record `claude_review_findings_20260714.md`.
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
- [x] Claude source review completed and findings preserved in the archived workspace record `claude_review_findings_20260714.md`.
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
- [x] Claude source review completed; findings are preserved in the archived workspace record `claude_review_findings_20260714.md`.
- [x] Codex resolved or dispositioned every Claude finding and reran affected checks.
- [x] Live-PLC checks are not required for import pinning and deterministic metadata parity; no communication was performed.
- [x] Documentation, migration notes, changelog, and API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## Claude source review disposition — 2026-07-14

The authorized review scope and canonical findings are recorded in
the archived workspace record `claude_review_findings_20260714.md`. Codex checked each Rust and applicable cross-library
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

## BH-LIVE-SLMP-20260729 — Supplemental bug-hunt live verification

Scope: commit `e3ae97bba3e53a4b225c5830fbdccf485fc8559b`, profile `melsec:iq-r`, TCP
`192.168.250.100:1025`.

Target contract: the library sends profile-catalog range exceedances that fit the wire format, uses
the Q/L layout for J link-direct extended random and monitor operations, and leaves every test
device in its documented final state.

Acceptance evidence:

- [x] `D100` one-word read succeeded with value `0`.
- [x] `R32768` reached the PLC and surfaced `PlcEndCode`/`SlmpError` end code `0x4031` for command
  `0x0401`, subcommand `0x0002`; no pre-send profile-range rejection occurred.
- [x] Extended random read of `J1\W10` succeeded with value `0`.
- [x] Extended random word write changed `J1\W10` from `0` to `0x7C13`, read back `0x7C13`,
  restored `0`, and confirmed the restoration.
- [x] Extended random bit write changed `J1\B10` to ON, read ON, reset it to OFF, and confirmed OFF.
- [x] Extended monitor registration for `J1\W10` and one monitor cycle succeeded with value `0`;
  the TCP session was then closed.
- [x] Temporary one-operation live examples were removed and the repository working tree was clean.

Disposition: all supplemental live checks passed. The `R32768` result is PLC-side address evidence,
not authority to add a communication-library profile-range guard.

## RUST-LABEL-001 — Deterministic label-command wire contract

Scope: async array/random label read and write APIs.

Target contract: implement `GOAL-SLMP-LABEL-001` from the workspace decision record. Unit `0` is a
logical bit count padded per 16 bits, unit `1` is a logical byte count padded per two bytes, caller
write buffers are exact and even, and response count/metadata/length/trailing data are validated.

Compatibility impact: zero lengths, odd random-label data, unpadded array data, and malformed or
uncorrelated responses that were previously tolerated now return `SlmpError` before transport or
during response validation.

Acceptance criteria:

1. The shared bit and byte boundary vectors produce the approved padded wire lengths.
2. Invalid caller data leaves the request frame and traffic counters unchanged.
3. Response count, array metadata, positive/even length, truncation, and full consumption are checked.
4. Unknown data type IDs and random spare values remain observable.

- [x] Implementation completed in this repository.
- [x] Tests added for every local acceptance criterion.
- [x] Formatting, Clippy, complete tests, docs, and package checks passed.
- [x] Codex self-review completed and accepted findings corrected.
- [x] Live PLC verification is not required for deterministic arithmetic and injected response vectors.
- [x] Documentation, migration note, changelog, and package contents agree.
- [x] Final acceptance verified.

Verification evidence:

- Rustfmt, Clippy for all targets with CLI features and warnings denied, all unit/integration/doc
  tests, and the Node crate check passed.
- `cargo package --allow-dirty` packaged 57 files and its verification compile passed against the
  final source and test state.
- The no-auto-publish guard and `git diff --check` passed.

Self-review disposition:

- Accepted: the first arithmetic implementation triggered Clippy's manual-division-ceiling
  finding. It now uses the standard integer `div_ceil` operation.
- Accepted: invalid request-unit and truncated item-header cases were missing from the first test
  draft. Those cases were added and reverified.
- No rejected, duplicate, or deferred finding changes this contract.

## RUST-REQUEST-001 — Representable and transport-safe request payloads

Scope: async request submission plus Array/Random Label Read/Write payload construction.

Target contract: implement `GOAL-SLMP-REQUEST-001` from the workspace decision record. TCP command
payloads are limited to 65,529 bytes. UDP 3E/4E payloads are limited to 65,492/65,488 bytes so the
complete frame is at most 65,507 bytes. Rejection precedes send, counters, request-frame publication,
and 4E serial allocation. Label aggregate growth is checked before extending the payload vector.

Compatibility impact: oversized inputs now return `SlmpError` deterministically and are never
truncated or split automatically.

Acceptance criteria:

1. TCP 3E/4E and UDP 3E/4E boundary frames encode the exact request-data length and UDP datagram size.
2. Boundary-plus-one rejection preserves serial, counters, and the last request frame.
3. All four label builders accept 65,528 bytes and reject 65,530-byte aggregates, including
   abbreviation, multiple-point, and write-data cases.
4. Random Label Write rejects individual data lengths 65,536 and 65,537 before wire conversion.

- [x] Implementation completed in this repository.
- [x] Tests added for every local acceptance criterion.
- [x] Formatting, Clippy, complete tests, docs, and package checks passed.
- [x] Codex self-review completed and accepted findings corrected.
- [x] Live PLC verification is not required for deterministic field/datagram arithmetic.
- [x] Documentation, migration note, changelog, and generated API agree.
- [x] Final acceptance verified.

Verification evidence:

- `run_ci.bat` passed Rustfmt, Clippy for all targets with CLI features and warnings denied, all
  unit/integration/doc tests, and the Node crate check.
- `cargo package --allow-dirty` packaged 57 files and verified the packaged crate by compiling it.
- Canonical profile drift, the no-auto-publish guard, and `git diff --check` passed.

Self-review disposition:

- Accepted: the frame builder retains a defense-in-depth protocol guard in addition to the earlier
  transport-aware request guard, so future internal callers cannot wrap the length field.
- Rejected: request-time connection prevention is not applicable because this client connects during
  construction. Oversized rejection still precedes every request-time send, statistic, last-frame,
  and serial mutation, which is the native state-transition contract.
- No duplicate or deferred finding changes this contract.

## GOAL-SERIAL-DEFER-001 — Effective single-request capacity

Scope: every public TCP/UDP 3E/4E command path, including labels, monitor operations, raw commands,
and helpers that return decoded collections.

Target contract: each operation has one effective maximum derived from the narrowest request field,
complete transport frame/datagram, response field, decoder, output collection, and canonical profile
limit. The exact maximum is one request. Maximum-plus-one is rejected before frame publication,
4E serial allocation, statistics, transport activity, retry, or automatic split.

Compatibility impact: oversized inputs that could previously truncate, wrap, or rely on implicit
multi-request behavior are deterministic pre-transport errors.

Acceptance criteria:

1. Common raw payload tests cover TCP/UDP 3E/4E exact maximum and maximum-plus-one and assert frame,
   serial, and statistic preservation on rejection.
2. Label aggregate builders cover the protocol maximum and next representable over-limit value.
3. Command/profile count validators and exact response decoders reject over-limit or trailing output
   without a fallback request.
4. `read_named`, writes, and monitor helpers contain no hidden split, retry, or resend path.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required for deterministic size arithmetic and injected transport vectors.
- [x] Documentation, migration notes, changelog, and generated API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## GOAL-SERIAL-DEFER-002 — One absolute request deadline

Scope: ordinary TCP and UDP exchanges, correlation, parsing, and command-specific payload decoding.

Target contract: one monotonic deadline begins immediately before the first transport send and
covers send, complete receive, discarded foreign responses, correlation, protocol parse, and output
decode. Queue waiting does not consume it. Expiry retires the exact transport and never resends.

Compatibility impact: segmented or unrelated responses and decoder work can no longer extend the
configured operation duration; the affected client must be replaced after timeout.

Acceptance criteria:

1. TCP/UDP wrong-serial and foreign-route floods cannot extend the deadline.
2. Segmented TCP receive uses the original deadline for every segment.
3. Parse and payload-decode failures retire the transport; deadline checks bracket completed decode.
4. A fresh explicitly connected client can operate independently after an old generation expires.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required for deterministic deadline and injected-response behavior.
- [x] Documentation, migration notes, changelog, and generated API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## GOAL-SERIAL-DEFER-002-CONNECT — One absolute connection deadline

Scope: explicit TCP and UDP connection establishment, including IPv4 hostname
resolution, established address selection, socket work, TCP configuration, and
final client-state adoption.

Target contract: `SlmpClient::connect` creates one overflow-checked monotonic
deadline immediately before resolution or socket work. IPv4 DNS and every TCP
candidate selected by the existing sequential policy share that deadline. UDP
resolution, bind, connect, and adoption use the same deadline. TCP no-delay and
optional keepalive configuration complete before adoption and within the same
deadline. Expiry returns `SlmpErrorKind::Timeout`, disposes partial sockets, and
never adopts a late result. Candidate exhaustion before expiry remains a
`SlmpErrorKind::Transport` failure with its cause. IPv4-only behavior is unchanged.

Compatibility impact: the public API and IPv4 policy do not change. A connection
that previously consumed DNS time plus a fresh timeout for each TCP candidate can
now return timeout at the configured absolute bound.

Acceptance criteria:

1. TCP and UDP create exactly one connection deadline before resolver or socket work.
2. Delayed resolver completion after expiry returns Timeout, starts no candidate,
   and cannot be adopted later.
3. Sequential TCP candidates receive only the time remaining to the same deadline;
   a later candidate succeeds only when it completes before that deadline.
4. IPv4 literals bypass DNS, while hostname results remain filtered to IPv4 without
   alternate-family, route, profile, or discovery fallback.
5. TCP no-delay/keepalive configuration and UDP bind/connect finish before final
   client adoption and are bounded by the same deadline.
6. Absolute expiry is Timeout; pre-deadline resolver/candidate/socket failure is
   Transport; validation remains General; no connection failure is OutcomeUnknown.
7. Deterministic resolver/candidate tests, loopback TCP/UDP tests, complete Rust
   gates, documentation checks, and actual-diff self-review pass.

Self-review findings:

- `CONNECT-R-1` — Accepted and corrected. Native DNS, candidate, bind/connect,
  no-delay, and keepalive failures that complete before absolute expiry are
  explicitly mapped to Transport; only the library deadline creates Timeout.
- `CONNECT-R-2` — Accepted and corrected. TCP configuration runs in a detached
  blocking task owned only by the connect future. If its deadline wins, the late
  task result is dropped with the socket and cannot reach client state.
- `CONNECT-R-3` — Rejected with rationale. No public cancellation-token parameter
  is added: Rust caller cancellation is dropping the connect future. That path
  produces no timeout result, owns no client state, and cannot adopt detached DNS
  or configuration output, so it remains distinguishable without an API change.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required for deterministic resolver, loopback, and deadline behavior.
- [x] Documentation, migration notes, changelog, and generated API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## GOAL-FIFO-DEFER-001 — Ordinary client FIFO and close generation

Scope: all clones of one `SlmpClient`; separate client instances are separate queues.

Target contract: one shared client admits calls in FIFO order and permits one wire transaction at a
time. Rust arguments are fixed before admission by ownership/immutable borrowing. Dropping a waiting
future removes it without send or delay. `close` immediately invalidates active and queued work for
the exact connection generation without waiting behind the active operation.

Compatibility impact: close no longer waits for the active mutex holder; queued calls deterministically
fail without reaching transport.

Acceptance criteria:

1. Concurrent calls produce one non-pipelined ordered frame stream with unique 4E serials.
2. A cancelled waiter sends no frame and its successor proceeds in the next FIFO position.
3. Close returns promptly, active and queued calls get the correct error class, and no queued frame is sent.
4. A slow operation on one client does not block another client instance.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required for local admission and socket-generation behavior.
- [x] Documentation, migration notes, changelog, and generated API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## GOAL-ERROR-DEFER-001 — Pairwise operational outcomes

Scope: public `SlmpErrorKind`, state-changing command classification, and raw command behavior.

Target contract: validation, timeout, cancellation, closed, not-connected, transport, malformed
response, PLC end code, profile feature, and unknown state-changing outcome are pairwise
machine-readable. `OutcomeUnknown` carries a structured timeout/cancelled/closed/transport/malformed
reason. Once a state-changing send may have started, those failures are never returned as a safe
retry signal. A correlated PLC end code remains definitive PLC NG and leaves the transport reusable.

Compatibility impact: failures formerly grouped into `General` now use specific enum values, and
callers matching the non-exhaustive enum should handle `OutcomeUnknown` without automatic resend.

Acceptance criteria:

1. Public kinds and structured unknown reasons are pairwise distinct.
2. Active close and malformed acknowledgements after a write return structured `OutcomeUnknown`.
3. Read timeout/malformed paths retain their direct class and retire the transport.
4. PLC NG remains `PlcEndCode`, preserves the end code, and permits a later request on the complete
   correlated session.
5. `raw_command` uses `SlmpCommand::is_state_changing` and cannot bypass conservative classification.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required for deterministic error mapping and injected response vectors.
- [x] Documentation, migration notes, changelog, and generated API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## GOAL-AGGREGATE-DEFER-001 — Explicit Rust aggregate boundary

Scope: `read_named`, `poll_named`, aggregate write helpers, and `write_bit_in_word`.

Target contract: `read_named` and each polling cycle remain one random-read request or reject before
transport; no automatic read split is added. State-changing aggregates remain one request or reject.
`write_bit_in_word` is the explicit non-aggregate two-command RMW exception: both frames occupy one
FIFO turn, the operation remains non-atomic at the PLC, and its write uses conservative unknown-outcome
classification.

Compatibility impact: there is no implicit split or partial aggregate result. The RMW helper now
prevents another operation on the same client from interleaving between its read and write.

Acceptance criteria:

1. `read_named` exact profile limit succeeds as one request and limit-plus-one rejects without send.
2. Named writes reject mixed request families and bit-in-word entries before transport.
3. Source and wire tests find no hidden aggregate split, retry, or fallback path.
4. Under a queued competitor, RMW wire order is read-target, write-target, then competitor.
5. Documentation states temporal snapshot risk, PLC-level non-atomicity, and unknown write outcome.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required for deterministic batching, admission, and injected-response behavior.
- [x] Documentation, migration notes, changelog, and generated API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## 2026-08-01 Codex self-review disposition

| Finding | Disposition | Resolution |
| --- | --- | --- |
| RUST-20260801-01 | Accepted | `close` originally waited behind the active client mutex. Added a per-client close signal, non-blocking exact-generation retirement, and active/queued tests. |
| RUST-20260801-02 | Accepted | The transport was restored before response parsing and semantic decode. Restoration now follows successful parse, and semantic decode is deadline-checked; malformed decode retires the transport. |
| RUST-20260801-03 | Accepted | Four error kinds could not distinguish operational outcomes. Added pairwise public kinds and structured unknown-outcome reasons, including raw-command state classification. |
| RUST-20260801-04 | Accepted | `write_bit_in_word` released the client lock between read and write. It now holds one FIFO turn and has deterministic interleaving coverage. |
| RUST-20260801-05 | Duplicate | Existing request/label/profile boundary tests already cover exact maximum, maximum-plus-one, pre-transport state preservation, and no split; retained and included in the final gate. |
| RUST-20260801-06 | Rejected | Adding automatic `read_named` splitting would violate the approved Rust single-request contract and temporal-snapshot guarantee; no implementation change made. |
| RUST-20260801-07 | Deferred | Live PLC confirmation is unnecessary for local FIFO, arithmetic, timeout, injected malformed response, and classification contracts; no live communication was performed. |
| RUST-20260801-08 | Accepted | Package verification exposed that `tokio::select!` was available only through dev/CLI feature unification. Enabled Tokio `macros` for the distributed library dependency and reran package/consumer gates. |
| RUST-20260801-09 | Accepted | The persistent package gate used only `cargo package --list`, so the recorded generated-crate and isolated-consumer evidence was not reproducible from the gate. It now generates and extracts the `.crate`, checks its exact consumer boundary, builds packaged targets and rustdoc, and compiles a separate path consumer from the extracted artifact. |
| RUST-20260801-10 | Accepted | Worktree source validation copied paths returned by `git ls-files`; a tracked deletion therefore failed at `Copy-Item` instead of producing the intended archive. The gate now creates a synthetic tree with a temporary Git index and `git add -A`, covering modifications, non-ignored untracked files, and deletions without changing the real index. |
| RUST-20260801-11 | Accepted | The first temporary index was placed under the repository `build/` tree, allowing `git add -A` to stage the index and lock files themselves. The index now lives in the validated workspace parent, outside the repository, and both index paths are removed in `finally`. |
| RUST-20260801-12 | Accepted | The source archive was extracted below the original repository's Cargo workspace, so its nested generated-crate check could be associated with the outer checkout. The complete source-archive work root now lives in the validated workspace parent outside the repository; packaged-crate and consumer checks cannot resolve through the original workspace. |
| RUST-20260801-13 | Accepted | The final Clippy gate found a redundant closure around `TcpStream::connect` in the new absolute-deadline connector. Replaced it with the function item and reran Clippy plus the complete Rust, package, source-archive, MSRV, and representative Windows gates. |

Final verification evidence:

- Rust 1.85 compiled the published crate with all targets and all features.
- Rustfmt, Clippy with warnings denied, workspace/all-target/all-feature tests, examples, the Node
  binding, and rustdoc with warnings denied passed.
- The persistent package gate generated a 39-file `.crate` with 13 declared
  Cargo examples, excluded repository tests and maintainer tooling, verified the
  extracted package targets and rustdoc, and compiled a separate consumer project
  against only the extracted package.
- The worktree source-archive gate created a complete 75-file synthetic Git tree,
  retained tests and maintainer inputs, extracted outside the original Cargo
  workspace, and ran the complete local gate from that extracted tree. Its nested
  no-checkout package gate produced the expected 38-file crate (the direct Git
  package's VCS metadata is unavailable in a source-archive extraction), built all
  13 examples and rustdoc, and compiled the isolated consumer.
- Canonical profile drift, no-auto-publish policy, and `git diff --check` passed.
- No live PLC communication, commit, push, release, or public-registry publication was performed.

## GOAL-CROSS-OS-CI-001 — Required Windows representative contract smoke

Implementation scope: the repository CI workflow and existing deterministic
Tokio loopback tests. Runtime code, public API, packaging, release workflows,
the Ubuntu MSRV job, and the Ubuntu full workspace gate are unchanged.

Target contract: the primary Ubuntu gates remain authoritative. One additional
non-optional Windows stable-Rust job runs only representative localhost
contracts for segmented TCP receive under one deadline, post-send cancellation
retirement, refused connection classification, close retirement of active/queued
work, UDP timeout retirement, and rejection of delayed data before a new session.
The job has a ten-minute bound and does not run the complete test, feature,
Node, documentation, or package matrices.

Compatibility impact: none; this adds CI evidence only.

Machine-verifiable acceptance criteria:

1. `.github/workflows/ci.yml` contains exactly one `windows-latest` contract-
   smoke job in addition to the unchanged Ubuntu MSRV and full gates.
2. The Windows job is required by workflow semantics: it has no conditional,
   failure suppression, or `continue-on-error` path.
3. Its six explicit test filters cover connection failure, fragmented receive/
   deadline behavior, cancellation, active/queued close retirement, timeout
   generation retirement, reconnect, and delayed-data rejection.
4. The Windows job installs only the stable toolchain, runs the bounded
   integration-test subset, and does not package, publish, or contact a PLC.

- [x] Implementation completed in this repository.
- [x] Existing deterministic tests explicitly selected for every acceptance criterion.
- [x] The exact six-command representative selector passed locally on Windows with Rust 1.95 stable.
- [x] The new Windows CI job passed on GitHub for the final source state.
- [x] Codex self-review completed after the local representative and complete verification runs.
- [x] Live PLC checks are not required; all selected behavior uses localhost loopback.
- [x] Maintainer CI documentation agrees with the workflow; no user migration note or changelog entry is required.
- [x] Final acceptance criteria verified and the item marked complete.

Verification evidence: the exact selector passed on the local Windows host.
The complete local stable gate, exact Rust 1.85 MSRV check, package consumer,
current-worktree source archive, canonical profile, no-auto-publish, and
`git diff --check` also passed on the reviewed source state. The GitHub-hosted
Windows representative job passed for reviewed functional source commit
`94558b4b8475c15048daca1b971f1bd7dbb2d0dd` in
[PR #50](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/pull/50), with the
Ubuntu MSRV, full Rust, and sample checks also green. The follow-up change
records that external evidence only and does not change runtime or workflow
behavior.

Self-review disposition:

- Accepted and corrected: the initial subset omitted connection failure and
  delayed-data isolation across sessions. Existing deterministic cases are now
  selected without changing the Ubuntu MSRV or full workspace gate.
- Rejected: duplicating the four integration cases into a Windows-only test
  target would create a second contract source without new coverage. Exact
  filters keep the repository-owned tests authoritative.
- Duplicate findings: none. Deferred findings: none.

## GOAL-SLMP-REVIEW-R1-001 — Single-request named reads

Implementation scope: Rust named-read planning and polling, long-timer route
selection, tests, examples, rustdoc, user documentation, and changelog.

Target contract: `read_named` and every `poll_named` cycle emit exactly one
canonical Random Read or reject the complete plan before transport. `LTN`,
`LSTN`, `LTS`, `LTC`, `LSTS`, and `LSTC` entries that require the Direct
long-timer route use `read_typed` or an explicit long-timer helper and never an
implicit named-read fallback.

Compatibility impact: contradictory examples and unreachable fallback code are
removed. A long-timer Direct entry in a named plan deterministically fails with
zero requests; supported typed and explicit long-timer APIs remain available.

Machine-verifiable acceptance criteria:

1. Representative current and state entries, including a mixed `D100:U` plan,
   fail before send in both `read_named` and `poll_named`.
2. No named execution branch can issue a second Direct Read.
3. Quick Start, examples, guides, rustdoc, and API reference contain no such
   named-read usage and identify the explicit migration.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, tests, examples, documentation, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required; planning and zero-send behavior are deterministic local properties.
- [x] Documentation, migration notes, changelog, and API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## GOAL-SLMP-REVIEW-R2-001 — Canonical route-validation capabilities

Implementation scope: Rust route-validation planning, canonical profile lookup,
generic Random cases, LZ-specific cases, tests, maintainer documentation, and
changelog.

Target contract: Block, Random, and LZ admission comes from the canonical
profile and address-family rules. Q/L E71 unit profiles run Block and Random but
skip LZ; base Q/L profiles run Random without Block; generic Random cases never
parse or use an inapplicable LZ sample.

Compatibility impact: diagnostic false failures and false skips are corrected.
Runtime SLMP APIs, profile capabilities, and wire behavior do not change.

Machine-verifiable acceptance criteria:

1. Every selectable profile's effective Block, Random, and LZ decisions match
   canonical profile state and address-family admission.
2. All four Q/L E71 profiles run generic Block and Random while skipping LZ.
3. Base Q/L profiles run Random, skip Block, and skip LZ without parsing LZ.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, tests, examples, documentation, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required; canonical planning is a deterministic local property.
- [x] Documentation, migration notes, changelog, and API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## GOAL-SLMP-REVIEW-D1-001 — Definitive result precedence

Implementation scope: Rust TCP/UDP response completion, concurrent close state
transitions, success and PLC-end-code behavior, tests, user and maintainer
documentation, and changelog.

Target contract: after complete correlation, protocol validation, and
command-specific result or PLC-end-code decode, that definitive result wins over
a concurrent close or a local deadline observed only after decode. The local
condition still promptly retires the transport. Before definitive completion,
close and timeout classification take precedence over a malformed payload or
command-decode failure; incomplete reads return `Closed`/`Timeout`, possibly
transmitted incomplete writes return the corresponding `OutcomeUnknown`, and
queued retired-generation calls send nothing.

Compatibility impact: a narrow close/deadline race now returns the
already-established success or PLC error instead of replacing it with a local
lifecycle result. Conversely, an incomplete malformed/decode failure now keeps
an already-observed close or timeout classification. Reconnect behavior is
unchanged.

Machine-verifiable acceptance criteria:

1. Deterministic TCP and UDP races preserve decoded success and PLC end code for
   representative reads and writes across concurrent close and post-decode
   deadline observation.
2. Close or timeout before definitive completion retains
   closed/timeout/outcome-unknown behavior, including malformed payload and
   command-decode failures.
3. The preserved result does not restore a retired transport or admit queued
   work from the retired generation.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, tests, examples, documentation, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required; lifecycle races use deterministic local transports.
- [x] Documentation, migration notes, changelog, and API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

Final audit disposition (2026-08-01): accepted and corrected a
deadline-boundary coverage gap. A deterministic unit test now proves that a
command-decode error with an already-expired local deadline returns `Timeout`,
retires the transport, and clears the pending decode state. After adding that
test, the current-worktree source-archive gate reran the complete local gate,
including all 71 library unit tests, integration and documentation tests, the
Node binding, the 38-file generated crate, and its isolated consumer. Canonical
profile drift, no-auto-publish policy, and `git diff --check` passed on the
final source state.

## GOAL-SLMP-REVIEW-N1-001 — Exact semantic device units

Implementation scope: Rust Direct, Extended Device, Random bit-write, Block
bit-entry, typed/named helpers, canonical device metadata, tests, user and
maintainer documentation, and changelog.

Target contract: every semantic bit-unit or bit-entry API accepts only a bit
device. Typed/named `BIT` accepts only bit devices and numeric/string dtypes
accept only word devices. Explicit low-level word APIs retain packed 16-bit
access to bit devices. A word-device bit uses `.n` or `write_bit_in_word`; no
invalid semantic call is translated into masking, RMW, another route, or retry.

Compatibility impact: previously emitted invalid word-device bit-unit calls and
numeric typed access to bit devices now fail locally. Explicit packed-word and
bit-in-word operations remain supported.

Machine-verifiable acceptance criteria:

1. Direct, extended, random-write, block-bit, typed, and named wrong-unit calls
   reject before request counters or transport.
2. Qualified `G/HG` bit access rejects while valid bit-device routes remain.
3. An explicit low-level word read of a bit device still emits word-unit access.
4. Tests and documentation distinguish bit-device bits, packed bit-device
   words, and word-device bit-in-word access.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, tests, examples, documentation, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required; device classification and zero-send behavior are deterministic local properties.
- [x] Documentation, migration notes, changelog, and API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

## 2026-08-01 review-item self-review disposition

| Finding | Disposition | Resolution |
| --- | --- | --- |
| RUST-REVIEW-20260801-01 | Accepted | Removed two remaining runtime error messages that still directed Direct long-family callers to `read_named`; all such guidance now names `read_typed` or an explicit long helper. |
| RUST-REVIEW-20260801-02 | Accepted | Initial unit coverage proved only the final command-decode helper. Added deterministic TCP and UDP barriers for decoded reads, acknowledged writes, PLC end codes, and close-before-decode behavior. |
| RUST-REVIEW-20260801-03 | Accepted | Initial unit validation covered bit blocks only. Added the approved inverse word-block/device check and zero-send tests for both wrong directions. |
| RUST-REVIEW-20260801-04 | Accepted | Replaced two independent bit/word device lists with one exhaustive `SlmpDeviceUnit` classifier used by both public predicates. |
| RUST-REVIEW-20260801-05 | Accepted | The first source-archive command used its default committed `HEAD`, so it was rejected as evidence for this dirty worktree. The final gate explicitly used `-Worktree` and passed on the synthetic tree containing all current changes. |
| RUST-REVIEW-20260801-06 | Rejected | Restoring a post-decode close override would contradict definitive-result precedence and could encourage retry after an acknowledged write; the transport is retired while the established result is preserved. |
| RUST-REVIEW-20260801-07 | Duplicate | Existing FIFO/close tests already prove queued retired-generation work sends nothing and incomplete transmitted writes remain `OutcomeUnknown`/`Closed`; retained them in the final full gate. |
| RUST-REVIEW-20260801-08 | Deferred | Live PLC verification is not required for deterministic planning, canonical local capability selection, unit classification, or lifecycle barriers. No live communication was performed. |
| RUST-REVIEW-20260801-09 | Accepted | Non-definitive malformed payload and command-decode failures could retain malformed classification after concurrent close. TCP and UDP now apply close/timeout classification before returning an unconfirmed decode failure, with deterministic read and unexpected-write-payload races. |
| RUST-REVIEW-20260801-10 | Accepted | A deadline checked after successful command decoding or framed PLC-error construction could overwrite that definitive result. Deadline checks now bracket the non-definitive decode phase while an established success or PLC end code remains definitive and retires the expired transport; unit and TCP/UDP deadline barriers cover both result forms. |

Final verification evidence for the reviewed worktree:

- `cargo fmt --all -- --check`, Clippy with all targets/features and warnings denied,
  all-target/all-feature tests, rustdoc with warnings denied, and the no-auto-publish
  guard passed.
- The generated crate check produced a 39-file crate, compiled all 13 packaged
  examples and rustdoc, and compiled an isolated consumer from only the extracted
  package.
- `scripts/check_source_archive.ps1 -Worktree` created the synthetic current-tree
  archive, reran the complete local gate including the Node binding, generated the
  38-file no-checkout crate expected inside a Git archive, and passed its isolated
  consumer check.
- `git diff --check` passed. No live PLC communication, commit, push, release, or
  registry publication was performed.

## GOAL-SLMP-SPAN-20260801 — Complete wire-address span admission

Stable identifier: `SLMP-SPAN-20260801-RUST`.

Implementation scope: Rust contiguous Direct word/bit/DWord/Float32 operations,
Random entries, Monitor registration, Block routes, applicable Extended Device
routes, long-timer Direct status blocks, validation ordering, tests, user/API
documentation, migration notes, and changelog.

Target contract: before request-counter mutation or transport, every applicable
operation proves that its complete consumed device span fits the selected address
field. Q/L-compatible and link-direct wire layouts use 24 bits and iQ-R layouts
use 32 bits. Word devices consume one number per word, packed word access to bit
devices consumes 16 numbers per word, ordinary DWord/Float32 values consume two
word-device numbers, packed bit-device DWords consume 32 numbers, bit blocks
consume 16 bit-device numbers per block point, and Direct LTN/LSTN status blocks
consume one logical device per four wire words. Random/Monitor long scalar
entries retain their existing one-device semantic width. This is wire
representability only; canonical profile usable ranges are not pre-send guards.

Compatibility impact: requests that previously wrapped or truncated their final
device number, or reached transport with an unrepresentable span, now fail
locally with `SlmpError`. Exact-boundary requests remain admitted. No
compatibility alias or silent split is retained.

Machine-verifiable acceptance criteria:

1. Q/L-compatible 24-bit and iQ-R 32-bit Direct word/bit read and write accept
   one point at the maximum and reject a two-point span from that maximum with
   zero request-counter or transport effects.
2. Ordinary DWord/Float32 read and write accept one value at maximum-minus-one,
   reject two values there, and reject one value at the maximum.
3. Packed bit-device word/DWord and bit-block routes use 16/32-device expansion;
   Direct long-timer status blocks use one logical device per four wire words.
4. Random and Monitor DWord entries and word/bit Block reads and writes apply the
   same route-specific span rules, including Extended Device layouts where that
   contiguous-width contract applies.
5. Validation uses checked wide arithmetic, runs before request-counter mutation
   or transport, and does not consult the profile device-range catalog.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC checks are not required; wire-field arithmetic and zero-send admission are deterministic local properties.
- [x] Documentation, migration notes, changelog, and API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

Self-review disposition (2026-08-01):

- Accepted: the initial shared width helper treated every `LTN`/`LSTN` use as a
  four-word Direct status block, which would have misclassified Random/Monitor
  scalar entries. Direct long-status semantics and scalar DWord-entry semantics
  are now explicit and independently tested.
- Accepted: existing Random/Extended Random and Block write-overlap checks still
  used fixed logical widths and 32-bit end arithmetic. They now use the same
  packed bit-device 16/32-width model as admission, with overlap regression tests
  and wide end arithmetic.
- Accepted: ordinary Random-write overlap still treated every DWord entry as two
  devices. Native `LTN`/`LSTN`/`LCN`/`LZ` DWord entries now use their one-device
  width, with adjacent-entry transport coverage.
- Accepted: ordinary and qualified Random writes and Block writes originally ran
  overlap checks before complete selected-field span admission. They now resolve
  every applicable route, validate every span, then check overlap and construct
  payloads. Combined invalid-span-plus-overlap tests pin the wire-field error and
  zero-request result for all three write families.
- Accepted: selected-field comparison initially reused a `u32` span-end helper,
  so an iQ-R overflow could be classified as arithmetic overflow before the
  selected wire maximum was compared. Admission now computes in `u64` and emits
  the stable wire-field error.
- Accepted: long-timer, long-retentive-timer, and LZ single-request helpers still
  ran the older `u32` span-end check before selected-wire validation. Their
  admission now uses the shared wide selected-field validator; decode-time
  checked arithmetic remains as defensive response handling.
- Accepted: `write_bit_in_word` preflight originally happened only after its read
  and did not reject a bit-device target. It now requires a word device and runs
  all write-policy and span admission before the read; S/M regressions prove an
  invalid target sends neither half of the sequence.
- Accepted: initial coverage did not exercise packed bit-device DWord/Float32,
  link-direct 24-bit Extended Device, or long-timer exact-boundary behavior.
  Focused zero-send and exact-boundary tests now cover each applicable route.
- Accepted: the initial Rust wording did not state that J-qualified link-direct
  stays 24-bit on an iQ-R client. The user and API guides now state that selected
  layout explicitly, matching the other implementations.
- Rejected: enforcing the configured PLC device-range catalog here would turn a
  wire-representability invariant into profile policy and contradict the
  approved contract; no such guard was added.
- Rejected: moving every Rust admission check ahead of the shared-client mutex is
  not required by the approved contract, which requires completion before
  request-counter mutation or transport. Validation remains inside the serialized
  turn where appropriate.
- Deferred: none. Live PLC communication is not required for deterministic
  arithmetic and pre-transport state assertions.

Verification evidence: after every accepted self-review correction,
`run_ci.bat` passed formatting, Clippy with warnings denied, rustdoc with warnings
denied, all unit/integration/doc tests, the Node crate check, and the generated
39-file crate plus all 13 packaged examples and an isolated consumer. The route
guard suite passed all 78 tests, and the package contract also passed on the
declared Rust 1.85 minimum toolchain. `git diff --check` passed; no live PLC
communication, registry operation, commit, push, or publication was performed.

## GOAL-SLMP-ERROR-INFO-CORRELATION-001 — Correlate PLC error information with the request

Stable identifier: `SLMP-ERROR-INFO-CORRELATION-001`.

Implementation scope: Rust 3E/4E TCP and UDP response correlation, structured
PLC end-code parsing, malformed-response classification, state-changing outcome
classification, transport invalidation, tests, maintainer and user
documentation, migration notes, changelog, and generated API reference.

Target contract: when a non-zero end-code response contains at least the
9-byte SLMP error-information prefix, its network, station, module I/O,
multidrop, command, and subcommand must exactly match the originating request.
A mismatch is a malformed response and invalidates the active transport. For a
state-changing request that may have been transmitted, the public result is
`OutcomeUnknown(MalformedResponse)`; it must never be reported as a definitive
PLC error. For a non-state-changing request, the public result is a malformed
response. Bytes following the required 9-byte prefix remain permitted, do not
participate in the required equality check, and retain their existing handling.

Compatibility impact: responses whose outer envelope matches but whose error
information identifies a different route, command, or subcommand are no longer
accepted as the current request's PLC error. The connection cannot be reused
after that mismatch. Correctly correlated PLC errors, including those with
additional error data, retain their existing structured result.

Deferred specification boundary: the treatment of a non-zero end-code response
whose error information is absent or shorter than 9 bytes is not decided by
this item. Implementation and tests for this goal must not infer or silently
change that behavior.

Machine-verifiable acceptance criteria for the Rust implementation:

1. For both 3E and 4E frames, a non-zero end-code response with a 9-byte error
   prefix whose route, command, and subcommand match the request retains the
   existing structured PLC-error result.
2. For both TCP and UDP, independent mismatches in network, station, module I/O,
   multidrop, command, and subcommand are rejected as malformed and leave the
   client transport unusable until a new connection is established.
3. For representative transmitted state-changing commands on TCP and UDP, each
   mismatch returns `OutcomeUnknown(MalformedResponse)` and never returns a
   definitive PLC-end-code error.
4. For representative read-only commands on TCP and UDP, each mismatch returns
   a malformed-response error and the following request cannot reuse the
   invalidated transport.
5. A correctly correlated error prefix followed by additional bytes retains the
   existing PLC-error result and does not fail merely because the error payload
   exceeds 9 bytes.
6. Tests distinguish outer-envelope/serial correlation from error-information
   correlation and exercise both 3E and 4E without relying on live PLC hardware.
7. Existing behavior for absent or shorter-than-9-byte error information remains
   outside this item's assertions until its specification is decided separately.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every acceptance criterion.
- [x] Relevant static checks, unit tests, integration tests, examples, and package/build checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Required live-PLC checks passed, or each unavailable check has an explicit release disposition.
- [x] Documentation, migration notes, changelog, and generated API reference agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.

### Verification evidence and self-review disposition (2026-08-02)

- `run_ci.bat`: PASS. Formatting, clippy with `-D warnings`, rustdoc with
  `-D warnings`, the complete Cargo test suite, crate-file validation, all 13
  examples, and an isolated generated-crate consumer completed successfully.
- Deterministic loopback fixtures cover TCP and UDP, 3E and 4E, all six
  independently mismatched identity fields, read-only and state-changing
  classifications, transport retirement, and matching prefixes followed by
  zero, one, or three additional bytes.
- Codex self-review inspected the actual diff, public error fields, parse order,
  request target and command identity, outer-envelope versus structured-error
  correlation, short-error boundary, outcome-unknown mapping, transport
  invalidation, tests, documentation, packaging, and .NET consistency.
  Accepted finding: the first implementation accepted trailing bytes but
  discarded them while .NET retained them; `SlmpErrorInfo.extra` and exact
  zero/one/multiple-byte retention tests corrected the inconsistency. Rejected
  findings: none. Duplicate findings: none. Deferred findings: none.
- Live PLC verification is not required for this item: parser correlation and
  lifecycle behavior are completely observable with deterministic transport
  fixtures, and no PLC/profile compatibility claim changed. No live PLC
  communication was performed.

## PERF-010E1 — Single general-request payload validation

Implementation scope: `ClientInner::request`, request payload preflight, the
private request-frame builder, and deterministic unit tests. PERF-010E2 DNS and
TCP socket-option blocking tasks are outside this item and remain unchanged.

Target contract: after the closed-state check and before frame, serial,
statistics, or transport mutation, a general request fully validates its
payload exactly once. Successful preflight returns a private prepared value
containing the validated payload borrow and calculated request data length.
Only that value can call the mutation-only private frame builder. Response
framing, correlation, deadlines, close handling, and error classification are
unchanged.

Compatibility impact: none. Public API, errors, validation order and text,
maximum lengths, 3E/4E wire bytes, serial progression, FIFO, request count, and
transport state retain their existing contract; only duplicate CPU work is
removed.

Machine-verifiable acceptance criteria:

1. A normal general request increments the test-only full-validation counter once.
2. The prepared builder does not increment that counter or recalculate length.
3. Oversized and malformed monitor-register payloads preserve the previous
   request frame, 4E serial, statistics, and transport before send.
4. Existing TCP/UDP 3E/4E exact-boundary frame tests remain byte-identical.
5. Response validation and PERF-010E2 connection-task code have no diff.

- [x] Implementation completed in this repository.
- [x] Tests added or updated for every machine-verifiable acceptance criterion.
- [x] Relevant MSRV, format, Clippy, rustdoc, full unit/integration, N-API, example, package, and current-worktree source-archive checks passed.
- [x] Codex self-review completed against the approved contract and cross-language consistency requirements.
- [x] Live PLC verification is not required because validation count, mutation boundaries, frames, and state preservation are deterministic local behavior.
- [x] Maintainer documentation, changelog, and API documentation agree with the implementation.
- [x] Final acceptance criteria verified and the item marked complete.
