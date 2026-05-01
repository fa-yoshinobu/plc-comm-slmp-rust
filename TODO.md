# TODO

This file tracks active follow-up items for the SLMP Rust library.

## 1. Validation

- **Live-check Q-series runtime ranges**
  Confirm QCPU/LCPU/QnU/QnUDV `Z`, `ZR`, and `R` runtime range behavior on real
  hardware. The expected behavior is:
  - QCPU `Z` is selected by probing `Z15` and resolves to 10 or 16 points.
  - LCPU/QnU/QnUDV `Z` is fixed at 20 points.
  - QCPU/LCPU/QnU/QnUDV `ZR` is selected by probing readable addresses and may
    resolve to 0 points.
  - `R` matches the probed `ZR` size and is capped at `R32767`.
  - `LCPU` was live-checked on 2026-05-01: `Z` remains the spec-fixed
    `Z0-Z19` range, `ZR393215` read successfully, and `ZR393216` returned
    `0x4031`. `R` matches that `ZR` size, capped at `R32767`.

## 2. Protocol Follow-Up

- **Extended Specification live coverage expansion**
  Keep Rust aligned with the .NET, Python, and Node-RED stacks after broader
  address-range, transport, and PLC-family coverage is validated. QnUDV has no
  `HG`; `U0\G10` read-only on the current QnUDV target returned `0xC070` with
  command `0x0401` subcommand `0x0080`.

- **Mixed block write root cause**
  Keep Rust behavior aligned if the root cause for one-request mixed `1406`
  write rejection is identified in the shared SLMP libraries. On the current
  QnUDV target, word-only, bit-only, and mixed `1406` block writes returned
  `0xC059`, so this appears to be block-write command support rather than a
  mixed-only rejection on that target.

## 3. Practical Limits

- ASCII mode is intentionally out of scope.
