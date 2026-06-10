#![allow(dead_code)]

use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcFamily, SlmpTargetAddress, SlmpTransportMode,
    parse_named_target, parse_target_auto_number,
};
use std::env;
use std::error::Error;

pub fn env_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn env_bool(key: &str) -> bool {
    matches!(
        env::var(key).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("True") | Ok("yes") | Ok("YES")
    )
}

pub fn env_csv(key: &str, default: &str) -> Vec<String> {
    env_string(key, default)
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

pub fn options_from_env() -> Result<SlmpConnectionOptions, Box<dyn Error>> {
    let host = env_string("SLMP_HOST", "127.0.0.1");
    let family = parse_plc_family(&env_string("SLMP_PLC_FAMILY", "iq-r"))?;
    let mut options = SlmpConnectionOptions::new(host, family);
    options.port = env_string("SLMP_PORT", "1025").parse()?;
    options.transport_mode = match env_string("SLMP_TRANSPORT", "tcp")
        .to_ascii_lowercase()
        .as_str()
    {
        "udp" => SlmpTransportMode::Udp,
        _ => SlmpTransportMode::Tcp,
    };
    options.monitoring_timer = env_string("SLMP_MONITORING_TIMER", "16").parse()?;
    options.timeout =
        std::time::Duration::from_millis(env_string("SLMP_TIMEOUT_MS", "3000").parse()?);
    let has_network_station_target =
        env::var("SLMP_NETWORK").is_ok() || env::var("SLMP_STATION").is_ok();
    if has_network_station_target {
        if env::var("SLMP_TARGET").is_ok() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Use either SLMP_TARGET or SLMP_NETWORK/SLMP_STATION, not both.",
            )
            .into());
        }
        let network = env_string("SLMP_NETWORK", "");
        let station = env_string("SLMP_STATION", "");
        if network.is_empty() || station.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SLMP_NETWORK and SLMP_STATION must be specified together.",
            )
            .into());
        }
        options.target = SlmpTargetAddress {
            network: parse_target_auto_number(&network)? as u8,
            station: parse_target_auto_number(&station)? as u8,
            module_io: parse_target_auto_number(&env_string("SLMP_MODULE_IO", "0x03FF"))? as u16,
            multidrop: parse_target_auto_number(&env_string("SLMP_MULTIDROP", "0x00"))? as u8,
        };
    } else if let Ok(target) = env::var("SLMP_TARGET") {
        options.target = parse_named_target(&target)?.target;
    }
    Ok(options)
}

pub async fn connect_from_env() -> Result<SlmpClient, Box<dyn Error>> {
    Ok(SlmpClient::connect(options_from_env()?).await?)
}

pub fn print_connection_banner(example: &str) {
    let family = env_string("SLMP_PLC_FAMILY", "iq-r");
    let profile = SlmpPlcFamily::parse_label(&family).map(SlmpPlcFamily::defaults);
    println!(
        "{example}: host={} port={} plc_family={} frame={} compatibility={} transport={} target={}",
        env_string("SLMP_HOST", "127.0.0.1"),
        env_string("SLMP_PORT", "1025"),
        family,
        profile
            .map(|profile| format!("{:?}", profile.frame_type))
            .unwrap_or_else(|| "unknown".to_string()),
        profile
            .map(|profile| format!("{:?}", profile.compatibility_mode))
            .unwrap_or_else(|| "unknown".to_string()),
        env_string("SLMP_TRANSPORT", "tcp"),
        format_env_target()
    );
}

fn format_env_target() -> String {
    match (env::var("SLMP_NETWORK"), env::var("SLMP_STATION")) {
        (Ok(network), Ok(station)) => match (
            parse_target_auto_number(&network),
            parse_target_auto_number(&station),
            parse_target_auto_number(&env_string("SLMP_MODULE_IO", "0x03FF")),
            parse_target_auto_number(&env_string("SLMP_MULTIDROP", "0x00")),
        ) {
            (Ok(network), Ok(station), Ok(module_io), Ok(multidrop)) => {
                format!(
                    "network={network} station={station} module_io=0x{module_io:04X} multidrop=0x{multidrop:02X}"
                )
            }
            _ => format!("network={network} station={station}"),
        },
        _ => env::var("SLMP_TARGET").unwrap_or_else(|_| "default".to_string()),
    }
}

fn parse_plc_family(value: &str) -> Result<SlmpPlcFamily, Box<dyn Error>> {
    SlmpPlcFamily::parse_label(value).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "SLMP_PLC_FAMILY is required. Use iq-f, iq-r, iq-l, mx-f, mx-r, qcpu, lcpu, qnu, or qnudv."
                .to_string(),
        )
        .into()
    })
}
