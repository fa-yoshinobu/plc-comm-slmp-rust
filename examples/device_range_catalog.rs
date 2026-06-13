//! Print the live device-range catalog for the configured PLC profile.

mod common;

use common::{options_from_env, print_connection_banner};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("device_range_catalog");
    let options = options_from_env()?;
    let client = plc_comm_slmp::SlmpClient::connect(options).await?;
    let catalog = client.read_device_range_catalog().await?;
    println!(
        "catalog -> model={} code=0x{:04X} plc_profile={:?}",
        catalog.model, catalog.model_code, catalog.plc_profile
    );

    // Each entry includes support state, notation, live point count, and display range.
    for entry in &catalog.entries {
        println!(
            "{} supported={} notation={:?} points={} range={}",
            entry.device,
            entry.supported,
            entry.notation,
            entry
                .point_count
                .map(|value| value.to_string())
                .unwrap_or_else(|| "open".to_string()),
            entry
                .address_range
                .clone()
                .unwrap_or_else(|| "-".to_string())
        );
    }

    Ok(())
}
