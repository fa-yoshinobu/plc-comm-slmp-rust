# QnUDV Device Range Sample Validation

Date: 2026-05-03

Target:

- PLC: Mitsubishi QnUDV
- Host: `192.168.250.100`
- TCP port: `1025`
- UDP port: `1027`
- Rust profile: `SLMP_PLC_FAMILY=qnudv` (`Frame3E` + `Legacy`)

Profile selection:

- Do not use connection-profile probing.
- Do not select `Frame4E` for QnUDV.
- `read_type_name` did not return on this target; the selected PLC family is the
  authority for frame, compatibility, and device-range catalog behavior.

Commands:

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=qnudv \
SLMP_TRANSPORT=tcp \
cargo run --features cli --example route_validation_compare

SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=qnudv \
SLMP_TRANSPORT=tcp \
cargo run --features cli --example device_range_sample_compare

SLMP_HOST=192.168.250.100 \
SLMP_PORT=1027 \
SLMP_PLC_FAMILY=qnudv \
SLMP_TRANSPORT=udp \
cargo run --features cli --example route_validation_compare

SLMP_HOST=192.168.250.100 \
SLMP_PORT=1027 \
SLMP_PLC_FAMILY=qnudv \
SLMP_TRANSPORT=udp \
cargo run --features cli --example device_range_sample_compare
```

Route validation result:

- TCP `1025`: `passed=9`, `failed=0`, `warned=0`, `skipped=10`
- UDP `1027`: `passed=9`, `failed=0`, `warned=0`, `skipped=10`
- `DeviceReadBlock` / `DeviceWriteBlock` are skipped for QnUDV (`0x0406` /
  `0x1406` unsupported).
- Random read/write routes are skipped for QnUDV (`0x0403` / `0x1402`
  unsupported).
- `LZ`, `RD`, and long timer/counter devices are unsupported by the QnUDV
  catalog.
- Direct typed word/dword/float round-trip passed on `D9000`, `D9002`, and
  `D9004`.
- Out-of-range checks passed on direct bit/word/typed routes for `X`, `Y`, `M`,
  `D`, `R`, `ZR`, `SM`, and `SD`.

Route skipped cases:

- `read_block_matches_direct`
- `write_block_roundtrip`
- `lz_blocks_rejected`
- `read_random_matches_direct`
- `write_random_roundtrip`
- `lz_word_entries_rejected`
- `lz_random_dword_roundtrip`
- `lz_invalid_dtypes_rejected`
- `RD_out_of_range`
- `LZ_out_of_range`

Device range sample result:

- TCP `1025`: `passed=249`, `read_failed=0`, `write_failed=0`,
  `readback_failed=1`, `restore_failed=0`, `skipped=11`, `unsupported=1`,
  `bit_blocks_passed=0`, `bit_blocks_failed=0`
- UDP `1027`: same result as TCP.
- Bit-block checks are not run because QnUDV does not support the block route.
- `S` is untested because the parser/client intentionally does not expose the
  MELSEC step-relay device yet.

Sample skipped devices:

- `LTS`
- `LTC`
- `LTN`
- `LSTS`
- `LSTC`
- `LSTN`
- `LCS`
- `LCC`
- `LCN`
- `LZ`
- `RD`
- Android Rust bridge and iOS C ABI were also run over TCP `1025`. Route
  validation passed with `passed=9`, `failed=0`, `warned=0`, `skipped=10`.
  Device range sample validation failed with the same
  `readback_failed=1`, confirming that app-side bridge decoding does not hide
  the NG result.

Readback NG point:

- `SD1279`: expected written `U16`, observed `U16(0)`

Human review result:

- This readback NG point is expected PLC/system-device behavior for this target.
- It is recorded as `readback_failed` in the harness output so the raw test fact
  remains visible, but it is not treated as an unresolved library bug.

Note:

- TCP and UDP write-heavy sample runs must not be executed concurrently against
  the same PLC addresses. A simultaneous run was discarded because competing
  writes can create artificial read-back mismatches.
