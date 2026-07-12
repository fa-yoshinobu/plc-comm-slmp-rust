#![allow(dead_code)]

use plc_comm_slmp::{
    SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpError, SlmpErrorKind, SlmpPlcProfile,
    SlmpTargetAddress, SlmpTransportMode, parse_named_target, read_typed,
};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;

pub type MonitorResult<T> = Result<T, String>;

#[derive(Clone, Debug)]
pub struct TagSpec {
    pub name: String,
    pub address: String,
}

#[derive(Clone, Debug)]
pub struct PlcEndpoint {
    pub name: String,
    pub host: String,
    pub plc_profile: String,
    pub port: u16,
    pub transport: String,
    pub target: SlmpTargetAddress,
    pub timeout_ms: u64,
    pub interval: Duration,
}

#[derive(Clone)]
pub struct CsvWriter {
    path: Arc<PathBuf>,
    lock: Arc<Mutex<()>>,
}

impl CsvWriter {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: Arc::new(path),
            lock: Arc::new(Mutex::new(())),
        }
    }

    async fn write_collection(
        &self,
        endpoint: &PlcEndpoint,
        values: &BTreeMap<String, String>,
    ) -> MonitorResult<()> {
        let _guard = self.lock.lock().await;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let needs_header = !self.path.exists()
            || self
                .path
                .metadata()
                .map_err(|error| error.to_string())?
                .len()
                == 0;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path.as_ref())
            .map_err(|error| error.to_string())?;
        if needs_header {
            writeln!(file, "timestamp,plc,tag,value").map_err(|error| error.to_string())?;
        }
        let timestamp = timestamp();
        for (tag, value) in values {
            writeln!(file, "{timestamp},{},{tag},{value}", endpoint.name)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}

pub fn parse_transport(value: &str) -> MonitorResult<String> {
    let transport = value.to_ascii_lowercase();
    match transport.as_str() {
        "tcp" | "udp" => Ok(transport),
        _ => Err("transport must be tcp or udp".to_string()),
    }
}

pub fn parse_tag_spec(value: &str) -> MonitorResult<TagSpec> {
    if let Some((name, address)) = value.split_once('=') {
        if name.is_empty() || address.is_empty() {
            return Err("expected NAME=ADDRESS".to_string());
        }
        return Ok(TagSpec {
            name: name.to_string(),
            address: address.to_string(),
        });
    }
    Ok(TagSpec {
        name: normalize_tag_name(value),
        address: value.to_string(),
    })
}

pub fn parse_plc_spec(
    value: &str,
    timeout_ms: u64,
    interval: Duration,
) -> MonitorResult<PlcEndpoint> {
    let Some((name, rest)) = value.split_once('=') else {
        return Err("expected NAME=HOST,PROFILE,PORT,TRANSPORT,TARGET".to_string());
    };
    let parts = rest.splitn(5, ',').map(str::trim).collect::<Vec<_>>();
    if name.is_empty() || parts.len() != 5 || parts.iter().any(|value| value.is_empty()) {
        return Err("expected NAME=HOST,PROFILE,PORT,TRANSPORT,TARGET".to_string());
    }
    let port = parts[2].parse::<u16>().map_err(|error| error.to_string())?;
    if port == 0 {
        return Err("port must be in 1..=65535".to_string());
    }
    let transport = parse_transport(parts[3])?;
    let target = parse_named_target(parts[4])
        .map_err(|error| error.to_string())?
        .target;
    Ok(PlcEndpoint {
        name: name.to_string(),
        host: parts[0].to_string(),
        plc_profile: parts[1].to_string(),
        port,
        transport,
        target,
        timeout_ms,
        interval,
    })
}

pub async fn monitor_endpoint(
    endpoint: PlcEndpoint,
    tags: Vec<TagSpec>,
    cycles: Option<usize>,
    initial_backoff: Duration,
    max_backoff: Duration,
    writer: Option<CsvWriter>,
) -> MonitorResult<()> {
    if tags.is_empty() {
        return Err("at least one tag is required".to_string());
    }

    let mut client: Option<SlmpClient> = None;
    let mut completed = 0_usize;
    let mut backoff = initial_backoff;
    let mut connected_once = false;

    while cycles.is_none_or(|limit| completed < limit) {
        if client.is_none() {
            log_state(
                &endpoint.name,
                "reconnecting",
                &format!(
                    "{} {}:{} profile={}",
                    endpoint.transport, endpoint.host, endpoint.port, endpoint.plc_profile
                ),
            );
            match SlmpClient::connect(options_for(&endpoint)?).await {
                Ok(new_client) => {
                    client = Some(new_client);
                    log_state(
                        &endpoint.name,
                        if connected_once {
                            "recovered"
                        } else {
                            "connected"
                        },
                        &format!("{} tags", tags.len()),
                    );
                    connected_once = true;
                    backoff = initial_backoff;
                }
                Err(error) if is_retryable_slmp(&error) => {
                    log_state(
                        &endpoint.name,
                        "reconnecting",
                        &format!(
                            "connect failed: {error}; retry in {:.1}s",
                            backoff.as_secs_f64()
                        ),
                    );
                    sleep(backoff).await;
                    backoff = next_backoff(backoff, max_backoff);
                    continue;
                }
                Err(error) => return Err(error.to_string()),
            }
        }

        let active = client.as_ref().expect("client was just connected");
        match read_collection(active, &tags).await {
            Ok(values) => {
                log_state(&endpoint.name, "read", &format_collection(&values));
                if let Some(csv_writer) = &writer {
                    csv_writer.write_collection(&endpoint, &values).await?;
                }
                completed += 1;
                if cycles.is_none_or(|limit| completed < limit) {
                    sleep(endpoint.interval).await;
                }
            }
            Err(error) if is_retryable_slmp(&error) => {
                log_state(&endpoint.name, "lost", &error.to_string());
                if let Some(client) = client.take() {
                    let _ = client.close().await;
                }
                log_state(
                    &endpoint.name,
                    "reconnecting",
                    &format!("retry in {:.1}s", backoff.as_secs_f64()),
                );
                sleep(backoff).await;
                backoff = next_backoff(backoff, max_backoff);
            }
            Err(error) => return Err(error.to_string()),
        }
    }

    if let Some(client) = client.take() {
        let _ = client.close().await;
    }
    Ok(())
}

pub fn split_address(address: &str) -> MonitorResult<(&str, &str)> {
    if let Some((device, dtype)) = address.rsplit_once(':') {
        if device.is_empty() || dtype.is_empty() {
            return Err(format!("address must be DEVICE:DTYPE: {address}"));
        }
        Ok((device, dtype))
    } else {
        Err(format!(
            "address must explicitly specify DEVICE:DTYPE: {address}"
        ))
    }
}

pub fn format_endpoint(endpoint: &PlcEndpoint) -> String {
    format!(
        "{}: {} {}:{} profile={} target={:02X},{:02X},{:04X},{:02X} interval={}s",
        endpoint.name,
        endpoint.transport,
        endpoint.host,
        endpoint.port,
        endpoint.plc_profile,
        endpoint.target.network,
        endpoint.target.station,
        endpoint.target.module_io,
        endpoint.target.multidrop,
        endpoint.interval.as_secs_f64()
    )
}

pub fn format_tags(tags: &[TagSpec]) -> String {
    tags.iter()
        .map(|tag| format!("{}={}", tag.name, tag.address))
        .collect::<Vec<_>>()
        .join(", ")
}

fn normalize_tag_name(address: &str) -> String {
    address
        .replace(['\\', ':', '.', '-', '/'], "_")
        .to_ascii_lowercase()
}

fn options_for(endpoint: &PlcEndpoint) -> MonitorResult<SlmpConnectionOptions> {
    let profile = SlmpPlcProfile::parse_label(&endpoint.plc_profile)
        .ok_or_else(|| format!("unsupported plc profile: {}", endpoint.plc_profile))?;
    if profile.is_base_profile() {
        return Err("melsec:qcpu is a base profile; use melsec:qcpu:qj71e71-100.".into());
    }
    let transport_mode = match endpoint.transport.as_str() {
        "udp" => SlmpTransportMode::Udp,
        "tcp" => SlmpTransportMode::Tcp,
        _ => return Err("transport must be explicitly set to tcp or udp".into()),
    };
    let mut options = SlmpConnectionOptions::new(
        endpoint.host.clone(),
        endpoint.port,
        transport_mode,
        endpoint.target,
        profile,
    )
    .map_err(|error| error.to_string())?;
    options.timeout = Duration::from_millis(endpoint.timeout_ms);
    Ok(options)
}

async fn read_collection(
    client: &SlmpClient,
    tags: &[TagSpec],
) -> Result<BTreeMap<String, String>, SlmpError> {
    let mut values = BTreeMap::new();
    for tag in tags {
        let (device, dtype) = split_address(&tag.address).map_err(SlmpError::new)?;
        let value = read_typed(
            client,
            SlmpAddress::parse(device, client.plc_profile().await)?,
            dtype,
        )
        .await?;
        values.insert(tag.name.clone(), format!("{value:?}"));
    }
    Ok(values)
}

fn is_retryable_slmp(error: &SlmpError) -> bool {
    !matches!(
        error.kind,
        SlmpErrorKind::PlcEndCode | SlmpErrorKind::ProfileFeature
    )
}

fn next_backoff(current: Duration, max: Duration) -> Duration {
    Duration::from_secs_f64((current.as_secs_f64() * 2.0).min(max.as_secs_f64()))
}

fn format_collection(values: &BTreeMap<String, String>) -> String {
    values
        .iter()
        .map(|(tag, value)| format!("{tag}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn log_state(plc_name: &str, state: &str, message: &str) {
    println!("{} [{plc_name}] [{state}] {message}", timestamp());
}

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:03}", now.as_secs(), now.subsec_millis())
}
