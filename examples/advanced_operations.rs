mod common;

use common::{connect_from_env, env_csv, env_string, print_connection_banner};
use plc_comm_slmp::{SlmpAddress, SlmpBlockRead, SlmpExtensionSpec, parse_qualified_device};
use std::error::Error;

fn parse_word_devices(
    key: &str,
    default: &str,
) -> Result<Vec<plc_comm_slmp::SlmpDeviceAddress>, Box<dyn Error>> {
    env_csv(key, default)
        .into_iter()
        .map(|device| Ok(SlmpAddress::parse(&device)?))
        .collect()
}

fn build_extension_spec(device: &plc_comm_slmp::SlmpQualifiedDeviceAddress) -> SlmpExtensionSpec {
    SlmpExtensionSpec {
        extension_specification: device.extension_specification.unwrap_or(0),
        direct_memory_specification: device.direct_memory_specification.unwrap_or(0),
        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("advanced_operations");
    let client = connect_from_env().await?;

    let type_name = client.read_type_name().await?;
    println!(
        "type name -> model={} code=0x{:04X} has_model_code={}",
        type_name.model, type_name.model_code, type_name.has_model_code
    );

    let word_devices = parse_word_devices("SLMP_RANDOM_WORDS", "D100,R10")?;
    let dword_devices = parse_word_devices("SLMP_RANDOM_DWORDS", "D200,LTN10")?;
    let random = client.read_random(&word_devices, &dword_devices).await?;
    println!("random word values   -> {:?}", random.word_values);
    println!("random dword values  -> {:?}", random.dword_values);

    let word_block = SlmpBlockRead {
        device: SlmpAddress::parse(&env_string("SLMP_BLOCK_WORD_DEVICE", "D100"))?,
        points: env_string("SLMP_BLOCK_WORD_POINTS", "4").parse()?,
    };
    let bit_block = SlmpBlockRead {
        device: SlmpAddress::parse(&env_string("SLMP_BLOCK_BIT_DEVICE", "M10"))?,
        points: env_string("SLMP_BLOCK_BIT_POINTS", "8").parse()?,
    };
    let block = client.read_block(&[word_block], &[bit_block]).await?;
    println!("block word values    -> {:?}", block.word_values);
    println!("block packed bits    -> {:?}", block.bit_values);

    let qualified = parse_qualified_device(&env_string("SLMP_EXT_DEVICE", "J1\\W10"))?;
    let extension = build_extension_spec(&qualified);
    let ext_values = client
        .read_words_extended(
            qualified,
            env_string("SLMP_EXT_POINTS", "1").parse()?,
            extension,
        )
        .await?;
    println!("extended read        -> {:?}", ext_values);

    let loopback = client.self_test_loopback(b"rust-slmp").await?;
    println!(
        "self-test loopback   -> {:?}",
        String::from_utf8_lossy(&loopback)
    );
    Ok(())
}
