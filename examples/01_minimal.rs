//! Minimal high-level read example for `192.168.250.100:1025`.

use plc_comm_slmp::{SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, read_typed};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new(
        "192.168.250.100",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )?;
    let client = SlmpClient::connect(options).await?;
    let value = read_typed(
        &client,
        SlmpAddress::parse("D100", plc_comm_slmp::SlmpPlcProfile::IqR)?,
        "U",
    )
    .await?;
    println!("{:?}", value);
    client.close().await?;

    Ok(())
}
