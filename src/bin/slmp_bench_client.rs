use plc_comm_slmp::{
    SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpPlcProfile,
    SlmpTargetAddress, SlmpTransportMode, parse_target_auto_number,
};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(value) => println!("{value}"),
        Err(error) => println!("{}", json!({"status":"error","message":error})),
    }
}

async fn run() -> Result<Value, String> {
    let config = BenchConfig::from_args(std::env::args().skip(1).collect())?;
    if config.concurrency != 1 {
        return Err("slmp_bench_client currently supports concurrency=1 only".to_string());
    }

    match config.scenario.as_str() {
        "connect" => measure_connect(&config).await,
        "timeout" => measure_timeout(&config).await,
        _ => measure_with_persistent_client(&config).await,
    }
}

async fn measure_with_persistent_client(config: &BenchConfig) -> Result<Value, String> {
    let client = SlmpClient::connect(config.connection_options()?)
        .await
        .map_err(err_msg)?;

    for _ in 0..config.warmup {
        if run_operation_with_timeout(&client, config).await.is_err() {
            break;
        }
    }

    let result = measure_loop(config, || run_operation_with_timeout(&client, config)).await;
    let close_result = client.close().await;
    if let Err(error) = close_result {
        eprintln!("close warning: {}", error.message);
    }
    Ok(result)
}

async fn measure_connect(config: &BenchConfig) -> Result<Value, String> {
    for _ in 0..config.warmup {
        if connect_once(config).await.is_err() {
            break;
        }
    }

    Ok(measure_loop(config, || connect_once(config)).await)
}

async fn measure_timeout(config: &BenchConfig) -> Result<Value, String> {
    for _ in 0..config.warmup {
        if timeout_once(config).await.is_err() {
            break;
        }
    }

    Ok(measure_loop(config, || timeout_once(config)).await)
}

async fn measure_loop<'a, F, Fut>(config: &'a BenchConfig, mut operation: F) -> Value
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<String, String>> + 'a,
{
    let mut latencies = Vec::with_capacity(config.iterations);
    let mut failures = 0usize;
    let mut last_error = String::new();
    let mut sample = String::new();
    let total = Instant::now();

    for _ in 0..config.iterations {
        let start = Instant::now();
        match operation().await {
            Ok(value) => {
                sample = value;
                latencies.push(start.elapsed().as_secs_f64() * 1000.0);
            }
            Err(error) => {
                failures = config.iterations;
                last_error = error;
                break;
            }
        }
    }

    let elapsed_ms = total.elapsed().as_secs_f64() * 1000.0;
    let stats = Stats::from(&latencies);
    json!({
        "status": "success",
        "library": "RustSlmp",
        "case_name": config.case_name,
        "scenario": config.scenario,
        "requests_per_iteration": requests_per_iteration(&config.scenario),
        "iterations": config.iterations,
        "failures": failures,
        "elapsed_ms": elapsed_ms,
        "min_ms": stats.min,
        "max_ms": stats.max,
        "stddev_ms": stats.stddev,
        "p50_ms": stats.p50,
        "p90_ms": stats.p90,
        "p99_ms": stats.p99,
        "allocated_bytes_per_iteration": -1,
        "allocated_bytes_per_request": -1,
        "gc_heap_bytes": -1,
        "gc_gen0": -1,
        "gc_gen1": -1,
        "gc_gen2": -1,
        "sample": sample,
        "last_error": last_error,
        "latencies_ms": latencies,
    })
}

async fn run_operation_with_timeout(
    client: &SlmpClient,
    config: &BenchConfig,
) -> Result<String, String> {
    timeout(config.operation_timeout, execute_scenario(client, config))
        .await
        .map_err(|_| "operation timeout".to_string())?
}

async fn execute_scenario(client: &SlmpClient, config: &BenchConfig) -> Result<String, String> {
    let parse_address = |value| parse_address_for_profile(value, config.plc_profile);
    let parse_device_list = |value| parse_device_list_for_profile(value, config.plc_profile);
    match config.scenario.as_str() {
        "words" => {
            let values = client
                .read_words_raw(parse_address(&config.word_device)?, config.word_points)
                .await
                .map_err(err_msg)?;
            Ok(format_word_sample(&values))
        }
        "bits" => {
            let values = client
                .read_bits(parse_address(&config.bit_device)?, config.bit_points)
                .await
                .map_err(err_msg)?;
            Ok(format_bit_sample(&values))
        }
        "dwords" => {
            let values = client
                .read_dwords_raw(parse_address(&config.dword_device)?, config.dword_points)
                .await
                .map_err(err_msg)?;
            Ok(format_dword_sample(&values))
        }
        "pair" => {
            let bits = client
                .read_bits(parse_address(&config.bit_device)?, config.bit_points)
                .await
                .map_err(err_msg)?;
            let words = client
                .read_words_raw(parse_address(&config.word_device)?, config.word_points)
                .await
                .map_err(err_msg)?;
            Ok(format!(
                "bits={}; words={}",
                format_bit_sample(&bits),
                format_word_sample(&words)
            ))
        }
        "random" => {
            let word_devices = parse_device_list(&config.random_word_devices)?;
            let dword_devices = parse_device_list(&config.random_dword_devices)?;
            let values = client
                .read_random(&word_devices, &dword_devices)
                .await
                .map_err(err_msg)?;
            Ok(format!(
                "random_words={}; random_dwords={}",
                format_word_sample(&values.word_values),
                format_dword_sample(&values.dword_values)
            ))
        }
        "writewords" => {
            let values = pattern_words(config.word_points as usize, 0);
            client
                .write_words(parse_address(&config.word_device)?, &values)
                .await
                .map_err(err_msg)?;
            Ok(format!(
                "wrote {} words to {}",
                values.len(),
                config.word_device
            ))
        }
        "writebits" => {
            let values = (0..config.bit_points)
                .map(|idx| idx % 2 == 0)
                .collect::<Vec<_>>();
            client
                .write_bits(parse_address(&config.bit_device)?, &values)
                .await
                .map_err(err_msg)?;
            Ok(format!(
                "wrote {} bits to {}",
                values.len(),
                config.bit_device
            ))
        }
        "randomwrite" => {
            let word_entries = parse_device_list(&config.random_word_devices)?
                .into_iter()
                .enumerate()
                .map(|(index, device)| (device, index as u16))
                .collect::<Vec<_>>();
            let dword_entries = parse_device_list(&config.random_dword_devices)?
                .into_iter()
                .enumerate()
                .map(|(index, device)| (device, index as u32))
                .collect::<Vec<_>>();
            client
                .write_random_words(&word_entries, &dword_entries)
                .await
                .map_err(err_msg)?;
            Ok(format!(
                "random_wrote {} words and {} dwords",
                word_entries.len(),
                dword_entries.len()
            ))
        }
        "writeverify" => {
            let values = pattern_words(config.word_points as usize, 0xA500);
            let device = parse_address(&config.word_device)?;
            client.write_words(device, &values).await.map_err(err_msg)?;
            let read_back = client
                .read_words_raw(device, config.word_points)
                .await
                .map_err(err_msg)?;
            if values != read_back {
                return Err("write verify mismatch".to_string());
            }
            Ok(format!(
                "verified {} words at {}",
                values.len(),
                config.word_device
            ))
        }
        "keepalivelong" => {
            if config.idle_ms > 0 {
                tokio::time::sleep(Duration::from_millis(config.idle_ms)).await;
            }
            let values = client
                .read_words_raw(parse_address_for_profile("D1000", config.plc_profile)?, 1)
                .await
                .map_err(err_msg)?;
            Ok(format!(
                "alive after {}ms idle: {}",
                config.idle_ms,
                format_word_sample(&values)
            ))
        }
        other => Err(format!("unsupported rust benchmark scenario: {other}")),
    }
}

async fn connect_once(config: &BenchConfig) -> Result<String, String> {
    let client = timeout(
        config.operation_timeout,
        SlmpClient::connect(config.connection_options()?),
    )
    .await
    .map_err(|_| "connect timeout".to_string())?
    .map_err(err_msg)?;
    client.close().await.map_err(err_msg)?;
    Ok("connected and disposed".to_string())
}

async fn timeout_once(config: &BenchConfig) -> Result<String, String> {
    let mut options = config.connection_options()?;
    options.host = "127.0.0.1".to_string();
    options.port = 1;
    options.timeout = Duration::from_millis(500);
    match timeout(Duration::from_millis(750), SlmpClient::connect(options)).await {
        Ok(Ok(client)) => {
            let read = timeout(
                Duration::from_millis(750),
                client.read_words_raw(parse_address_for_profile("D1000", config.plc_profile)?, 1),
            )
            .await;
            let _ = client.close().await;
            match read {
                Ok(Ok(_)) => Ok("connected (unexpected)".to_string()),
                _ => Ok("timeout/error detected".to_string()),
            }
        }
        _ => Ok("timeout/error detected".to_string()),
    }
}

fn pattern_words(count: usize, prefix: u16) -> Vec<u16> {
    (0..count)
        .map(|index| prefix | ((index as u16) & 0x00FF))
        .collect()
}

fn parse_address_for_profile(
    value: &str,
    plc_profile: SlmpPlcProfile,
) -> Result<SlmpDeviceAddress, String> {
    SlmpAddress::parse(value, plc_profile).map_err(err_msg)
}

fn parse_device_list_for_profile(
    value: &str,
    plc_profile: SlmpPlcProfile,
) -> Result<Vec<SlmpDeviceAddress>, String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|value| parse_address_for_profile(value, plc_profile))
        .collect()
}

fn format_word_sample(values: &[u16]) -> String {
    format_values(values.iter().map(|value| format!("0x{value:04X}")))
}

fn format_dword_sample(values: &[u32]) -> String {
    format_values(values.iter().map(|value| format!("0x{value:08X}")))
}

fn format_bit_sample(values: &[bool]) -> String {
    format_values(values.iter().map(|value| {
        if *value {
            "1".to_string()
        } else {
            "0".to_string()
        }
    }))
}

fn format_values(values: impl Iterator<Item = String>) -> String {
    let items = values.take(9).collect::<Vec<_>>();
    if items.len() > 8 {
        format!("[{}, ...]", items[..8].join(", "))
    } else {
        format!("[{}]", items.join(", "))
    }
}

fn requests_per_iteration(scenario: &str) -> usize {
    match scenario {
        "pair" | "writeverify" => 2,
        _ => 1,
    }
}

fn err_msg(error: plc_comm_slmp::SlmpError) -> String {
    error.message
}

#[derive(Debug)]
struct BenchConfig {
    host: String,
    port: u16,
    transport: SlmpTransportMode,
    plc_profile: SlmpPlcProfile,
    target: SlmpTargetAddress,
    case_name: String,
    scenario: String,
    iterations: usize,
    warmup: usize,
    concurrency: usize,
    operation_timeout: Duration,
    idle_ms: u64,
    word_device: String,
    word_points: u16,
    bit_device: String,
    bit_points: u16,
    dword_device: String,
    dword_points: u16,
    random_word_devices: String,
    random_dword_devices: String,
}

impl BenchConfig {
    fn from_args(args: Vec<String>) -> Result<Self, String> {
        let host = option(&args, "--host").unwrap_or_else(|| "192.168.250.100".to_string());
        let transport = match required_option(&args, "--transport")?
            .to_ascii_lowercase()
            .as_str()
        {
            "udp" => SlmpTransportMode::Udp,
            "tcp" => SlmpTransportMode::Tcp,
            other => return Err(format!("unsupported transport: {other}")),
        };
        let port = required_option(&args, "--port")?
            .parse::<u16>()
            .map_err(|error| format!("--port: {error}"))?;
        if port == 0 {
            return Err("--port must be in 1..=65535".to_string());
        }
        let plc_profile = SlmpPlcProfile::parse_label(
            &option(&args, "--plc-profile")
                .ok_or_else(|| "--plc-profile is required".to_string())?,
        )
        .ok_or_else(|| "unsupported --plc-profile".to_string())?;
        if plc_profile.is_base_profile() {
            return Err("melsec:qcpu is a base profile; use melsec:qcpu:qj71e71-100.".to_string());
        }
        let operation_timeout_ms: u64 = parse_option(&args, "--operation-timeout-ms", "2000")?;

        Ok(Self {
            host,
            port,
            transport,
            plc_profile,
            target: target_from_args(&args)?,
            case_name: option(&args, "--case-name").unwrap_or_else(|| "rust-case".to_string()),
            scenario: option(&args, "--scenario")
                .unwrap_or_else(|| "words".to_string())
                .to_ascii_lowercase(),
            iterations: parse_option(&args, "--iterations", "1000")?,
            warmup: parse_option(&args, "--warmup", "50")?,
            concurrency: parse_option(&args, "--concurrency", "1")?,
            operation_timeout: Duration::from_millis(operation_timeout_ms),
            idle_ms: parse_option(&args, "--idle-ms", "60000")?,
            word_device: option(&args, "--word-device").unwrap_or_else(|| "D1000".to_string()),
            word_points: parse_option(&args, "--word-points", "1")?,
            bit_device: option(&args, "--bit-device").unwrap_or_else(|| "SM400".to_string()),
            bit_points: parse_option(&args, "--bit-points", "1")?,
            dword_device: option(&args, "--dword-device").unwrap_or_else(|| "D1000".to_string()),
            dword_points: parse_option(&args, "--dword-points", "1")?,
            random_word_devices: option(&args, "--random-word-devices")
                .unwrap_or_else(|| "D1000,D1010,D1100,D1200".to_string()),
            random_dword_devices: option(&args, "--random-dword-devices")
                .unwrap_or_else(|| "D1300,D1400".to_string()),
        })
    }

    fn connection_options(&self) -> Result<SlmpConnectionOptions, String> {
        let mut options = SlmpConnectionOptions::new(
            self.host.clone(),
            self.port,
            self.transport,
            self.target,
            self.plc_profile,
        )
        .map_err(|error| error.message)?;
        options.timeout = self.operation_timeout;
        Ok(options)
    }
}

fn target_from_args(args: &[String]) -> Result<SlmpTargetAddress, String> {
    let network =
        parse_target_auto_number(&required_option(args, "--network")?).map_err(err_msg)?;
    let station =
        parse_target_auto_number(&required_option(args, "--station")?).map_err(err_msg)?;
    let module_io =
        parse_target_auto_number(&required_option(args, "--module-io")?).map_err(err_msg)?;
    let multidrop =
        parse_target_auto_number(&required_option(args, "--multidrop")?).map_err(err_msg)?;
    Ok(SlmpTargetAddress {
        network: u8::try_from(network).map_err(|_| "--network must fit in u8".to_string())?,
        station: u8::try_from(station).map_err(|_| "--station must fit in u8".to_string())?,
        module_io: u16::try_from(module_io)
            .map_err(|_| "--module-io must fit in u16".to_string())?,
        multidrop: u8::try_from(multidrop).map_err(|_| "--multidrop must fit in u8".to_string())?,
    })
}

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0].eq_ignore_ascii_case(name))
        .map(|pair| pair[1].clone())
}

fn required_option(args: &[String], name: &str) -> Result<String, String> {
    option(args, name).ok_or_else(|| format!("{name} is required"))
}

fn parse_option<T>(args: &[String], name: &str, default: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    option(args, name)
        .unwrap_or_else(|| default.to_string())
        .parse::<T>()
        .map_err(|error| format!("{name}: {error}"))
}

struct Stats {
    min: f64,
    max: f64,
    stddev: f64,
    p50: f64,
    p90: f64,
    p99: f64,
}

impl Stats {
    fn from(values: &[f64]) -> Self {
        if values.is_empty() {
            return Self {
                min: 0.0,
                max: 0.0,
                stddev: 0.0,
                p50: 0.0,
                p90: 0.0,
                p99: 0.0,
            };
        }
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let stddev = if values.len() > 1 {
            let variance = values
                .iter()
                .map(|value| {
                    let diff = value - avg;
                    diff * diff
                })
                .sum::<f64>()
                / (values.len() - 1) as f64;
            variance.sqrt()
        } else {
            0.0
        };
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.total_cmp(b));
        Self {
            min,
            max,
            stddev,
            p50: percentile(&sorted, 50.0),
            p90: percentile(&sorted, 90.0),
            p99: percentile(&sorted, 99.0),
        }
    }
}

fn percentile(sorted: &[f64], percent: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let rank = (percent / 100.0) * (sorted.len().saturating_sub(1)) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = rank - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}
