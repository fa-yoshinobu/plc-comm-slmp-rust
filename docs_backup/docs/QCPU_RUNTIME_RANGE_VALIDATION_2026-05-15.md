# QCPU Validation 2026-05-15

## Target

- Host: `192.168.250.100`
- Port: `1025`
- Transport: TCP
- Access profile: 3E frame, legacy compatibility
- Family selected for catalog: `QCPU`
- Connected PLC: Mitsubishi `Q12HCPU`

This file is the current Q12HCPU/QCPU record for 2026-05-15 and supersedes the
earlier same-day notes that only covered a subset of the checks.

## Range Command

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_TRANSPORT=tcp \
SLMP_PLC_FAMILY=qcpu \
SLMP_PLC_TYPE=qcpu \
cargo run --features cli --example device_range_catalog
```

## Runtime Range Results

The catalog command completed successfully. The PLC reported `model=QCPU`,
`model_code=0x0000`, and `family=QCpu`.

The resulting QCPU runtime catalog reports:

- `Z0-Z15`, 16 points
- `ZR0-ZR131071`, 131072 points
- `R0-R32767`, 32768 points

This confirms that the QCPU `Z` range resolves to the 16-point form on the
connected Q12HCPU target. `ZR` resolves from readable-address probing, and `R`
matches the probed `ZR` size capped at `R32767`.

## Cross-Stack Route Results

The same Q12HCPU target was checked through all maintained SLMP stacks:

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
`../plc-comm-slmp-cross-verify/QCPU_CROSS_STACK_VALIDATION_2026-05-15.md`.
