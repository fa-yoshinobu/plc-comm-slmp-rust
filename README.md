[![CI](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/plc-comm-slmp-rust.svg)](https://crates.io/crates/plc-comm-slmp-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

# MELSEC SLMP for Rust

Rust library for MELSEC SLMP (Binary 3E/4E) PLC communication.

## Supported PLC profiles

The maintained profile table is in [PLC profiles](docs/PROFILES.md). Choose one exact canonical PLC profile from that table.

## Supported device types

The maintained device and range tables are in [Supported registers](docs/SUPPORTED_REGISTERS.md). Use that page for supported device families, address syntax, and profile-specific notes.

## Installation

```bash
cargo add plc-comm-slmp-rust
```

The package name is `plc-comm-slmp-rust`; the Rust import path is `plc_comm_slmp`.

## Quick example

```rust
use plc_comm_slmp::{
    read_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let value = read_typed(&client, SlmpAddress::parse("D100")?, "U").await?;
    println!("{:?}", value);
    client.close().await?;

    Ok(())
}
```

## Documentation

| Page | Use it for |
| --- | --- |
| [Full documentation site](https://fa-yoshinobu.github.io/plc-comm-docs-site/) | Unified docs for all PLC communication libraries. |
| [Getting started](docs/GETTING_STARTED.md) | Install the crate, connect to your PLC, and run your first SLMP read/write. |
| [Usage guide](docs/USAGE_GUIDE.md) | Use the high-level API and common SLMP workflows. |
| [Supported registers](docs/SUPPORTED_REGISTERS.md) | Check supported device families, address syntax, and numbering rules. |
| [PLC profiles](docs/PROFILES.md) | Choose the canonical MELSEC profile and frame behavior. |
| [Examples](examples/README.md) | Run maintained Rust examples. |

## Hardware verified

Live-device verification is maintained in [Latest communication verification](docs/LATEST_COMMUNICATION_VERIFICATION.md).
See that page for verified PLC models, transports, dates, limitations, and retained validation notes.

## License and registry

| Item | Value |
| --- | --- |
| License | [MIT](LICENSE) |
| Registry | [crates.io](https://crates.io/crates/plc-comm-slmp-rust) |
| Package | `plc-comm-slmp-rust` |
