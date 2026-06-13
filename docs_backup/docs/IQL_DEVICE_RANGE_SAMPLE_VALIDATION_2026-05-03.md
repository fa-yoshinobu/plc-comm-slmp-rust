# iQ-L Device Range Sample Validation

Date: 2026-05-03

Target:

- PLC: Mitsubishi iQ-L / L16HCPU
- Host: `192.168.250.100`
- Port: `1025`
- Transport: TCP
- Rust profile: `SLMP_PLC_FAMILY=iq-l` (`Frame4E` + `Iqr`)

Additional live path checked the same day:

- Port: `1027`
- Transport: UDP

Command:

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-l \
cargo run --features cli --example device_range_sample_compare
```

Method:

- Read the live iQ-L device-range catalog.
- For each supported device code, sample up to 10 addresses:
  - first address
  - second address where available
  - middle and distributed points
  - final address
- For each sampled address:
  - read original value
  - write test value A
  - verify read-back
  - write test value B
  - verify read-back
  - restore original value
- For normal bit devices, compare contiguous bit reads with `DeviceReadBlock`
  bit-block reads. The block command returns packed 16-bit `bit_values`, matching
  the .NET `BitWordValues` behavior.

Result:

- Passed: 352 sampled points
- Read failures: 0
- Write failures: 0
- Restore failures: 0
- Skipped: 0
- Unsupported: 1 (`S`)
- Bit-block checks: 129 passed, 0 failed

UDP `1027` result:

- Passed: 352 sampled points
- Read failures: 0
- Write failures: 0
- Restore failures: 0
- Skipped: 0
- Unsupported: 1 (`S`)
- Bit-block checks: 129 passed, 0 failed
- Android Rust bridge and iOS C ABI returned the same pass counts over UDP.

Passed device codes:

`X`, `Y`, `M`, `B`, `SB`, `F`, `V`, `L`, `D`, `W`, `SW`, `R`, `TS`, `TC`,
`TN`, `STS`, `STC`, `STN`, `CS`, `CC`, `CN`, `LTS`, `LTC`, `LTN`, `LSTS`,
`LSTC`, `LSTN`, `LCS`, `LCC`, `LCN`, `Z`, `LZ`, `ZR`, `RD`, `SM`, `SD`.

Unsupported:

- `S`: the live iQ-L catalog reports `S0-S1023`, but the Rust parser/client does
  not currently expose the MELSEC step-relay device. Keep `S` out of app support
  until parser/client behavior is intentionally added and validated.

Notes:

- `LZ0` and `LZ1` initially exposed a Rust helper route bug: `read_typed(..., "D")`
  was using the guarded direct dword path. The helper was corrected to use random
  dword access for `LZ`, then both `LZ0` and `LZ1` passed.
- `DeviceReadBlock` bit blocks are not boolean arrays. They are packed word
  values; for example one bit-block point covers 16 bit devices. The validation
  compares `read_bits` packed into words against `read_block(...).bit_values`.
- `SM` bit-block validation is read-only because a 16-bit block can include
  system relays that do not accept arbitrary writes.

## Route Validation

The Rust route validation harness was added after comparing the more advanced
`plc-comm-slmp-dotnet` random, block, typed, and guard behavior. It checks that
high-level helper routes and direct command routes agree on the live PLC.

Command:

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-l \
cargo run --features cli --example route_validation_compare
```

Method:

- Compare mixed `read_block` values with direct reads.
- Verify `write_block` roundtrip behavior.
- Compare `read_random` word/dword results with direct and typed reads.
- Verify `write_random_words` and `write_random_bits` roundtrip behavior.
- Verify typed `U`, `D`, and `F` roundtrips.
- Verify `LZ` typed `D` uses the random dword route.
- Confirm invalid `LZ` routes are rejected by the library.
- Confirm range errors across representative device families and routes.

Result:

- Passed: 18
- Failed: 0
- Warned: 1
- Skipped: 0

The same shared route checker was invoked through the Android Rust bridge and
iOS C ABI on 2026-05-03 with the same summary: `passed=18`, `failed=0`,
`warned=1`, `skipped=0`.

UDP `1027` route validation returned the same summary through Rust,
Android Rust bridge, and iOS C ABI: `passed=18`, `failed=0`, `warned=1`,
`skipped=0`.

Route warning:

- `ZR786432` is outside the live iQ-L `ZR` range. Direct read/write, typed
  access, block access, and random write returned range error `0x4031`.
  However, random read of the same word address succeeded and returned `0xFFFF`.
  The .NET CLI showed the same behavior, so this is recorded as a live PLC route
  warning rather than a Rust-only failure.
