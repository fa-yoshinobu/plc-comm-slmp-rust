mod common;

use common::{env_csv, env_string, options_from_env, print_connection_banner};
use plc_comm_slmp::{SlmpClient, SlmpRouteValidationOptions, run_route_validation_compare};
use std::error::Error;

fn make_error(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::other(message.into()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("route_validation_compare");
    let options = options_from_env()?;
    let client = SlmpClient::connect(options).await?;
    let report = run_route_validation_compare(
        &client,
        SlmpRouteValidationOptions {
            word_device: env_string("SLMP_ROUTE_WORD_DEVICE", "D9000"),
            dword_device: env_string("SLMP_ROUTE_DWORD_DEVICE", "D9002"),
            float_device: env_string("SLMP_ROUTE_FLOAT_DEVICE", "D9004"),
            bit_device: env_string("SLMP_ROUTE_BIT_DEVICE", "M100"),
            lz_device: env_string("SLMP_ROUTE_LZ_DEVICE", "LZ0"),
            range_family: None,
            range_error_devices: env_csv("SLMP_ROUTE_RANGE_DEVICES", "X,Y,M,D,R,ZR,RD,LZ,SM,SD"),
        },
    )
    .await?;

    println!("model -> {} family={:?}", report.model, report.family);
    for case in &report.cases {
        println!(
            "{:?} {} {}: {}",
            case.status, case.route, case.name, case.detail
        );
    }
    println!(
        "summary -> passed={} failed={} warned={} skipped={}",
        report.summary.passed, report.summary.failed, report.summary.warned, report.summary.skipped
    );

    if !report.is_success() {
        return Err(make_error("one or more route validation checks failed"));
    }

    Ok(())
}
