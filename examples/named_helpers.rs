//! High-level named read/write and polling example.

mod common;

use common::{connect_from_env, env_bool, env_csv, env_string, print_connection_banner};
use futures_util::StreamExt;
use plc_comm_slmp::{NamedAddress, SlmpValue, poll_named, read_named, write_named};
use std::error::Error;
use std::time::Duration;

fn print_collection(label: &str, values: &NamedAddress) {
    println!("{label}");
    for (address, value) in values {
        println!("  {address:<12} {value:?}");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("named_helpers")?;
    let client = connect_from_env().await?;

    // Named reads let you collect mixed word, float, bit-in-word, and long-family values.
    let addresses = env_csv(
        "SLMP_NAMED_ADDRESSES",
        "D100:U,D200:F,D50.3,LTN10:D,LTS10:BIT",
    );
    let values = read_named(&client, &addresses).await?;
    print_collection("named typed collection", &values);

    if env_bool("SLMP_ENABLE_WRITES") {
        let mut updates = NamedAddress::new();
        updates.insert(
            env_string("SLMP_NAMED_WRITE_WORD", "D700:U"),
            SlmpValue::U16(env_string("SLMP_NAMED_WRITE_WORD_VALUE", "42").parse()?),
        );
        updates.insert(
            env_string("SLMP_NAMED_WRITE_FLOAT", "D702:F"),
            SlmpValue::F32(env_string("SLMP_NAMED_WRITE_FLOAT_VALUE", "3.14").parse()?),
        );
        write_named(&client, &updates).await?;
        println!("write_named -> {updates:?}");
    } else {
        println!("set SLMP_ENABLE_WRITES=1 to run the write_named part of this example");
    }

    // Polling reuses the compiled address plan after the first tick.
    let interval_ms: u64 = env_string("SLMP_POLL_INTERVAL_MS", "1000").parse()?;
    let mut stream = Box::pin(poll_named(
        &client,
        &addresses,
        Duration::from_millis(interval_ms),
    ));
    if let Some(values) = stream.next().await.transpose()? {
        print_collection("poll_named first tick", &values);
    }
    Ok(())
}
