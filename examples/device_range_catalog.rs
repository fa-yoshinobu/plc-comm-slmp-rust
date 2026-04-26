mod common;

use common::{env_string, options_from_env, print_connection_banner};
use plc_comm_slmp::SlmpDeviceRangeFamily;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("device_range_catalog");
    let options = options_from_env()?;
    let plc_type = parse_plc_type(&env_string("SLMP_PLC_TYPE", ""))?;
    let client = plc_comm_slmp::SlmpClient::connect(options).await?;
    let catalog = client
        .read_device_range_catalog_for_family(plc_type)
        .await?;
    println!(
        "catalog -> selected_family={:?} model={} code=0x{:04X} family={:?}",
        plc_type, catalog.model, catalog.model_code, catalog.family
    );

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

fn parse_plc_type(value: &str) -> Result<SlmpDeviceRangeFamily, std::io::Error> {
    let normalized = value.trim().to_ascii_lowercase().replace(['-', '_'], "");
    match normalized.as_str() {
        "iqr" => Ok(SlmpDeviceRangeFamily::IqR),
        "mxf" => Ok(SlmpDeviceRangeFamily::MxF),
        "mxr" => Ok(SlmpDeviceRangeFamily::MxR),
        "iqf" => Ok(SlmpDeviceRangeFamily::IqF),
        "qcpu" | "q" => Ok(SlmpDeviceRangeFamily::QCpu),
        "lcpu" | "l" => Ok(SlmpDeviceRangeFamily::LCpu),
        "qnu" => Ok(SlmpDeviceRangeFamily::QnU),
        "qnudv" | "qnudvcpu" => Ok(SlmpDeviceRangeFamily::QnUDV),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "SLMP_PLC_TYPE is required. Use iq-r, mx-f, mx-r, iq-f, qcpu, lcpu, qnu, or qnudv.",
        )),
    }
}
