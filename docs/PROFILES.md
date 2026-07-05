# PLC profiles

The canonical profile is the stable configuration value for PLC selection.
`SlmpPlcProfile` is the Rust API selector that binds that profile to the SLMP
frame and compatibility mode used by the client.

For cross-profile capability and device-range details, see the [SLMP Profile Reference](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/).

## Profiles table

| Canonical profile | Human label | Rust selector | Frame | Mode | Notes |
| --- | --- | --- | --- | --- | --- |
| `melsec:iq-f` | MELSEC iQ-F | `SlmpPlcProfile::IqF` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | iQ-F address rules; `DX` and `DY` are rejected. |
| `melsec:iq-r` | MELSEC iQ-R | `SlmpPlcProfile::IqR` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:iq-r:rj71en71` | MELSEC iQ-R via RJ71EN71 | `SlmpPlcProfile::IqRRj71En71` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Ethernet-unit profile using iQ-R protocol mode. |
| `melsec:iq-l` | MELSEC iQ-L | `SlmpPlcProfile::IqL` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R-equivalent address parsing while remaining separately selectable. |
| `melsec:mx-f` | MELSEC MX-F | `SlmpPlcProfile::MxF` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:mx-r` | MELSEC MX-R | `SlmpPlcProfile::MxR` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:lcpu` | MELSEC LCPU | `SlmpPlcProfile::LCpu` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `melsec:lcpu:lj71e71-100` | MELSEC LCPU via LJ71E71-100 | `SlmpPlcProfile::LCpuLj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile using 4E frames with Q/L compatibility. |
| `melsec:qcpu:qj71e71-100` | MELSEC QCPU via QJ71E71-100 | `SlmpPlcProfile::QCpuQj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile for QCPU connections. |
| `melsec:qnu` | MELSEC QnU | `SlmpPlcProfile::QnU` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | QnU profile. Use direct or random device commands for normal access. |
| `melsec:qnu:qj71e71-100` | MELSEC QnU via QJ71E71-100 | `SlmpPlcProfile::QnUQj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile using 4E frames with Q/L compatibility. |
| `melsec:qnudv` | MELSEC QnUDV | `SlmpPlcProfile::QnUDV` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | QnUDV profile. Use direct or random device commands for normal access. |
| `melsec:qnudv:qj71e71-100` | MELSEC QnUDV via QJ71E71-100 | `SlmpPlcProfile::QnUDVQj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile using 4E frames with Q/L compatibility. |

`melsec:qcpu` is base-only and used only for inherited address and range rules.
Connection options reject it and point callers to `melsec:qcpu:qj71e71-100`.

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
