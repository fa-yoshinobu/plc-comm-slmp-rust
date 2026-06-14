# iQ-R Extended Device Coverage 2026-05-29

- Target: `192.168.250.100:1025` TCP and `192.168.250.100:1027` UDP
- PLC: MELSEC `R08CPU`
- Profile: `SLMP_PLC_FAMILY=iq-r` (`Frame4E` + `Iqr`)
- Scope: Extended Specification write-check coverage
- Remote password: enabled on the port; validation used `SLMP_REMOTE_PASSWORD`
  to unlock before the sweep and lock again before exit.

## Command

```bash
SLMP_REMOTE_PASSWORD=<password> \
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_TRANSPORT=tcp \
SLMP_PLC_FAMILY=iq-r \
SLMP_EXT_DEVICES='U3E0\G10' \
SLMP_EXT_POINTS='1,2' \
SLMP_EXT_WRITE_CHECK=1 \
cargo run --features cli --example extended_device_coverage

SLMP_REMOTE_PASSWORD=<password> \
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1027 \
SLMP_TRANSPORT=udp \
SLMP_PLC_FAMILY=iq-r \
SLMP_EXT_DEVICES='U3E0\G10' \
SLMP_EXT_POINTS='1,2' \
SLMP_EXT_WRITE_CHECK=1 \
cargo run --features cli --example extended_device_coverage
```

## Result

| Transport | Device | Points | Direct | Result | Detail |
|---|---|---:|---:|---|---|
| TCP | `U3E0\G10` | 1 | `0xF8` | OK | before `0x0000`, wrote/read back `0x001E`, restore OK |
| TCP | `U3E0\G10` | 2 | `0xF8` | OK | before `0x0000, 0x0000`, wrote/read back `0x001E, 0x001F`, restore OK |
| UDP | `U3E0\G10` | 1 | `0xF8` | OK | before `0x0000`, wrote/read back `0x001E`, restore OK |
| UDP | `U3E0\G10` | 2 | `0xF8` | OK | before `0x0000, 0x0000`, wrote/read back `0x001E, 0x001F`, restore OK |

`HG` requires a multi-CPU coverage target and is not part of this R08CPU
single-CPU executable coverage set. `J` paths were not tested because no routed
network target was provided.
