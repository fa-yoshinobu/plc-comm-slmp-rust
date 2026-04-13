# Recipes

These examples are intended to be runnable as-is with environment variables.

## Common Environment Variables

| Variable | Default | Meaning |
| --- | --- | --- |
| `SLMP_HOST` | `127.0.0.1` | PLC or mock server host |
| `SLMP_PORT` | `1025` | TCP/UDP port |
| `SLMP_FRAME` | `4e` | `3e` or `4e` |
| `SLMP_SERIES` | `iqr` | `iqr` or `legacy` |
| `SLMP_TRANSPORT` | `tcp` | `tcp` or `udp` |
| `SLMP_TARGET` | unset | `SELF`, `SELF-CPU1`, `NW1-ST2`, or `NAME,NET,ST,IO,MD` |
| `SLMP_TIMEOUT_MS` | `3000` | socket timeout |
| `SLMP_MONITORING_TIMER` | `16` | SLMP monitoring timer |
| `SLMP_ENABLE_WRITES` | `0` | set `1` to enable write examples |

## Raw Read / Write

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_FRAME=4e \
SLMP_SERIES=iqr \
cargo run --example raw_read_write
```

To enable write/read-back:

```bash
SLMP_ENABLE_WRITES=1 \
SLMP_WRITE_ADDRESS=D600 \
SLMP_WRITE_VALUES=111,222 \
cargo run --example raw_read_write
```

## High-Level Named Helpers

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_NAMED_ADDRESSES='D100,D200:F,D50.3,LTN10:D,LTS10' \
cargo run --example named_helpers
```

To run the write helper too:

```bash
SLMP_ENABLE_WRITES=1 \
SLMP_NAMED_WRITE_WORD=D700 \
SLMP_NAMED_WRITE_WORD_VALUE=42 \
SLMP_NAMED_WRITE_FLOAT='D702:F' \
SLMP_NAMED_WRITE_FLOAT_VALUE=3.14 \
cargo run --example named_helpers
```

## Type Name, Random, Block, Extended, Self-Test

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_RANDOM_WORDS='D100,R10' \
SLMP_RANDOM_DWORDS='D200,LTN10' \
SLMP_EXT_DEVICE='J1\W10' \
cargo run --example advanced_operations
```

## Full Device Matrix Compare

This runs through the real-device matrix used during validation and checks that
multiple read/write paths stay aligned on the same addresses.

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_FRAME=4e \
SLMP_SERIES=iqr \
cargo run --example device_matrix_compare
```

This example exits non-zero when command routes disagree.

Target only a few devices while debugging:

```bash
SLMP_COMPARE_ONLY='LTS10,LTC10,LCS10,LCC10,LTN10,LSTN10' \
cargo run --example device_matrix_compare
```

## Cross-Verify

Rust-only run:

```bash
cd ../plc-comm-slmp-cross-verify
python verify.py --clients rust
```

Full parity:

```bash
cd ../plc-comm-slmp-cross-verify
python verify.py
```

Live profile run:

```bash
cd ../plc-comm-slmp-cross-verify
python slmp_live_verify.py \
  --ip 192.168.250.100 \
  --port 1025 \
  --profile r120pcpu_tcp1025 \
  --include-stateful \
  --include-remote
```
