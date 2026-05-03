mod common;

use common::{env_csv, env_string, options_from_env, print_connection_banner};
use plc_comm_slmp::{SlmpClient, SlmpDeviceRangeSampleOptions, run_device_range_sample_compare};
use std::error::Error;

fn make_error(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::other(message.into()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("device_range_sample_compare");
    let sample_points = env_string("SLMP_SAMPLE_POINTS", "10").parse::<usize>()?;
    let only = env_csv("SLMP_SAMPLE_ONLY", "");
    let options = options_from_env()?;
    let client = SlmpClient::connect(options).await?;
    let report = run_device_range_sample_compare(
        &client,
        SlmpDeviceRangeSampleOptions {
            sample_points,
            only,
        },
    )
    .await?;

    println!(
        "catalog -> model={} family={:?} sample_points={}",
        report.model, report.family, report.sample_points
    );

    for device in &report.devices {
        if let Some(reason) = &device.untested_reason {
            println!("UNTESTED {}: {reason}", device.device);
            continue;
        }

        println!(
            "DEVICE {} range={} kind={:?} samples={} bit_blocks={}",
            device.device,
            device.address_range.as_deref().unwrap_or("n/a"),
            device.value_kind,
            device.sample_addresses.len(),
            device.bit_block_addresses.len()
        );
        for address in &device.sample_addresses {
            println!("SAMPLE {address}");
        }
        for address in &device.bit_block_addresses {
            println!("BIT-BLOCK {address}");
        }
        for failure in &device.failures {
            println!(
                "FAIL {} {}: {}",
                failure.phase, failure.address, failure.message
            );
        }
    }

    println!(
        "summary -> passed={} read_failed={} write_failed={} readback_failed={} restore_failed={} skipped={} unsupported={} bit_blocks_passed={} bit_blocks_failed={}",
        report.summary.passed,
        report.summary.read_failed,
        report.summary.write_failed,
        report.summary.readback_failed,
        report.summary.restore_failed,
        report.summary.skipped,
        report.summary.unsupported,
        report.summary.bit_blocks_passed,
        report.summary.bit_blocks_failed
    );

    if report.summary.restore_failed > 0 {
        return Err(make_error("one or more restore operations failed"));
    }
    if !report.is_success() {
        return Err(make_error("one or more device sample checks failed"));
    }

    Ok(())
}
