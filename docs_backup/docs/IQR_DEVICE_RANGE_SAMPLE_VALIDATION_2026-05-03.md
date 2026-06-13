# iQ-R Device Range Sample Validation

Date: 2026-05-03

Target:

- PLC: Mitsubishi iQ-R / R08CPU
- Host: `192.168.250.100`
- TCP port: `1025`
- UDP port: `1027`
- Rust profile: `SLMP_PLC_FAMILY=iq-r` (`Frame4E` + `Iqr`)

Commands:

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-r \
SLMP_TRANSPORT=tcp \
cargo run --features cli --example device_range_sample_compare

SLMP_HOST=192.168.250.100 \
SLMP_PORT=1027 \
SLMP_PLC_FAMILY=iq-r \
SLMP_TRANSPORT=udp \
cargo run --features cli --example device_range_sample_compare
```

Result:

- Model: `R08CPU`
- Passed: 352 sampled points
- Read failures: 0
- Write failures: 0
- Restore failures: 0
- Skipped: 0
- Unsupported: 1 (`S`)
- Bit-block checks: 135 passed, 0 failed

The same shared all-device checker was invoked through the Android Rust bridge
and iOS C ABI over both TCP `1025` and UDP `1027` with the same pass counts.

Passed device codes:

`X`, `Y`, `M`, `B`, `SB`, `F`, `V`, `L`, `D`, `W`, `SW`, `R`, `TS`, `TC`,
`TN`, `STS`, `STC`, `STN`, `CS`, `CC`, `CN`, `LTS`, `LTC`, `LTN`, `LSTS`,
`LSTC`, `LSTN`, `LCS`, `LCC`, `LCN`, `Z`, `LZ`, `ZR`, `RD`, `SM`, `SD`.

Unsupported:

- `S`: the live iQ-R catalog reports `S`, but the Rust parser/client does not
  currently expose the MELSEC step-relay device. Keep `S` out of app support
  until parser/client behavior is intentionally added and validated.

Notes:

- The first full TCP run was made while the X unit was installed. `X0` stayed
  ON and write/read-back failed for `X0` plus bit-block writes starting at `X0`
  and `X1`. After the X unit was removed, the full all-device test passed over
  both TCP and UDP.
- The live iQ-R `R08CPU` reports `D0-D10239` and `ZR0-ZR65535`, which are
  smaller than the previously checked iQ-L `L16HCPU` ranges.

## Route Validation

Commands:

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-r \
SLMP_TRANSPORT=tcp \
cargo run --features cli --example route_validation_compare

SLMP_HOST=192.168.250.100 \
SLMP_PORT=1027 \
SLMP_PLC_FAMILY=iq-r \
SLMP_TRANSPORT=udp \
cargo run --features cli --example route_validation_compare
```

Result:

- Passed: 18
- Failed: 0
- Warned: 1
- Skipped: 0

The same shared route checker was invoked through the Android Rust bridge and
iOS C ABI over both TCP `1025` and UDP `1027` with the same summary.

Route warning:

- `ZR65536` is outside the live iQ-R `ZR` range. Direct read/write, typed
  access, block access, and random write returned range error `0x4031`.
  However, random read of the same word address succeeded. This matches the
  same route-warning pattern seen on the iQ-L target.
