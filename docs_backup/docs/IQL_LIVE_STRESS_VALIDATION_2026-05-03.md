# iQ-L Live Stress Validation - 2026-05-03

## Target

- PLC: iQ-L `L16HCPU`
- Host: `192.168.250.100`
- TCP: `1025`
- UDP: `1027`
- Profile: `SLMP_PLC_FAMILY=iq-l` (`Frame3E` + `Iqr`)
- PLC was connected directly to this PC during this run.

## Harness

```sh
SLMP_HOST=192.168.250.100 \
SLMP_TCP_PORT=1025 \
SLMP_UDP_PORT=1027 \
SLMP_STRESS_TRANSPORTS=tcp,udp \
cargo run --features cli --example iql_live_stress --quiet
```

The harness performs write/read-back/restore checks. It does not downgrade NG to
read-only success.

## Passed

Both TCP `1025` and UDP `1027` passed:

- Direct word max request: `D14000`, 960 words.
- Direct dword max request: `D15000`, 480 dwords.
- Chunked word ordering/restore: `D16000`, 1024 words, 128-word chunks.
- Chunked dword ordering/restore: `D17000`, 512 dwords, 64-dword chunks.
- Direct bit write/read-back/restore: `M100`, 960 bits.
- Random word/dword write/read-back/restore at the current reference size:
  `SLMP_RANDOM_DEVICE_POINTS=48`.
- Mixed random write/read-back/restore at 24 word + 24 dword points.
- One word-block + one bit-block write/read-back/restore.
- Typed helper write/read-back/restore for `D:U`, `D:D`, `D:F`, `M:BIT`,
  `LZ:D`, and `RD:U`.
- Local route-limit errors:
  - `read_words_single_request` count 961.
  - `read_dwords_single_request` count 481.
  - `write_words_single_request` length 961.
  - `read_random` word count 256.
- TCP and UDP wrong-port and reconnect behavior.

## Observed NG / Reference Data

These are recorded as live target observations, not as fixed library-wide
limits:

- Random read probe at 49 word points succeeded.
- Separate reference probes showed random read succeeds at 96 word points and
  fails at 97 word points with `0xC054` on this PLC.
- Random write at 96 word points failed with `0xC054` on this PLC. Random
  write/read-back/restore passed at 48 points.
- Two word-block write candidate failed with `0xC05B` on both TCP and UDP. The
  one word-block + one bit-block route passed.
- Before fixing `SlmpClient::close`, TCP reconnect inside the full stress
  process failed after the route sequence: `Connection refused (os error 61)`
  for three reconnect attempts. The cause was that `close()` only cleared frame
  buffers and did not shut down the TCP stream. After making `close()` shut down
  the TCP stream and dropping the main stress client before reconnect probes,
  TCP reconnect passed.

## Verification Commands

```sh
cargo check --features cli --example iql_live_stress
cargo test --test route_guards
```

Results:

- `cargo check --features cli --example iql_live_stress`: passed.
- `cargo test --test route_guards`: 17 passed.
