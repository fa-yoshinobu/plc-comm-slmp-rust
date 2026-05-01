# TODO

This file tracks active follow-up items for the SLMP Rust library.

## 1. Validation

- **Run Rust test suite in an environment with Cargo**
  The Q-series runtime range changes and iQ-F X/Y octal handling are present in
  source, but `cargo` was not available in the current Windows environment. Run
  the device-range/address tests and full suite once the Rust toolchain is
  installed.

- **Live-check Q-series runtime ranges**
  Confirm QCPU/LCPU/QnU/QnUDV `Z`, `ZR`, and `R` runtime range behavior on real
  hardware. The expected behavior is:
  - QCPU `Z` is selected by probing `Z15` and resolves to 10 or 16 points.
  - LCPU/QnU/QnUDV `Z` is fixed at 20 points.
  - QCPU/LCPU/QnU/QnUDV `ZR` is selected by probing readable addresses and may
    resolve to 0 points.
  - `R` follows the probed `ZR` count and is capped at `R32767`.

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

## 4. Completed Recently

- [x] **Validate iQ-F X/Y octal handling on sibling stacks**: FX5UC-32MT/D returned `X0000-X1777` and `Y0000-Y1777` as `Base8`; `X100` and `Y100` read successfully through iQ-F octal address parsing in .NET, Python, and Node-RED. Rust still needs Cargo-based local execution.
- [x] **Resolve Q-series runtime device ranges**: QCPU/LCPU/QnU/QnUDV `ZR` ranges are selected by probing readable addresses, `R` follows the probed `ZR` count capped at `R32767`, QCPU `Z` is selected by probing `Z15`, and LCPU/QnU/QnUDV `Z` is fixed at 20 points.
