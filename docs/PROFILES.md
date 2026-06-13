# PLC profiles

`SlmpPlcProfile` is the public selector that binds a PLC family to the SLMP frame and compatibility mode used by the client.

## Profiles table

| `SlmpPlcProfile` variant | Hardware | Frame | Mode | Notes |
| --- | --- | --- | --- | --- |
| `SlmpPlcProfile::IqF` | MELSEC iQ-F | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | iQ-F address rules; `DX` and `DY` are rejected. |
| `SlmpPlcProfile::IqR` | MELSEC iQ-R | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `SlmpPlcProfile::IqL` | MELSEC iQ-L | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R address parsing rules. |
| `SlmpPlcProfile::MxF` | MELSEC MX-F | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `SlmpPlcProfile::MxR` | MELSEC MX-R | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `SlmpPlcProfile::QCpu` | MELSEC QCPU | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `SlmpPlcProfile::LCpu` | MELSEC LCPU | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `SlmpPlcProfile::QnU` | MELSEC QnU | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `SlmpPlcProfile::QnUDV` | MELSEC QnUDV | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |

## How to select

```rust
let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
options.port = 1025;
let client = SlmpClient::connect(options).await?;
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
