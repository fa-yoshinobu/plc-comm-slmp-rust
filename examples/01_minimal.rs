//! Minimal high-level read example for `192.168.250.100:1025`.

use plc_comm_slmp::{SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, read_typed};

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
