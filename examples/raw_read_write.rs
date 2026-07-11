//! Minimal raw word read example with an optional write/read-back section.

mod common;

use common::{connect_from_env, env_bool, env_csv, env_string, print_connection_banner};
use plc_comm_slmp::SlmpAddress;
use std::error::Error;

fn parse_u16_csv(key: &str, default: &str) -> Result<Vec<u16>, Box<dyn Error>> {
    env_csv(key, default)
        .into_iter()
        .map(|value| value.parse::<u16>().map_err(Into::into))
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("raw_read_write")?;
    let client = connect_from_env().await?;
    let plc_profile = client.plc_profile().await;

    // Start with a plain data-register read before trying any writes.
    let read_address = env_string("SLMP_READ_ADDRESS", "D100");
    let read_count: u16 = env_string("SLMP_READ_COUNT", "2").parse()?;
    let words = client
        .read_words_raw(SlmpAddress::parse(&read_address, plc_profile)?, read_count)
        .await?;
    println!("read {read_address} count={read_count} -> {words:?}");

    if !env_bool("SLMP_ENABLE_WRITES") {
        println!("set SLMP_ENABLE_WRITES=1 to run the write/read-back part of this example");
        return Ok(());
    }

    // Writes are opt-in so the example is read-only by default.
    let write_address = env_string("SLMP_WRITE_ADDRESS", "D600");
    let write_values = parse_u16_csv("SLMP_WRITE_VALUES", "111,222")?;
    client
        .write_words(
            SlmpAddress::parse(&write_address, plc_profile)?,
            &write_values,
        )
        .await?;
    let read_back = client
        .read_words_raw(
            SlmpAddress::parse(&write_address, plc_profile)?,
            write_values.len() as u16,
        )
        .await?;
    println!("write {write_address} values={write_values:?}");
    println!("read-back {write_address} -> {read_back:?}");
    Ok(())
}
