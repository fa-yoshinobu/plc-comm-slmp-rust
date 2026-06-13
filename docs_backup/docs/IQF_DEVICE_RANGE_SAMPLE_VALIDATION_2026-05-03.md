# iQ-F Device Range Sample Validation

Date: 2026-05-03

Target:

- PLC: Mitsubishi iQ-F / FX5UC-32MT/D
- Host: `192.168.250.100`
- TCP port: `1025`
- UDP port: `1027`
- Rust profile: `SLMP_PLC_FAMILY=iq-f` (`Frame3E` + `Legacy`)

Profile selection:

- Select `Frame3E` + `Legacy` for iQ-F.
- Do not select `Frame4E` for iQ-F. It is unsupported by the manual.
- Do not use connection-profile probing. The selected PLC family decides the
  frame and compatibility profile.

Commands:

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-f \
SLMP_TRANSPORT=tcp \
cargo run --features cli --example device_range_sample_compare

SLMP_HOST=192.168.250.100 \
SLMP_PORT=1027 \
SLMP_PLC_FAMILY=iq-f \
SLMP_TRANSPORT=udp \
cargo run --features cli --example device_range_sample_compare
```

Result:

- Model: `FX5UC-32MT/D`
- Passed: 262 sampled points
- Read failures: 0
- Write failures: 0
- Restore failures: 0
- Skipped: 10
- Unsupported: 0
- Bit-block checks: 110 passed, 0 failed

The same shared all-device checker was invoked through the Android Rust bridge
and iOS C ABI over both TCP `1025` and UDP `1027` with the same pass counts.

Passed device codes:

`X`, `Y`, `M`, `B`, `SB`, `F`, `L`, `D`, `W`, `SW`, `R`, `TS`, `TC`, `TN`,
`STS`, `STC`, `STN`, `CS`, `CC`, `CN`, `LCS`, `LCC`, `LCN`, `Z`, `LZ`, `SM`,
`SD`.

Skipped by catalog:

`V`, `S`, `LTS`, `LTC`, `LTN`, `LSTS`, `LSTC`, `LSTN`, `ZR`, `RD`.

Important ranges:

- `X0000-X1777`, `Y0000-Y1777`
- `M0-M7679`, `L0-L7679`
- `D0-D7999`
- `R0-R32767`
- `LCS0-LCS63`, `LCC0-LCC63`, `LCN0-LCN63`
- `Z0-Z19`, `LZ0-LZ1`
- `SM0-SM9999`, `SD0-SD11999`

## Route Validation

Commands:

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-f \
SLMP_TRANSPORT=tcp \
cargo run --features cli --example route_validation_compare

SLMP_HOST=192.168.250.100 \
SLMP_PORT=1027 \
SLMP_PLC_FAMILY=iq-f \
SLMP_TRANSPORT=udp \
cargo run --features cli --example route_validation_compare
```

Result:

- Passed: 17
- Failed: 0
- Warned: 0
- Skipped: 2 (`ZR`, `RD`)

The same shared route checker was invoked through the Android Rust bridge and
iOS C ABI over both TCP `1025` and UDP `1027` with the same summary.

Route notes:

- The generic route validation defaults `D9000/D9002/D9004` are outside this
  iQ-F device range, so the harness now uses `D1000/D1002/D1004` when the
  selected family resolves to `IqF` and the caller did not override the devices.
- The live iQ-F target returns range error `0xC056` for out-of-range route
  checks, unlike the iQ-L/iQ-R targets where the same checks returned `0x4031`.
