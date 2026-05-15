# TODO

This file tracks active follow-up items for the SLMP Rust library.

## 1. Validation

- [x] **iQ-L sample read/write validation**: On 2026-05-03, the live iQ-L `L16HCPU`
  at `192.168.250.100:1025` passed `device_range_sample_compare` with 352 sampled
  points across every parser-supported catalog device. Each device used up to 10
  addresses including the start, end, middle, and distributed points, with
  read/write/read-back/restore verification. The same run also passed 129
  bit-block checks comparing contiguous `read_bits` results with `DeviceReadBlock`
  packed bit-word values. `S` remains unsupported by design. See
  `docs/IQL_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md`. The same shared
  checker was also invoked through the Android Rust bridge and iOS C ABI on
  2026-05-03 with the same pass counts. The same validation also passed over
  UDP `192.168.250.100:1027` through Rust, Android Rust bridge, and iOS C ABI.

- [x] **iQ-L route validation against .NET-style coverage**: On 2026-05-03, the live
  iQ-L `L16HCPU` at `192.168.250.100:1025` passed the new
  `route_validation_compare` harness (`passed=18`, `failed=0`, `warned=1`,
  `skipped=0`). The harness covers block, random, typed, `LZ` guard behavior,
  and range-error routes, following the more advanced `plc-comm-slmp-dotnet`
  coverage. The same shared checker also passed through the Android Rust bridge
  and iOS C ABI over both TCP `1025` and UDP `1027`. The single warning is live PLC behavior: out-of-range
  `ZR786432` returns `0x4031` on direct/typed/block/random-write routes, but
  random read succeeds with `0xFFFF`; .NET showed the same result.

- [x] **iQ-R sample and route validation**: On 2026-05-03, the live iQ-R `R08CPU`
  at `192.168.250.100` passed the same all-device sample and route validation
  over TCP `1025` and UDP `1027`. Final all-device result:
  `passed=352`, `read_failed=0`, `write_failed=0`, `restore_failed=0`,
  `unsupported=1`, `bit_blocks_passed=135`, `bit_blocks_failed=0`. Route result:
  `passed=18`, `failed=0`, `warned=1`, `skipped=0`. The same shared checkers
  passed through the Android Rust bridge and iOS C ABI. The first full TCP run
  while the X unit was installed failed only on `X0` / bit-block writes starting
  at `X0` and `X1`; after the X unit was removed, full validation passed. See
  `docs/IQR_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md`.

- [x] **iQ-F sample and route validation**: On 2026-05-03, the live iQ-F
  `FX5UC-32MT/D` at `192.168.250.100` passed all-device sample and route
  validation over TCP `1025` and UDP `1027` using `Frame3E` + `Legacy`.
  Final all-device result: `passed=262`, `read_failed=0`, `write_failed=0`,
  `restore_failed=0`, `skipped=10`, `unsupported=0`, `bit_blocks_passed=110`,
  `bit_blocks_failed=0`. Route result: `passed=17`, `failed=0`, `warned=0`,
  `skipped=2` (`ZR`, `RD`). Do not select `Frame4E` for iQ-F; it is unsupported
  by the manual. See `docs/IQF_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md`.

- [x] **LCPU route and all-device candidate validation**: On 2026-05-03, the live
  LCPU at `192.168.250.100` was checked over TCP `1025` and UDP `1027` using
  `SLMP_PLC_FAMILY=lcpu` (`Frame3E` + `Legacy`). Route validation passed on the
  supported routes with `passed=9`, `failed=0`, `warned=0`, `skipped=10`.
  `read_type_name`, `Frame4E`, block routes, random routes, `LZ`, and `RD` are
  not used for LCPU validation. The all-device candidate sample intentionally
  reports NG instead of hiding it: TCP/UDP both returned `passed=245`,
  `read_failed=0`, `write_failed=0`, `readback_failed=5`, `restore_failed=0`,
  `skipped=11`, `unsupported=1`. Readback NG points were `SB6FF`, `SB7FF`,
  `SW6FF`, `SW7FF`, and `SD1279`. See
  `docs/LCPU_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md`.

- [x] **Investigate LCPU readback NG points**: Human review confirmed that
  `SB6FF`, `SB7FF`, `SW6FF`, `SW7FF`, and `SD1279` are expected
  PLC/system-device behavior for this target, not unresolved Rust library bugs.
  The harness output still records the raw `readback_failed=5` result.

- [x] **QnUDV route and all-device candidate validation**: QnUDV has been
  live-checked with the maintained validation harnesses. The earlier sample
  validation intentionally reported the expected system-device readback NG
  instead of hiding it. The current QnUDV record is the 2026-05-15
  `Q06UDVCPU` run at `192.168.250.100:1025`, which passed cross-stack direct
  read/write/restore, random read/write, word-only block read/write, bit-block
  NG behavior, mixed-block NG behavior, and read-only `U0\G10` Extended
  Specification checks across Python, Node-RED, .NET, Rust, and C++ Minimal.
  See `docs/QNUDV_RUNTIME_RANGE_VALIDATION_2026-05-15.md` and
  `../plc-comm-slmp-cross-verify/QNUDV_CROSS_STACK_VALIDATION_2026-05-15.md`.

- [x] **Investigate QnUDV readback NG point**: Human review confirmed that
  `SD1279` is expected PLC/system-device behavior for this target, not an
  unresolved Rust library bug. The harness output still records the raw
  `readback_failed=1` result.

- [x] **iQ-L max-count and chunking validation**: On 2026-05-03, the live iQ-L
  `L16HCPU` at `192.168.250.100` passed the `iql_live_stress` direct/chunk
  checks over TCP `1025` and UDP `1027`: 960-word direct, 480-dword direct,
  1024-word chunked, 512-dword chunked, and 960-bit direct write/read-back/
  restore. Typed helper routes for `D:U`, `D:D`, `D:F`, `M:BIT`, `LZ:D`, and
  `RD:U` also passed. Random write/read-back/restore passed at the reference
  size `SLMP_RANDOM_DEVICE_POINTS=48`; this is a target observation only, not a
  fixed library-wide upper limit. See
  `docs/IQL_LIVE_STRESS_VALIDATION_2026-05-03.md`.

- [x] **iQ-L basic live read/write stress validation**: Added
  `examples/iql_live_stress.rs` and ran it against the live iQ-L over TCP
  `1025` and UDP `1027`. The harness records pass/fail timing, performs
  read/write/read-back/restore checks, and prints `OBSERVED-NG` for candidate
  routes instead of hiding them. Current observed candidates: random read 49
  word points succeeded, random read 96 word points succeeded in a separate
  probe, random write 96 word points returned `0xC054`, and two word-block
  write returned `0xC05B`.

- [x] **iQ-L timeout and reconnect validation**: UDP wrong-port/reconnect passed
  in `iql_live_stress`. TCP reconnect initially failed inside the full stress
  process because `SlmpClient::close()` only cleared frame buffers and did not
  shut down the TCP stream, leaving the previous connection open. After fixing
  `close()` to shut down TCP and dropping the main stress client before reconnect
  probes, TCP wrong-port/reconnect also passed over `192.168.250.100:1025`.

- [x] **Live-check Q-series runtime ranges for QCPU/LCPU/QnUDV**: Confirmed
  QCPU/LCPU/QnUDV `Z`, `ZR`, and `R` runtime range behavior on real hardware.
  The expected behavior is:
  - QCPU `Z` is selected by probing `Z15` and resolves to 10 or 16 points.
  - LCPU/QnU/QnUDV `Z` is fixed at 20 points.
  - QCPU/LCPU/QnU/QnUDV `ZR` is selected by probing readable addresses and may
    resolve to 0 points.
  - `R` matches the checked `ZR` size and is capped at `R32767`.
  - `QCPU` was live-checked on 2026-05-15 against a Mitsubishi `Q12HCPU` at
    `192.168.250.100:1025` over TCP with 3E/legacy access: the catalog command
    reported `Z0-Z15`, `ZR0-ZR131071`, and `R0-R32767`. The current Q12HCPU
    record also includes cross-stack direct read/write/restore, random
    read/write, word-only block read/write, bit-block NG behavior, mixed-block
    NG behavior, and read-only `U0\G10` Extended Specification checks. See
    `docs/QCPU_RUNTIME_RANGE_VALIDATION_2026-05-15.md` and
    `../plc-comm-slmp-cross-verify/QCPU_CROSS_STACK_VALIDATION_2026-05-15.md`.
  - `LCPU` was live-checked on 2026-05-01: `Z` remains the spec-fixed
    `Z0-Z19` range, `ZR393215` read successfully, and `ZR393216` returned
    `0x4031`. `R` matches that `ZR` size, capped at `R32767`.
  - `QnUDV` was live-checked on 2026-05-15 against Mitsubishi `Q06UDVCPU` at
    `192.168.250.100:1025` over TCP with 3E/legacy access: `Z19` read
    successfully, `Z20` returned `0x4031`, `ZR393215` read successfully,
    `ZR393216` returned `0x4031`, and `R32767` read successfully with `R32768`
    blocked by the client guard or returned as out of range.

- [x] **Live-check QnU runtime ranges**: On 2026-05-15, the live Mitsubishi
  `Q26UDEHCPU` at `192.168.250.100:1025` was checked over TCP with 3E/legacy
  access. The catalog reported `Z0-Z19`, `ZR0-ZR655359`, and `R0-R32767`.
  Boundary reads confirmed `Z19` OK / `Z20` -> `0x4031`, `ZR655359` OK /
  `ZR655360` -> `0x4031`, and `R32767` OK with `R32768` blocked by the client
  guard or returned as out of range. The same target also passed cross-stack
  direct read/write/restore, random read/write, word-only block read/write,
  bit-block NG behavior, mixed-block NG behavior, and read-only `U0\G10`
  Extended Specification checks across Python, Node-RED, .NET, Rust, and C++
  Minimal. See
  `docs/QNU_RUNTIME_RANGE_VALIDATION_2026-05-15.md` and
  `../plc-comm-slmp-cross-verify/QNU_CROSS_STACK_VALIDATION_2026-05-15.md`.

## 2. Protocol Follow-Up

- [ ] **Extended Specification live coverage expansion**: Run the expanded
  `extended_device_coverage` sweep across the remaining PLC-family and transport
  matrix. Keep OK/NG rows visible in the generated report. QnUDV has no `HG`;
  QnUDV `U0\G10` read-only was live-checked on 2026-05-15 against `Q06UDVCPU`
  and returned `[0]` across Python, Node-RED, .NET, Rust, and C++ Minimal. QCPU
  `U0\G10` read-only was live-checked on
  2026-05-15 against `Q12HCPU` and returned `[0]` across Python, Node-RED,
  .NET, Rust, and C++ Minimal. QnU `U0\G10` read-only was live-checked on
  2026-05-15 against `Q26UDEHCPU` and returned `[0]` across the same five
  stacks.
  On the current iQ-L target, `U3E0\G...` is the valid Extended Specification
  live-coverage path; `HG` and `J` paths are not part of that PLC's executable
  coverage set.
