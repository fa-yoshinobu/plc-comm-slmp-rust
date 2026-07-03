# PLC profiles

The canonical profile is the stable configuration value for PLC selection.
`SlmpPlcProfile` is the Rust API selector that binds that profile to the SLMP
frame and compatibility mode used by the client.

## Profiles table

| Canonical profile | Human label | Rust selector | Frame | Mode | Notes |
| --- | --- | --- | --- | --- | --- |
| `melsec:iq-f` | MELSEC iQ-F | `SlmpPlcProfile::IqF` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | iQ-F address rules; `DX` and `DY` are rejected. |
| `melsec:iq-r` | MELSEC iQ-R | `SlmpPlcProfile::IqR` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:iq-l` | MELSEC iQ-L | `SlmpPlcProfile::IqL` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R-equivalent address parsing while remaining separately selectable. |
| `melsec:mx-f` | MELSEC MX-F | `SlmpPlcProfile::MxF` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:mx-r` | MELSEC MX-R | `SlmpPlcProfile::MxR` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:qcpu` | MELSEC QCPU | `SlmpPlcProfile::QCpu` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Q CPU profile. Strict profile rejects unavailable block routes; use direct or random device commands. |
| `melsec:lcpu` | MELSEC LCPU | `SlmpPlcProfile::LCpu` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `melsec:qnu` | MELSEC QnU | `SlmpPlcProfile::QnU` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | QnU profile. Strict profile rejects unavailable block routes; use direct or random device commands. |
| `melsec:qnudv` | MELSEC QnUDV | `SlmpPlcProfile::QnUDV` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | QnUDV profile. Strict profile rejects unavailable type-name and block routes. |

## How to select

```rust
use plc_comm_slmp::{SlmpClient, SlmpConnectionOptions, SlmpPlcProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    println!("{}", client.plc_profile().await.canonical_name());
    client.close().await?;

    Ok(())
}
```

Text configuration should store the canonical profile exactly:

```rust
use plc_comm_slmp::SlmpPlcProfile;

let profile = SlmpPlcProfile::parse_label("melsec:iq-r")
    .expect("unknown canonical PLC profile");
assert_eq!(profile, SlmpPlcProfile::IqR);
```

## Profile-specific cautions

| Canonical profile | Human label | Caution |
| --- | --- | --- |
| `melsec:iq-f` | MELSEC iQ-F | Frame 3E, Legacy mode. `DX` and `DY` are not valid. Do not use Frame 4E. |
| `melsec:iq-r` | MELSEC iQ-R | Frame 4E, iQR mode. |
| `melsec:iq-l` | MELSEC iQ-L | Frame 4E, iQR mode. |
| `melsec:mx-f` | MELSEC MX-F | Frame 4E, iQR mode. |
| `melsec:mx-r` | MELSEC MX-R | Frame 4E, iQR mode. |
| `melsec:qcpu` | MELSEC QCPU | Frame 3E, Legacy mode. Strict profile rejects block commands `0x0406` / `0x1406`. |
| `melsec:lcpu` | MELSEC LCPU | Frame 3E, Legacy mode. |
| `melsec:qnu` | MELSEC QnU | Frame 3E, Legacy mode. Strict profile rejects block commands `0x0406` / `0x1406`. |
| `melsec:qnudv` | MELSEC QnUDV | Frame 3E, Legacy mode. Strict profile rejects Read Type Name (`0x0101`) and block commands `0x0406` / `0x1406`; disabling strict profile sends them and lets the PLC respond. |
