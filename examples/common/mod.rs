#![allow(dead_code)]

use plc_comm_slmp::{
    SlmpClient, SlmpCompatibilityMode, SlmpConnectionOptions, SlmpFrameType, SlmpTransportMode,
    parse_named_target,
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
    let mut options = SlmpConnectionOptions::new(host);
    options.port = env_string("SLMP_PORT", "1025").parse()?;
    options.frame_type = match env_string("SLMP_FRAME", "4e").to_ascii_lowercase().as_str() {
        "3e" => SlmpFrameType::Frame3E,
        _ => SlmpFrameType::Frame4E,
    };
    options.compatibility_mode = match env_string("SLMP_SERIES", "iqr")
        .to_ascii_lowercase()
        .as_str()
    {
        "legacy" | "ql" => SlmpCompatibilityMode::Legacy,
        _ => SlmpCompatibilityMode::Iqr,
    };
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
    if let Ok(target) = env::var("SLMP_TARGET") {
        options.target = parse_named_target(&target)?.target;
    }
    Ok(options)
}

pub async fn connect_from_env() -> Result<SlmpClient, Box<dyn Error>> {
    Ok(SlmpClient::connect(options_from_env()?).await?)
}

pub fn print_connection_banner(example: &str) {
    println!(
        "{example}: host={} port={} frame={} series={} transport={} target={}",
        env_string("SLMP_HOST", "127.0.0.1"),
        env_string("SLMP_PORT", "1025"),
        env_string("SLMP_FRAME", "4e"),
        env_string("SLMP_SERIES", "iqr"),
        env_string("SLMP_TRANSPORT", "tcp"),
        env::var("SLMP_TARGET").unwrap_or_else(|_| "default".to_string())
    );
}
