# QnU Validation 2026-05-15

## Target

- Host: `192.168.250.100`
- Port: `1025`
- Transport: TCP
- Access profile: 3E frame, legacy compatibility
- Family selected for catalog: `QnU`
- Connected PLC: Mitsubishi `Q26UDEHCPU`

This file is the current QnU record for 2026-05-15. It supersedes the earlier
same-day QnU notes and records the Q26UDEHCPU results only.

## Catalog Command

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_TRANSPORT=tcp \
SLMP_PLC_FAMILY=qnu \
SLMP_PLC_TYPE=qnu \
cargo run --quiet --features cli --example device_range_catalog
```

## Runtime Range Results

The catalog command completed successfully. The PLC reported `model=QnU`,
`model_code=0x0000`, and `family=QnU`.

The resulting QnU runtime catalog reports:

- `Z0-Z19`, 20 points
- `ZR0-ZR655359`, 655360 points
- `R0-R32767`, 32768 points

## Boundary Reads

Read-only boundary checks across all maintained SLMP stacks:

- `Z19` read successfully.
- `Z20` failed as out of range.
- `ZR655359` read successfully.
- `ZR655360` failed as out of range.
- `R32767` read successfully.
- `R32768` failed as out of range or was rejected by a client guard.

Python, Node-RED, .NET, and Rust surfaced the out-of-range PLC response as
`0x4031` where the request reached the PLC. The C++ wrapper surfaced the same
PLC failures as internal error `7`.

This confirms the QnU rule on Q26UDEHCPU: `Z` is fixed at `Z0-Z19`, `ZR` is
runtime-probed, and `R` follows the checked `ZR` size capped at `R32767`.

## Cross-Stack Route Results

The same Q26UDEHCPU target was checked through all maintained SLMP stacks:

- Python
- Node-RED
- .NET
- Rust
- C++ Minimal

All stacks passed:

- direct `D9000` word write/readback/restore
- direct `Y1FFF` bit write/readback/restore
- `random-read` word device `D9000`
- `random-write-words` for `D9000`, followed by direct readback and restore
- `random-write-bits` for `Y1FFF`, followed by direct readback and restore
- word-only block read/write for `D9000`
- read-only Extended Specification `U0\G10`, returning `[0]`

The target rejected bit-block access:

- bit-only block read/write for `Y1FFF` returned `0x4031` on Python, Node-RED,
  .NET, and Rust. The C++ wrapper surfaced the same failure as internal error
  `7`.
- mixed word+bit block read returned `0x4031` on Python, Node-RED, .NET, and
  Rust.
- mixed word+bit block write returned `0xC056` on Python, Node-RED, .NET, and
  Rust.

Final restore confirmation after the checks:

- `D9000 = 0`
- `Y1FFF = 0`

The cross-stack evidence is also recorded in
`../plc-comm-slmp-cross-verify/QNU_CROSS_STACK_VALIDATION_2026-05-15.md`.
