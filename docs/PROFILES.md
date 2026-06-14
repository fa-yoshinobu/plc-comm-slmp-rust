# PLC profiles

`SlmpPlcProfile` is the public selector that binds a PLC family to the SLMP frame and compatibility mode used by the client.

## Profiles table

| `SlmpPlcProfile` variant | Canonical string | Hardware | Frame | Mode | Notes |
| --- | --- | --- | --- | --- | --- |
| `SlmpPlcProfile::IqF` | `melsec:iq-f` | MELSEC iQ-F | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | iQ-F address rules; `DX` and `DY` are rejected. |
| `SlmpPlcProfile::IqR` | `melsec:iq-r` | MELSEC iQ-R | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `SlmpPlcProfile::IqL` | `melsec:iq-l` | MELSEC iQ-L | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R-equivalent address parsing while remaining separately selectable. |
| `SlmpPlcProfile::MxF` | `melsec:mx-f` | MELSEC MX-F | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `SlmpPlcProfile::MxR` | `melsec:mx-r` | MELSEC MX-R | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `SlmpPlcProfile::QCpu` | `melsec:qcpu` | MELSEC QCPU | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `SlmpPlcProfile::LCpu` | `melsec:lcpu` | MELSEC LCPU | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `SlmpPlcProfile::QnU` | `melsec:qnu` | MELSEC QnU | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `SlmpPlcProfile::QnUDV` | `melsec:qnudv` | MELSEC QnUDV | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |

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

## Profile-specific cautions

| Profile | Caution |
| --- | --- |
| `SlmpPlcProfile::IqF` | Frame 3E, Legacy mode. `DX` and `DY` are not valid. Do not use Frame 4E. |
| `SlmpPlcProfile::IqR` | Frame 4E, iQR mode. |
| `SlmpPlcProfile::IqL` | Frame 4E, iQR mode. |
| `SlmpPlcProfile::MxF` | Frame 4E, iQR mode. |
| `SlmpPlcProfile::MxR` | Frame 4E, iQR mode. |
| `SlmpPlcProfile::QCpu` | Frame 3E, Legacy mode. |
| `SlmpPlcProfile::LCpu` | Frame 3E, Legacy mode. |
| `SlmpPlcProfile::QnU` | Frame 3E, Legacy mode. |
| `SlmpPlcProfile::QnUDV` | Frame 3E, Legacy mode. |
