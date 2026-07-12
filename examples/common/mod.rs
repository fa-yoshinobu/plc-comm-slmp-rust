#![allow(dead_code)]

use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpTargetAddress, SlmpTransportMode,
    parse_named_target,
};
use std::env;
use std::error::Error;

const PLC_PROFILE_REQUIRED_MESSAGE: &str = "SLMP_PLC_PROFILE is required. Use melsec:iq-f, melsec:iq-r, melsec:iq-r:rj71en71, melsec:iq-l, melsec:mx-f, melsec:mx-r, melsec:qcpu:qj71e71-100, melsec:lcpu, melsec:lcpu:lj71e71-100, melsec:qnu, melsec:qnu:qj71e71-100, melsec:qnudv, or melsec:qnudv:qj71e71-100.";

pub fn env_string(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn env_bool(key: &str) -> bool {
    matches!(
        env::var(key).as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE") | Ok("True") | Ok("yes") | Ok("YES")
    )
}

pub fn env_profile_label() -> Result<String, Box<dyn Error>> {
    env::var("SLMP_PLC_PROFILE").map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            PLC_PROFILE_REQUIRED_MESSAGE.to_string(),
        )
        .into()
    })
}

fn required_env(key: &str) -> Result<String, Box<dyn Error>> {
    env::var(key).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{key} is required"),
        )
        .into()
    })
}

pub fn env_transport_label() -> Result<String, Box<dyn Error>> {
    required_env("SLMP_TRANSPORT")
}

pub fn env_port_label() -> Result<String, Box<dyn Error>> {
    required_env("SLMP_PORT")
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
    let host = env_string("SLMP_HOST", "192.168.250.100");
    let plc_profile = parse_plc_profile(&env_profile_label()?)?;
    let transport = env_transport_label()?;
    let transport_mode = match transport.as_str() {
        "tcp" => SlmpTransportMode::Tcp,
        "udp" => SlmpTransportMode::Udp,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "SLMP_TRANSPORT must be exactly 'tcp' or 'udp'",
            )
            .into());
        }
    };
    let port: u16 = env_port_label()?.parse()?;
    let target = parse_env_target()?;
    let mut options = SlmpConnectionOptions::new(host, port, transport_mode, target, plc_profile)?;
    options.monitoring_timer = env_string("SLMP_MONITORING_TIMER", "16").parse()?;
    options.timeout =
        std::time::Duration::from_millis(env_string("SLMP_TIMEOUT_MS", "3000").parse()?);
    Ok(options)
}

fn parse_env_target() -> Result<SlmpTargetAddress, Box<dyn Error>> {
    let target_fields = [
        "SLMP_NETWORK",
        "SLMP_STATION",
        "SLMP_MODULE_IO",
        "SLMP_MULTIDROP",
    ];
    let has_target_field = target_fields.iter().any(|key| env::var(key).is_ok());
    if has_target_field {
        if env::var("SLMP_TARGET").is_ok() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Use either SLMP_TARGET or SLMP_NETWORK/SLMP_STATION, not both.",
            )
            .into());
        }
        let network = required_env("SLMP_NETWORK")?;
        let station = required_env("SLMP_STATION")?;
        let module_io = required_env("SLMP_MODULE_IO")?;
        let multidrop = required_env("SLMP_MULTIDROP")?;
        return Ok(parse_named_target(&format!(
            "ENV,{network},{station},{module_io},{multidrop}"
        ))?
        .target);
    }
    Ok(parse_named_target(&required_env("SLMP_TARGET")?)?.target)
}

pub async fn connect_from_env() -> Result<SlmpClient, Box<dyn Error>> {
    Ok(SlmpClient::connect(options_from_env()?).await?)
}

pub fn print_connection_banner(example: &str) -> Result<(), Box<dyn Error>> {
    let plc_profile = env_profile_label()?;
    let profile = SlmpPlcProfile::parse_label(&plc_profile).map(SlmpPlcProfile::defaults);
    println!(
        "{example}: host={} port={} plc_profile={} frame={} compatibility={} transport={} target={}",
        env_string("SLMP_HOST", "192.168.250.100"),
        env_port_label()?,
        plc_profile,
        profile
            .map(|profile| format!("{:?}", profile.frame_type))
            .unwrap_or_else(|| "unknown".to_string()),
        profile
            .map(|profile| format!("{:?}", profile.compatibility_mode))
            .unwrap_or_else(|| "unknown".to_string()),
        env_transport_label()?,
        format_env_target()?
    );
    Ok(())
}

fn format_env_target() -> Result<String, Box<dyn Error>> {
    if let Ok(target) = env::var("SLMP_TARGET") {
        parse_named_target(&target)?;
        return Ok(target);
    }
    let target = parse_env_target()?;
    Ok(format!(
        "network={} station={} module_io=0x{:04X} multidrop=0x{:02X}",
        target.network, target.station, target.module_io, target.multidrop
    ))
}

fn parse_plc_profile(value: &str) -> Result<SlmpPlcProfile, Box<dyn Error>> {
    let profile = SlmpPlcProfile::parse_label(value).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            PLC_PROFILE_REQUIRED_MESSAGE.to_string(),
        )
    })?;
    if profile.is_base_profile() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "melsec:qcpu is a base profile; use melsec:qcpu:qj71e71-100.",
        )
        .into());
    }
    Ok(profile)
}
