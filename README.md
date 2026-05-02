[![CI](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/plc-comm-slmp-rust.svg)](https://crates.io/crates/plc-comm-slmp-rust)
[![docs.rs](https://img.shields.io/docsrs/plc-comm-slmp-rust)](https://docs.rs/plc-comm-slmp-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/LICENSE)

# SLMP Protocol for Rust

Async Rust implementation of the SLMP library, based on the `plc-comm-slmp-dotnet`
implementation and aligned with the shared `plc-comm-slmp-cross-verify` harness.

The crate focuses on Binary 3E / 4E SLMP over TCP and UDP and keeps the same
operation meaning as the existing Python, .NET, C++, Node-RED, and Rust
verification clients.

## What This Repo Contains

- async Rust library crate: `src/`
- `cross-verify` wrapper binary: `src/bin/slmp_verify_client.rs`
- runnable examples: [`examples/`](examples)
- address and usage guides: [`docs/`](docs)
- minimal `napi-rs` workspace member for future Node packaging: `crates/slmp-node`

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
- live device-range catalog via user-selected PLC family plus family `SD` registers
- `slmp_verify_client` wrapper for `plc-comm-slmp-cross-verify`
- minimal `napi-rs` Node binding scaffold in `crates/slmp-node`

## Installation

Install from crates.io:

```bash
cargo add plc-comm-slmp-rust
```

The public package name is `plc-comm-slmp-rust`, and the library import path is
`plc_comm_slmp`.

Requires Rust 1.85 or newer.

`Cargo.toml`:

```toml
[dependencies]
plc-comm-slmp-rust = "0.1.9"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

### Raw Client Usage

```rust
use plc_comm_slmp::{
    SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcFamily,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcFamily::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let values = client.read_words_raw(SlmpAddress::parse("D100")?, 2).await?;
    println!("{values:?}");
    Ok(())
}
```

### Recommended High-Level Usage

```rust
use plc_comm_slmp::{
    read_named, write_named, NamedAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcFamily,
    SlmpValue,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcFamily::IqF);
    options.port = 1025;

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

## Runnable Examples

The repository includes examples that compile as part of the crate and can be
run directly against a PLC or mock server.

### `raw_read_write`

Low-level word read plus optional write/read-back.

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-r \
cargo run --features cli --example raw_read_write
```

Enable writes explicitly:

```bash
SLMP_ENABLE_WRITES=1 \
SLMP_WRITE_ADDRESS=D600 \
SLMP_WRITE_VALUES=111,222 \
cargo run --features cli --example raw_read_write
```

### `named_helpers`

Named snapshot, typed decoding, optional `write_named`, and one `poll_named`
tick.

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PLC_FAMILY=iq-f \
SLMP_NAMED_ADDRESSES='D100,D200:F,D50.3,LTN10:D,LTS10' \
cargo run --features cli --example named_helpers
```

### `advanced_operations`

Safe read-heavy sample that covers type-name, random read, block read, extended
device read, and self-test loopback.

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PLC_FAMILY=iq-r \
SLMP_RANDOM_WORDS='D100,R10' \
SLMP_RANDOM_DWORDS='D200,LTN10' \
SLMP_EXT_DEVICE='J1\W10' \
cargo run --features cli --example advanced_operations
```

### `device_range_catalog`

Reads the family-specific `SD` window for a user-selected PLC family and prints
`points` plus formatted address ranges such as `X0000-X2FFF`.

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-f \
cargo run --features cli --example device_range_catalog
```

### `connection_profile_probe`

Tries the standard frame and compatibility candidates, reads `read_type_name`,
then validates the resolved family by reading its full `SD` block once. The
result is advisory only so the caller can choose which settings to use.

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
cargo run --features cli --example connection_profile_probe
```

### `device_matrix_compare`

Real-PLC regression sample that writes the same address through multiple command
paths and checks that read-back stays aligned.

- bit devices: `write_bits`, `write_random_bits`, `write_typed`, `write_named`, raw `request`
- word devices: `write_words`, `write_random_words`, `write_typed`, `write_named`, raw `request`
- 32-bit devices: `write_dwords`, `write_random_words`, `write_typed`, `write_named`, raw `request`
- `J1\\...` devices: extended helper APIs plus raw `request`

```bash
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_FAMILY=iq-r \
cargo run --features cli --example device_matrix_compare
```

This example exits non-zero when command paths for the same address disagree.

Focus on a subset while debugging:

```bash
SLMP_COMPARE_ONLY='LTS10,LTC10,LCS10,LCC10,LTN10,LSTN10' \
cargo run --features cli --example device_matrix_compare
```

The shared environment variables for these examples are documented in
[`docs/RECIPES.md`](docs/RECIPES.md).

For live PLC-dependent device limits resolved from a user-selected PLC family
plus family `SD` registers, see [`docs/DEVICE_RANGES.md`](docs/DEVICE_RANGES.md).

## Public API Surface

Main exports:

- `SlmpConnectionOptions`
- `SlmpClient`
- `SlmpAddress`
- `probe_connection_profiles`
- `read_type_name` / `read_device_range_catalog` / `read_device_range_catalog_for_family`
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
- `SlmpDeviceRangeCatalog`, `SlmpDeviceRangeEntry`, `SlmpDeviceRangeFamily`
- `SlmpRandomReadResult`
- `SlmpBlockRead`, `SlmpBlockWrite`, `SlmpBlockReadResult`
- `SlmpLongTimerResult`
- `SlmpValue`

## Supported Address Forms

High-level helpers are intended to cover these forms first.

- plain word devices: `D100`, `R50`, `ZR0`, `TN0`, `CN0`
- plain bit devices: `M1000`, `X20`, `Y20`, `B10`
- typed suffixes: `D100:S`, `D200:D`, `D300:L`, `D400:F`
- bit-in-word form: `D50.3`
- long current-value forms: `LTN10:D`, `LSTN20:D`, `LCN30:D`
- extended devices: `J1\\SW0`, `U3\\G100`, `U1\\HG0`

`.bit` notation is only valid for word devices. Address bit devices directly.

See also:

- [`docs/ADDRESS_FORMS.md`](docs/ADDRESS_FORMS.md)
- [`docs/DEVICE_RANGES.md`](docs/DEVICE_RANGES.md)
- [`docs/RECIPES.md`](docs/RECIPES.md)

## Choosing the Right API

- Use raw device methods when you need exact SLMP request control.
- Use `read_typed` and `write_typed` when one address maps to one scalar value.
- Use `read_named` and `write_named` when your application needs a snapshot with
  mixed dtypes and bit-in-word decoding.
- Use `poll_named` for a lightweight periodic stream.
- Use `read_random` and `read_block` when you want to keep request counts low.
- Use the extended-device methods for `J...` and `U...` paths.
- `read_named` and `write_named` currently target plain device addresses, not
  `J...` or `U...` qualified addresses.

## Long-Family Behavior

The Rust implementation follows the same normalized behavior as the other
libraries:

- `LTN`, `LSTN`, `LCN`, and `LZ` default to 32-bit reads
- `LCN` high-level reads and writes use random dword access (`0x0403` / `0x1402`)
- `LTS`, `LTC`, `LSTS`, `LSTC`, `LCS`, and `LCC` are state reads
- `LCS` and `LCC` reads use direct bit read through `read_typed` / `read_named`
- `LCS` and `LCC` are rejected for `Read Random (0x0403)`, `Read Block (0x0406)`,
  `Write Block (0x1406)`, and `Entry Monitor Device (0x0801)`
- direct bit reads/writes (`0x0401` / `0x1401`) and Read Random (`0x0403`) for
  `LTS`, `LTC`, `LSTS`, and `LSTC` are rejected by the Rust client API; use
  helper APIs, random bit write (`0x1402`), or 4-word block reads from
  `LTN` / `LSTN`
- direct bit writes (`0x1401`) for `LCS` and `LCC` are also rejected; use
  `write_typed` / `write_named` so random bit write (`0x1402`) is selected
- direct dword reads for `LTN`, `LSTN`, `LCN`, and `LZ` are rejected; use helper APIs,
  random dword high-level access, or explicit 4-word block reads where supported

Route guard note: keep low-level direct bit routes guarded for
`LTS`/`LTC`/`LSTS`/`LSTC` and direct bit writes guarded for `LCS`/`LCC`.
High-level writes must continue to resolve to random bit write (`0x1402`).

That behavior is intentional and is enforced through
`plc-comm-slmp-cross-verify`.

## Cross-Verify

This repo is designed to participate in:

- `plc-comm-slmp-cross-verify/specs/shared`
- `python verify.py --clients rust`
- full parity runs with Python, .NET, C++, and Node-RED
- live PLC verification through the same saved baseline/profile flow

The wrapper binary used by the harness is:

```bash
cargo run --features cli --bin slmp_verify_client -- 127.0.0.1 9000 read-type
```

## Development

Format and test:

```bash
cargo fmt
cargo test
```

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

`crates/slmp-node` is currently a thin `napi-rs` scaffold. It is not yet the
main delivery path. The current purpose is to keep the Rust workspace ready for
future Node package work without redesigning the crate layout later.

## License

Distributed under the MIT License.
