# PLC profiles

The canonical profile is the stable configuration value for PLC selection.
`SlmpPlcProfile` is the Rust API selector that binds that profile to the SLMP
frame and compatibility mode used by the client.
Use `SlmpPlcProfile::display_name()` for UI labels. Store the canonical profile
string, not the display name.

For cross-profile capability and device-range details, see the [SLMP Profile Reference](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/).

## Profiles table

| Canonical profile | Display name | Rust selector | Frame | Mode | Notes |
| --- | --- | --- | --- | --- | --- |
| `melsec:iq-f` | MELSEC iQ-F (built-in) | `SlmpPlcProfile::IqF` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | iQ-F address rules; `DX` and `DY` are rejected. |
| `melsec:iq-r` | MELSEC iQ-R (built-in) | `SlmpPlcProfile::IqR` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:iq-r:rj71en71` | MELSEC iQ-R (RJ71EN71) | `SlmpPlcProfile::IqRRj71En71` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Ethernet-unit profile using iQ-R protocol mode. |
| `melsec:iq-l` | MELSEC iQ-L (built-in) | `SlmpPlcProfile::IqL` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R-equivalent address parsing while remaining separately selectable. |
| `melsec:mx-f` | MELSEC MX-F (built-in) | `SlmpPlcProfile::MxF` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:mx-r` | MELSEC MX-R (built-in) | `SlmpPlcProfile::MxR` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Iqr` | Uses iQ-R protocol mode. |
| `melsec:lcpu` | MELSEC-L (built-in) | `SlmpPlcProfile::LCpu` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | Legacy 3E profile. |
| `melsec:lcpu:lj71e71-100` | MELSEC-L (LJ71E71-100) | `SlmpPlcProfile::LCpuLj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile using 4E frames with Q/L compatibility. |
| `melsec:qcpu:qj71e71-100` | MELSEC-Q (QJ71E71-100) | `SlmpPlcProfile::QCpuQj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile for QCPU connections. |
| `melsec:qnu` | MELSEC QnU (built-in) | `SlmpPlcProfile::QnU` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | QnU profile. Use direct or random device commands for normal access. |
| `melsec:qnu:qj71e71-100` | MELSEC QnU (QJ71E71-100) | `SlmpPlcProfile::QnUQj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile using 4E frames with Q/L compatibility. |
| `melsec:qnudv` | MELSEC QnUDV (built-in) | `SlmpPlcProfile::QnUDV` | `SlmpFrameType::Frame3E` | `SlmpCompatibilityMode::Legacy` | QnUDV profile. Use direct or random device commands for normal access. |
| `melsec:qnudv:qj71e71-100` | MELSEC QnUDV (QJ71E71-100) | `SlmpPlcProfile::QnUDVQj71E71100` | `SlmpFrameType::Frame4E` | `SlmpCompatibilityMode::Legacy` | Ethernet unit profile using 4E frames with Q/L compatibility. |

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
