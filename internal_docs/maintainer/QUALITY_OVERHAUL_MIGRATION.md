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

- Scope: CPU-buffer and long timer/retentive timer helpers.
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
- [ ] Final acceptance criteria verified and this Rust batch marked complete.

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

The approved decisions, this acceptance record, the repository diff, and final local check results form the review package. Claude execution remains pending explicit user authorization for the named diff scope.
