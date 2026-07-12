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
    let device = args.get(1).cloned().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "device argument is required: polling_reconnect DEVICE DTYPE [INTERVAL_SECONDS]",
        )
    })?;
    let dtype = args.get(2).cloned().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "dtype argument is required: polling_reconnect DEVICE DTYPE [INTERVAL_SECONDS]",
        )
    })?;
    let interval = Duration::from_secs_f64(
        args.get(3)
            .cloned()
            .unwrap_or_else(|| env_string("SLMP_POLL_INTERVAL_SECONDS", "1"))
            .parse::<f64>()?,
    );

    let config = PollConfig {
        device,
        dtype,
        interval,
    };
    let initial_backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);
    let mut state = PollState {
        client: None,
        backoff: initial_backoff,
        connected_once: false,
    };

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                if let Some(client) = state.client.take() {
                    let _ = client.close().await;
                }
                log_state("closed", "interrupted by Ctrl+C");
                break;
            }
            result = poll_step(&config, &mut state, initial_backoff, max_backoff) => {
                result?;
            }
        }
    }

    Ok(())
}

struct PollConfig {
    device: String,
    dtype: String,
    interval: Duration,
}

struct PollState {
    client: Option<SlmpClient>,
    backoff: Duration,
    connected_once: bool,
}

async fn poll_step(
    config: &PollConfig,
    state: &mut PollState,
    initial_backoff: Duration,
    max_backoff: Duration,
) -> Result<(), Box<dyn Error>> {
    if state.client.is_none() {
        log_state("reconnecting", "opening SLMP session");
        match SlmpClient::connect(options_from_env()?).await {
            Ok(new_client) => {
                state.client = Some(new_client);
                log_state(
                    if state.connected_once {
                        "recovered"
                    } else {
                        "connected"
                    },
                    &format!("{}:{}", config.device, config.dtype),
                );
                state.connected_once = true;
                state.backoff = initial_backoff;
            }
            Err(error) if is_retryable_slmp(&error) => {
                log_state(
                    "reconnecting",
                    &format!(
                        "connect failed: {error}; retry in {:.1}s",
                        state.backoff.as_secs_f64()
                    ),
                );
                sleep(state.backoff).await;
                state.backoff = next_backoff(state.backoff, max_backoff);
                return Ok(());
            }
            Err(error) => return Err(Box::new(error)),
        }
    }

    let active = state.client.as_ref().expect("client was just connected");
    let plc_profile = active.plc_profile().await;
    match read_typed(
        active,
        SlmpAddress::parse(&config.device, plc_profile)?,
        &config.dtype,
    )
    .await
    {
        Ok(value) => {
            log_state(
                "read",
                &format!("{}:{}={value:?}", config.device, config.dtype),
            );
            sleep(config.interval).await;
        }
        Err(error) if is_retryable_slmp(&error) => {
            log_state("lost", &error.to_string());
            if let Some(client) = state.client.take() {
                let _ = client.close().await;
            }
            log_state(
                "reconnecting",
                &format!("retry in {:.1}s", state.backoff.as_secs_f64()),
            );
            sleep(state.backoff).await;
            state.backoff = next_backoff(state.backoff, max_backoff);
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
