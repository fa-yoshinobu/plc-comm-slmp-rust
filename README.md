[![CI](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/fa-yoshinobu/plc-comm-slmp-rust)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/LICENSE)

# SLMP Protocol for Rust

Async Rust implementation of the SLMP library, based on the `plc-comm-slmp-dotnet`
implementation and aligned with the shared `plc-comm-slmp-cross-verify` harness.

The crate focuses on Binary 3E / 4E SLMP over TCP and UDP and keeps the same
operation meaning as the existing Python, .NET, C++, and Node-RED libraries.

## Current Scope

- raw device access: word, bit, dword, float32
- random read/write
- block read/write
- extended-device read/write
- memory read/write
- extend-unit read/write
- remote operations and self-test
- high-level typed helpers
- high-level named read/write and polling helpers
- `slmp_verify_client` wrapper for `plc-comm-slmp-cross-verify`
- minimal `napi-rs` Node binding scaffold in `crates/slmp-node`

## Quick Start

Add the library as a path dependency while the crate is still GitHub-first:

```toml
[dependencies]
plc-comm-slmp-rust = { git = "https://github.com/fa-yoshinobu/plc-comm-slmp-rust" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Raw Client Usage

```rust
use plc_comm_slmp::{
    SlmpAddress, SlmpClient, SlmpCompatibilityMode, SlmpConnectionOptions, SlmpFrameType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100");
    options.port = 1025;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;

    let client = SlmpClient::connect(options).await?;
    let values = client.read_words_raw(SlmpAddress::parse("D100")?, 2).await?;
    println!("{values:?}");
    Ok(())
}
```

### Recommended High-Level Usage

```rust
use plc_comm_slmp::{
    read_named, write_named, NamedAddress, SlmpClient, SlmpCompatibilityMode,
    SlmpConnectionOptions, SlmpFrameType, SlmpValue,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100");
    options.port = 1025;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;

    let client = SlmpClient::connect(options).await?;

    let snapshot = read_named(
        &client,
        &["D100".into(), "D200:F".into(), "D50.3".into()],
    )
    .await?;

    println!("{:?}", snapshot["D100"]);
    println!("{:?}", snapshot["D200:F"]);
    println!("{:?}", snapshot["D50.3"]);

    let mut updates = NamedAddress::new();
    updates.insert("D300".into(), SlmpValue::U16(42));
    updates.insert("D400:F".into(), SlmpValue::F32(3.14));
    write_named(&client, &updates).await?;

    Ok(())
}
```

### Typed Access

```rust
use plc_comm_slmp::{read_typed, write_typed, SlmpAddress, SlmpValue};

# async fn demo(client: &plc_comm_slmp::SlmpClient) -> Result<(), plc_comm_slmp::SlmpError> {
let temperature = read_typed(client, SlmpAddress::parse("D200")?, "F").await?;
let position = read_typed(client, SlmpAddress::parse("D300")?, "L").await?;

write_typed(client, SlmpAddress::parse("D100")?, "U", &SlmpValue::U16(42)).await?;
write_typed(client, SlmpAddress::parse("D200")?, "F", &SlmpValue::F32(3.14)).await?;
write_typed(client, SlmpAddress::parse("D300")?, "L", &SlmpValue::I32(-100)).await?;
# Ok(())
# }
```

## Public API Surface

Main exports:

- `SlmpConnectionOptions`
- `SlmpClient`
- `SlmpAddress`
- `read_typed` / `write_typed`
- `write_bit_in_word`
- `read_named` / `write_named`
- `poll_named`
- `read_words_single_request` / `read_dwords_single_request`
- `read_words_chunked` / `read_dwords_chunked`
- `write_words_single_request` / `write_dwords_single_request`
- `write_words_chunked` / `write_dwords_chunked`

Important model types:

- `SlmpDeviceAddress`
- `SlmpQualifiedDeviceAddress`
- `SlmpTargetAddress`
- `SlmpExtensionSpec`
- `SlmpTypeNameInfo`
- `SlmpRandomReadResult`
- `SlmpBlockRead`, `SlmpBlockWrite`, `SlmpBlockReadResult`
- `SlmpLongTimerResult`
- `SlmpValue`

## Supported Address Forms

High-level helpers are intended to cover these forms first:

- plain word devices: `D100`, `R50`, `ZR0`, `TN0`, `CN0`
- plain bit devices: `M1000`, `X20`, `Y20`, `B10`
- typed suffixes: `D100:S`, `D200:D`, `D300:L`, `D400:F`
- bit-in-word form: `D50.3`
- long current-value forms: `LTN10:D`, `LSTN20:D`, `LCN30:D`
- extended devices: `J1\\SW0`, `U3\\G100`, `U1\\HG0`

`.bit` notation is only valid for word devices. Address bit devices directly.

## Cross-Verify

This repo is designed to participate in:

- `plc-comm-slmp-cross-verify/specs/shared`
- `python verify.py --clients rust`
- full parity runs with Python, .NET, C++, and Node-RED
- live PLC verification through the same saved baseline/profile flow

The wrapper binary used by the harness is:

```bash
cargo run --bin slmp_verify_client -- 127.0.0.1 9000 read-type
```

## Development

Run the Rust tests:

```bash
cargo test
```

Check the Node binding scaffold:

```bash
cargo check -p slmp-node
```

Run Rust-only parity through the canonical harness:

```bash
cd ../plc-comm-slmp-cross-verify
python verify.py --clients rust
```

Run full parity:

```bash
cd ../plc-comm-slmp-cross-verify
python verify.py
```

Run live PLC verification with the validated R120 profile:

```bash
cd ../plc-comm-slmp-cross-verify
python slmp_live_verify.py \
  --ip 192.168.250.100 \
  --port 1025 \
  --profile r120pcpu_tcp1025 \
  --include-stateful \
  --include-remote
```

## Node Binding

`crates/slmp-node` is currently a thin `napi-rs` scaffold.  
It is included so the Rust core can be expanded into a Node package later
without redesigning the workspace layout.

## License

Distributed under the MIT License.
