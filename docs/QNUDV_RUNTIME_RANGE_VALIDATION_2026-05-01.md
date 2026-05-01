# QnUDV Runtime Range Validation 2026-05-01

## Target

- Host: `192.168.250.100`
- Port: `1025`
- Transport: TCP
- Access profile: 3E frame, legacy compatibility
- Family selected for catalog: `QnUDV`

## Commands

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_TRANSPORT=tcp \
SLMP_PLC_FAMILY=qnudv \
SLMP_PLC_TYPE=qnudv \
SLMP_TIMEOUT_MS=5000 \
cargo run --features cli --example device_range_catalog
```

Boundary reads used `slmp_verify_client` with `--family qnudv --frame 3e --series ql`.

## Results

| Check | Result |
| --- | --- |
| `Z0` read | OK |
| `Z19` read | OK |
| `Z20` read | `0x4031` |
| `ZR0` read | OK |
| `ZR393215` read | OK |
| `ZR393216` read | `0x4031` |
| `R0` read | OK |
| `R32767` read | OK |
| `R32768` read | `0x4031` |

The resulting catalog reports:

- `Z0-Z19`, 20 points
- `ZR0-ZR393215`, 393216 points
- `R0-R32767`, 32768 points

`Read Type Name` returned `0xC059` on this target, so this validation used the explicit `QnUDV` family path rather than type-name auto-resolution.
