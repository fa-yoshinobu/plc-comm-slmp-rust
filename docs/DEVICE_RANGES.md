# Device Range Catalog

Device range rule definitions are maintained in the shared SLMP profile comparison, not in this Rust library documentation.

See the [SLMP profile comparison](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/profile-comparison/) for cross-profile capability and device-range details.

This page only describes the Rust API surface.

## Purpose

`read_device_range_catalog()` reads live device range bounds after you connect with an explicit `SlmpPlcProfile`.

The catalog is a connected diagnostics and application-layer validation aid. Normal read/write APIs do not use device-range upper bounds to decide whether an address can be sent. Applications that need PLC-specific range validation should read the catalog and apply that policy outside the transport operation.

## Profile Policy

The client does not infer `SlmpPlcProfile` from `ReadTypeName`, model text, or model code. Choose the profile in your application, configuration UI, or operator workflow.

## API

`SlmpClient` exposes:

```text
pub async fn read_device_range_catalog(&self) -> Result<SlmpDeviceRangeCatalog, SlmpError>
```

Example:

```rust
use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqF);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let catalog = client.read_device_range_catalog().await?;

    println!("selected={} -> {:?}", catalog.model, catalog.plc_profile);
    for entry in catalog.entries.iter().filter(|x| x.supported) {
        println!(
            "{}: points={:?}, range={:?}, source={}",
            entry.device, entry.point_count, entry.address_range, entry.source
        );
    }

    client.close().await?;
    Ok(())
}
```

Returned types:

- `SlmpDeviceRangeCatalog`
- `SlmpDeviceRangeEntry`
- `SlmpPlcProfile`
- `SlmpDeviceRangeCategory`
- `SlmpDeviceRangeNotation`

`SlmpDeviceRangeEntry.notation` uses `Decimal`, `Octal`, or `Hexadecimal` for the public address text this crate expects.
