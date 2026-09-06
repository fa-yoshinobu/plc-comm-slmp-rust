[![CI](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/plc-comm-slmp.svg)](https://crates.io/crates/plc-comm-slmp)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/LICENSE)

# MELSEC SLMP for Rust

Rust library for MELSEC SLMP (Binary 3E/4E) PLC communication.

## PLC Comm Family

This library is part of the plc-comm family. See the [package matrix](https://plc-comm-docs-site.fa-labo.com/package-matrix/) for protocol, language, registry, and install-command mapping.

## Supported PLC profiles

The maintained profile table is in [PLC profiles](https://plc-comm-docs-site.fa-labo.com/slmp/rust/PROFILES/). Choose one exact canonical PLC profile from that table.

## Supported device types

The maintained device and range tables are in the [SLMP Profile Reference](https://plc-comm-docs-site.fa-labo.com/slmp/profile-reference/). Use that page for supported device families, address syntax, and profile-specific notes.

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
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let value = read_typed(&client, SlmpAddress::parse("D100", SlmpPlcProfile::IqR)?, "U").await?;
    println!("{:?}", value);
    client.close().await?;

    Ok(())
}
```

## Documentation

| Page | Use it for |
| --- | --- |
| [Full documentation site](https://plc-comm-docs-site.fa-labo.com/) | Unified docs for all PLC communication libraries. |
| [Getting started](https://plc-comm-docs-site.fa-labo.com/slmp/rust/GETTING_STARTED/) | Install the crate, connect to your PLC, and run your first SLMP read/write. |
| [Usage guide](https://plc-comm-docs-site.fa-labo.com/slmp/rust/USAGE_GUIDE/) | Use the high-level API and common SLMP workflows. |
| [API reference](https://plc-comm-docs-site.fa-labo.com/slmp/rust/API_REFERENCE/) | Find public client methods, helpers, profile APIs, and error types. |
| [SLMP profile reference](https://plc-comm-docs-site.fa-labo.com/slmp/profile-reference/) | Check profile parameters, device families, address syntax, and numbering rules. |
| [PLC profiles](https://plc-comm-docs-site.fa-labo.com/slmp/rust/PROFILES/) | Choose the canonical MELSEC profile and frame behavior. |
| [Gotchas](https://plc-comm-docs-site.fa-labo.com/slmp/rust/GOTCHAS/) | Check whether this library has any current library-specific caveats. |
| [Performance](https://plc-comm-docs-site.fa-labo.com/performance/) | See measured latency, throughput, and long-run soak results from real PLC hardware. |
| [Choosing a Language](https://plc-comm-docs-site.fa-labo.com/choosing-a-language/) | Compare the .NET, Python, Rust, C++, and Node-RED implementations before you pick one. |
| [Examples](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/examples/README.md) | Run maintained Rust examples. |

For a zero-code connectivity check, see [PLC Scope](https://github.com/fa-yoshinobu/plc-scope-dotnet) (Windows).

## License and registry

| Item | Value |
| --- | --- |
| License | [MIT](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/LICENSE) |
| Registry | [crates.io](https://crates.io/crates/plc-comm-slmp) |
| Package | `plc-comm-slmp` |

## Commercial support

If you plan to embed this library in a paid or commercial product, please consider a separate support agreement or supporting the project as a sponsor.

Contact: <https://fa-labo.com/contact.html>
