[![CI](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/plc-comm-slmp.svg)](https://crates.io/crates/plc-comm-slmp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/LICENSE)

# MELSEC SLMP for Rust

Rust library for MELSEC SLMP (Binary 3E/4E) PLC communication.

## PLC Comm Family

This library is part of the plc-comm family. See the [package matrix](https://fa-yoshinobu.github.io/plc-comm-docs-site/package-matrix/) for protocol, language, registry, and install-command mapping.

## Supported PLC profiles

The maintained profile table is in [PLC profiles](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/rust/PROFILES/). Choose one exact canonical PLC profile from that table.

## Supported device types

The maintained device and range tables are in the [SLMP Profile Reference](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/). Use that page for supported device families, address syntax, and profile-specific notes.

## Installation

```bash
cargo add plc-comm-slmp
```

The package name is `plc-comm-slmp`; the Rust import path is `plc_comm_slmp`.

## Quick example

```rust
use plc_comm_slmp::{
    read_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR)?;
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
| [Getting started](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/rust/GETTING_STARTED/) | Install the crate, connect to your PLC, and run your first SLMP read/write. |
| [Usage guide](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/rust/USAGE_GUIDE/) | Use the high-level API and common SLMP workflows. |
| [SLMP profile reference](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/) | Check profile parameters, device families, address syntax, and numbering rules. |
| [PLC profiles](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/rust/PROFILES/) | Choose the canonical MELSEC profile and frame behavior. |
| [Examples](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/examples/README.md) | Run maintained Rust examples. |

## License and registry

| Item | Value |
| --- | --- |
| License | [MIT](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/LICENSE) |
| Registry | [crates.io](https://crates.io/crates/plc-comm-slmp) |
| Package | `plc-comm-slmp` |

## Commercial support

If you plan to embed this library in a paid or commercial product, please consider a separate support agreement or supporting the project as a sponsor.

Contact: <https://fa-labo.com/contact.html>
