//! Read-only polling loop with automatic reconnect.
//!
//! Connection settings use the same environment variables as the other SLMP
//! examples. The polled device, dtype, and interval can be passed as
//! positional arguments:
//!
//!   cargo run --features cli --example polling_reconnect -- D100 U 1

mod common;

use common::{env_string, options_from_env, print_connection_banner};
use plc_comm_slmp::{SlmpAddress, SlmpClient, SlmpError, SlmpErrorKind, read_typed};
use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("polling_reconnect")?;

    let args = std::env::args().collect::<Vec<_>>();
    let device = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| env_string("SLMP_POLL_DEVICE", "D100"));
    let dtype = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| env_string("SLMP_POLL_DTYPE", "U"));
    let interval = Duration::from_secs_f64(
        args.get(3)
            .cloned()
            .unwrap_or_else(|| env_string("SLMP_POLL_INTERVAL_SECONDS", "1"))
            .parse::<f64>()?,
    );

    let initial_backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);
    let mut backoff = initial_backoff;
    let mut client: Option<SlmpClient> = None;
    let mut connected_once = false;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if let Some(client) = client.take() {
                    let _ = client.close().await;
                }
                log_state("closed", "interrupted by Ctrl+C");
                break;
            }
            result = poll_step(&mut client, &device, &dtype, &mut backoff, initial_backoff, max_backoff, &mut connected_once, interval) => {
                if let Err(error) = result {
                    return Err(error);
                }
            }
        }
    }

    Ok(())
}

async fn poll_step(
    client: &mut Option<SlmpClient>,
    device: &str,
    dtype: &str,
    backoff: &mut Duration,
    initial_backoff: Duration,
    max_backoff: Duration,
    connected_once: &mut bool,
    interval: Duration,
) -> Result<(), Box<dyn Error>> {
    if client.is_none() {
        log_state("reconnecting", "opening SLMP session");
        match SlmpClient::connect(options_from_env()?).await {
            Ok(new_client) => {
                *client = Some(new_client);
                log_state(
                    if *connected_once { "recovered" } else { "connected" },
                    &format!("{device}:{dtype}"),
                );
                *connected_once = true;
                *backoff = initial_backoff;
            }
            Err(error) if is_retryable_slmp(&error) => {
                log_state(
                    "reconnecting",
                    &format!("connect failed: {error}; retry in {:.1}s", backoff.as_secs_f64()),
                );
                sleep(*backoff).await;
                *backoff = next_backoff(*backoff, max_backoff);
                return Ok(());
            }
            Err(error) => return Err(Box::new(error)),
        }
    }

    let active = client.as_ref().expect("client was just connected");
    match read_typed(active, SlmpAddress::parse(device)?, dtype).await {
        Ok(value) => {
            log_state("read", &format!("{device}:{dtype}={value:?}"));
            sleep(interval).await;
        }
        Err(error) if is_retryable_slmp(&error) => {
            log_state("lost", &error.to_string());
            if let Some(client) = client.take() {
                let _ = client.close().await;
            }
            log_state(
                "reconnecting",
                &format!("retry in {:.1}s", backoff.as_secs_f64()),
            );
            sleep(*backoff).await;
            *backoff = next_backoff(*backoff, max_backoff);
        }
        Err(error) => return Err(Box::new(error)),
    }
    Ok(())
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

fn log_state(state: &str, message: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    println!(
        "{}.{:03} [{state}] {message}",
        now.as_secs(),
        now.subsec_millis()
    );
}
