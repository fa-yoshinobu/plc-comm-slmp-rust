[![CI](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/ci.yml)
[![docs](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/docs.yml/badge.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/actions/workflows/docs.yml)
[![crates.io](https://img.shields.io/crates/v/plc-comm-slmp-rust.svg)](https://crates.io/crates/plc-comm-slmp-rust)
[![docs.rs](https://img.shields.io/docsrs/plc-comm-slmp-rust)](https://docs.rs/plc-comm-slmp-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/fa-yoshinobu/plc-comm-slmp-rust/blob/main/LICENSE)

# SLMP Protocol for Rust

Async Rust library for Mitsubishi SLMP (Seamless Message Protocol) Binary 3E/4E PLC communication.

## Supported PLC profiles

`SlmpConnectionOptions::new(host, plc_profile)` derives the frame and compatibility mode from the selected `SlmpPlcProfile`.

| `SlmpPlcProfile` variant | Canonical profile | Hardware | Frame | Notes |
| --- | --- | --- | --- | --- |
| `SlmpPlcProfile::IqF` | `melsec:iq-f` | MELSEC iQ-F | `SlmpFrameType::Frame3E` | Uses `SlmpCompatibilityMode::Legacy`; `X` and `Y` string addresses use iQ-F octal notation. |
| `SlmpPlcProfile::IqR` | `melsec:iq-r` | MELSEC iQ-R | `SlmpFrameType::Frame4E` | Uses `SlmpCompatibilityMode::Iqr`. |
| `SlmpPlcProfile::IqL` | `melsec:iq-l` | MELSEC iQ-L | `SlmpFrameType::Frame4E` | Uses `SlmpCompatibilityMode::Iqr`; keeps its own address/range profile with iQ-R-equivalent rules. |
| `SlmpPlcProfile::MxF` | `melsec:mx-f` | MELSEC MX-F | `SlmpFrameType::Frame4E` | Uses `SlmpCompatibilityMode::Iqr`. |
| `SlmpPlcProfile::MxR` | `melsec:mx-r` | MELSEC MX-R | `SlmpFrameType::Frame4E` | Uses `SlmpCompatibilityMode::Iqr`. |
| `SlmpPlcProfile::QCpu` | `melsec:qcpu` | MELSEC QCPU | `SlmpFrameType::Frame3E` | Uses `SlmpCompatibilityMode::Legacy`. |
| `SlmpPlcProfile::LCpu` | `melsec:lcpu` | MELSEC LCPU | `SlmpFrameType::Frame3E` | Uses `SlmpCompatibilityMode::Legacy`. |
| `SlmpPlcProfile::QnU` | `melsec:qnu` | MELSEC QnU | `SlmpFrameType::Frame3E` | Uses `SlmpCompatibilityMode::Legacy`. |
| `SlmpPlcProfile::QnUDV` | `melsec:qnudv` | MELSEC QnUDV | `SlmpFrameType::Frame3E` | Uses `SlmpCompatibilityMode::Legacy`. |

## Supported device types

This is a quick map of common families. See [supported registers](docs/SUPPORTED_REGISTERS.md) for the full helper-layer reference.

| Family | Examples | Default value |
| --- | --- | --- |
| Internal and latch relays | `M100`, `L100`, `F100`, `V100` | `SlmpValue::Bool` |
| Physical I/O | `X10`, `Y10`, `DX10`, `DY10` | `SlmpValue::Bool` |
| Link relays | `B10`, `SB10` | `SlmpValue::Bool` |
| Data registers | `D100`, `R50`, `ZR0`, `RD0` | `SlmpValue::U16` |
| Link word registers | `W10`, `SW10` | `SlmpValue::U16` |
| Timers | `TS10`, `TC10`, `TN10`, `STS10`, `STC10`, `STN10` | bits or words by device code |
| Counters | `CS10`, `CC10`, `CN10` | bits or words by device code |
| Long families | `LTN10:D`, `LSTN10:D`, `LCN10:D`, `LZ0:D` | `SlmpValue::U32` |

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

## Documentation links

| Page | Link |
| --- | --- |
| Getting started | [docs/GETTING_STARTED.md](docs/GETTING_STARTED.md) |
| Usage guide | [docs/USAGE_GUIDE.md](docs/USAGE_GUIDE.md) |
| Supported registers | [docs/SUPPORTED_REGISTERS.md](docs/SUPPORTED_REGISTERS.md) |
| PLC profiles | [docs/PROFILES.md](docs/PROFILES.md) |
| Examples | [examples/README.md](examples/README.md) |
| Full documentation site | [plc-comm-docs-site](https://github.com/fa-yoshinobu/plc-comm-docs-site) |

## Hardware verified

The repository includes live validation records under `docs/`.

| PLC | Canonical profile | API profile | Path | Validation record |
| --- | --- | --- | --- | --- |
| Mitsubishi iQ-F / FX5UC-32MT/D | `melsec:iq-f` | `SlmpPlcProfile::IqF` | TCP `1025`, UDP `1027` | [docs/IQF_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md](docs/IQF_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md) |
| Mitsubishi iQ-R / R08CPU | `melsec:iq-r` | `SlmpPlcProfile::IqR` | TCP `1025`, UDP `1027` | [docs/IQR_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md](docs/IQR_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md) |
| Mitsubishi iQ-L / L16HCPU | `melsec:iq-l` | `SlmpPlcProfile::IqL` | TCP `1025`, UDP `1027` | [docs/IQL_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md](docs/IQL_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md) |
| Mitsubishi LCPU | `melsec:lcpu` | `SlmpPlcProfile::LCpu` | TCP `1025`, UDP `1027` | [docs/LCPU_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md](docs/LCPU_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md) |
| Mitsubishi Q12HCPU | `melsec:qcpu` | `SlmpPlcProfile::QCpu` | TCP `1025` | [docs/QCPU_RUNTIME_RANGE_VALIDATION_2026-05-15.md](docs/QCPU_RUNTIME_RANGE_VALIDATION_2026-05-15.md) |
| Mitsubishi Q26UDEHCPU | `melsec:qnu` | `SlmpPlcProfile::QnU` | TCP `1025` | [docs/QNU_RUNTIME_RANGE_VALIDATION_2026-05-15.md](docs/QNU_RUNTIME_RANGE_VALIDATION_2026-05-15.md) |
| Mitsubishi Q06UDVCPU | `melsec:qnudv` | `SlmpPlcProfile::QnUDV` | TCP `1025` | [docs/QNUDV_RUNTIME_RANGE_VALIDATION_2026-05-15.md](docs/QNUDV_RUNTIME_RANGE_VALIDATION_2026-05-15.md) |

## License and registry

Distributed under the MIT License.

Package registry: [crates.io/crates/plc-comm-slmp-rust](https://crates.io/crates/plc-comm-slmp-rust)
