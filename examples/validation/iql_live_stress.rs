#[path = "../common/mod.rs"]
mod common;

use common::env_string;
use plc_comm_slmp::{
    SlmpBlockRead, SlmpBlockWrite, SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress,
    SlmpDeviceCode, SlmpPlcProfile, SlmpTransportMode, SlmpValue, parse_named_target,
    read_dwords_single_request, read_typed, read_words_single_request, write_dwords_single_request,
    write_typed, write_words_single_request,
};
use std::error::Error;
use std::future::Future;
use std::time::{Duration, Instant};

fn make_error(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::other(message.into()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let host = env_string("SLMP_HOST", "192.168.250.100");
    let transports = required_env("SLMP_STRESS_TRANSPORTS")?
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if transports.is_empty() {
        return Err(make_error("SLMP_STRESS_TRANSPORTS must not be empty"));
    }
    let uses_tcp = transports
        .iter()
        .any(|value| value.eq_ignore_ascii_case("tcp"));
    let uses_udp = transports
        .iter()
        .any(|value| value.eq_ignore_ascii_case("udp"));
    let tcp_port = uses_tcp
        .then(|| required_port("SLMP_TCP_PORT"))
        .transpose()?;
    let udp_port = uses_udp
        .then(|| required_port("SLMP_UDP_PORT"))
        .transpose()?;
    let mut failures = Vec::new();

    println!("iql_live_stress: host={host} tcp_port={tcp_port:?} udp_port={udp_port:?}");
    for transport in transports {
        match transport.to_ascii_lowercase().as_str() {
            "tcp" => {
                failures.extend(
                    run_transport(
                        &host,
                        tcp_port.expect("TCP port was validated"),
                        SlmpTransportMode::Tcp,
                    )
                    .await?,
                );
            }
            "udp" => {
                failures.extend(
                    run_transport(
                        &host,
                        udp_port.expect("UDP port was validated"),
                        SlmpTransportMode::Udp,
                    )
                    .await?,
                );
            }
            other => failures.push(format!("unknown transport '{other}'")),
        }
    }

    if failures.is_empty() {
        println!("summary -> passed all iQ-L live stress checks");
        Ok(())
    } else {
        println!("summary -> failed={}", failures.len());
        for failure in &failures {
            println!("NG {failure}");
        }
        Err(make_error("one or more iQ-L live stress checks failed"))
    }
}

async fn run_transport(
    host: &str,
    port: u16,
    transport_mode: SlmpTransportMode,
) -> Result<Vec<String>, Box<dyn Error>> {
    let label = format!("{transport_mode:?}:{port}");
    let mut failures = Vec::new();
    let options = options(host, port, transport_mode, 3_000)?;
    let client = SlmpClient::connect(options.clone()).await?;

    println!("transport {label} -> start");
    record_step(&mut failures, &label, "single direct max words", || {
        direct_words_roundtrip(&client)
    })
    .await;
    record_step(&mut failures, &label, "single direct max dwords", || {
        direct_dwords_roundtrip(&client)
    })
    .await;
    record_step(&mut failures, &label, "direct bit write/readback", || {
        bits_roundtrip(&client)
    })
    .await;
    record_step(&mut failures, &label, "random word/dword route", || {
        random_roundtrip(&client)
    })
    .await;
    record_step(&mut failures, &label, "block word/bit route", || {
        block_roundtrip(&client)
    })
    .await;
    record_step(&mut failures, &label, "typed helper route", || {
        typed_roundtrip(&client)
    })
    .await;
    record_step(&mut failures, &label, "expected limit errors", || {
        expected_limit_errors(&client)
    })
    .await;
    client.close().await?;
    drop(client);

    record_step(&mut failures, &label, "timeout and reconnect", || {
        timeout_and_reconnect(host, port, transport_mode)
    })
    .await;
    println!("transport {label} -> failed={}", failures.len());
    Ok(failures)
}

async fn record_step<F, Fut>(failures: &mut Vec<String>, label: &str, name: &str, step: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), Box<dyn Error>>>,
{
    let started = Instant::now();
    match step().await {
        Ok(()) => println!(
            "PASS {label} {name} elapsed_ms={}",
            started.elapsed().as_millis()
        ),
        Err(error) => {
            let message = format!("{label} {name}: {error}");
            println!("FAIL {message}");
            failures.push(message);
        }
    }
}

async fn direct_words_roundtrip(client: &SlmpClient) -> Result<(), Box<dyn Error>> {
    let start = device(SlmpDeviceCode::D, 14_000);
    let count = 960;
    let original = read_words_single_request(client, start, count).await?;
    let values = word_pattern(count, 0x1200);
    let result = async {
        write_words_single_request(client, start, &values).await?;
        let actual = read_words_single_request(client, start, count).await?;
        ensure_eq("direct words readback", &values, &actual)
    }
    .await;
    restore_words(client, start, &original).await?;
    result
}

async fn direct_dwords_roundtrip(client: &SlmpClient) -> Result<(), Box<dyn Error>> {
    let start = device(SlmpDeviceCode::D, 15_000);
    let count = 480;
    let original = read_dwords_single_request(client, start, count).await?;
    let values = dword_pattern(count, 0x1200_0000);
    let result = async {
        write_dwords_single_request(client, start, &values).await?;
        let actual = read_dwords_single_request(client, start, count).await?;
        ensure_eq("direct dwords readback", &values, &actual)
    }
    .await;
    restore_dwords(client, start, &original).await?;
    result
}

async fn bits_roundtrip(client: &SlmpClient) -> Result<(), Box<dyn Error>> {
    let start = device(SlmpDeviceCode::M, 100);
    let count = 960;
    let original = client.read_bits(start, count).await?;
    let values = (0..count)
        .map(|index| index % 3 == 0 || index % 7 == 0)
        .collect::<Vec<_>>();
    let result = async {
        client.write_bits(start, &values).await?;
        let actual = client.read_bits(start, count).await?;
        ensure_eq("bits readback", &values, &actual)
    }
    .await;
    client.write_bits(start, &original).await?;
    let restored = client.read_bits(start, count).await?;
    ensure_eq("bits restore", &original, &restored)?;
    result
}

async fn random_roundtrip(client: &SlmpClient) -> Result<(), Box<dyn Error>> {
    let random_points = env_string("SLMP_RANDOM_DEVICE_POINTS", "48").parse::<usize>()?;
    if !(2..=255).contains(&random_points) {
        return Err(make_error(
            "SLMP_RANDOM_DEVICE_POINTS must be in the range 2-255",
        ));
    }
    let mixed_words_count = random_points / 2;
    let mixed_dwords_count = random_points - mixed_words_count;
    println!(
        "REFERENCE random_device_points={random_points} for this iQ-L live target; practical limits are PLC/model dependent"
    );

    let word_devices = (0..random_points)
        .map(|index| device(SlmpDeviceCode::D, 10_000 + index as u32))
        .collect::<Vec<_>>();
    let dword_devices = (0..random_points)
        .map(|index| device(SlmpDeviceCode::D, 11_000 + (index * 2) as u32))
        .collect::<Vec<_>>();
    random_word_dword_roundtrip(client, &word_devices, &[], 0x3200, 0x3200_0000, "word-only")
        .await?;
    random_word_dword_roundtrip(
        client,
        &[],
        &dword_devices,
        0x3300,
        0x3300_0000,
        "dword-only",
    )
    .await?;

    let mixed_words = (0..mixed_words_count)
        .map(|index| device(SlmpDeviceCode::D, 10_500 + index as u32))
        .collect::<Vec<_>>();
    let mixed_dwords = (0..mixed_dwords_count)
        .map(|index| device(SlmpDeviceCode::D, 11_500 + (index * 2) as u32))
        .collect::<Vec<_>>();
    random_word_dword_roundtrip(
        client,
        &mixed_words,
        &mixed_dwords,
        0x3400,
        0x3400_0000,
        "mixed-reference",
    )
    .await?;
    observe_random_reference_probe(client, random_points + 1).await;
    Ok(())
}

async fn observe_random_reference_probe(client: &SlmpClient, points: usize) {
    if points > 255 {
        return;
    }
    let devices = (0..points)
        .map(|index| device(SlmpDeviceCode::D, 10_000 + index as u32))
        .collect::<Vec<_>>();
    match client.read_random(&devices, &[]).await {
        Ok(_) => println!("OBSERVED-OK random read probe word count {points}"),
        Err(error) => println!("OBSERVED-NG random read probe word count {points}: {error}"),
    }
}

async fn random_word_dword_roundtrip(
    client: &SlmpClient,
    word_devices: &[SlmpDeviceAddress],
    dword_devices: &[SlmpDeviceAddress],
    word_seed: u16,
    dword_seed: u32,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    let original = client.read_random(word_devices, dword_devices).await?;
    let word_values = word_pattern(word_devices.len(), word_seed);
    let dword_values = dword_pattern(dword_devices.len(), dword_seed);
    let word_entries = word_devices
        .iter()
        .copied()
        .zip(word_values.iter().copied())
        .collect::<Vec<_>>();
    let dword_entries = dword_devices
        .iter()
        .copied()
        .zip(dword_values.iter().copied())
        .collect::<Vec<_>>();
    let restore_word_entries = word_devices
        .iter()
        .copied()
        .zip(original.word_values.iter().copied())
        .collect::<Vec<_>>();
    let restore_dword_entries = dword_devices
        .iter()
        .copied()
        .zip(original.dword_values.iter().copied())
        .collect::<Vec<_>>();
    let result = async {
        client
            .write_random_words(&word_entries, &dword_entries)
            .await?;
        let actual = client.read_random(word_devices, dword_devices).await?;
        ensure_eq(
            &format!("random {label} word readback"),
            &word_values,
            &actual.word_values,
        )?;
        ensure_eq(
            &format!("random {label} dword readback"),
            &dword_values,
            &actual.dword_values,
        )
    }
    .await;
    client
        .write_random_words(&restore_word_entries, &restore_dword_entries)
        .await?;
    let restored = client.read_random(word_devices, dword_devices).await?;
    ensure_eq(
        &format!("random {label} word restore"),
        &original.word_values,
        &restored.word_values,
    )?;
    ensure_eq(
        &format!("random {label} dword restore"),
        &original.dword_values,
        &restored.dword_values,
    )?;
    result
}

async fn block_roundtrip(client: &SlmpClient) -> Result<(), Box<dyn Error>> {
    let word_blocks = vec![SlmpBlockRead {
        device: device(SlmpDeviceCode::D, 12_000),
        points: 8,
    }];
    let bit_blocks = vec![SlmpBlockRead {
        device: device(SlmpDeviceCode::M, 2_000),
        points: 2,
    }];
    let original = client.read_block(&word_blocks, &bit_blocks).await?;
    let word_values_a = word_pattern(8, 0x4200);
    let bit_values = vec![0xAAAA, 0x5555];
    let result = async {
        client
            .write_block(
                &[SlmpBlockWrite {
                    device: word_blocks[0].device,
                    values: word_values_a.clone(),
                }],
                &[SlmpBlockWrite {
                    device: bit_blocks[0].device,
                    values: bit_values.clone(),
                }],
            )
            .await?;
        let actual = client.read_block(&word_blocks, &bit_blocks).await?;
        ensure_eq("block word readback", &word_values_a, &actual.word_values)?;
        ensure_eq("block bit readback", &bit_values, &actual.bit_values)
    }
    .await;
    client
        .write_block(
            &[SlmpBlockWrite {
                device: word_blocks[0].device,
                values: original.word_values.clone(),
            }],
            &[SlmpBlockWrite {
                device: bit_blocks[0].device,
                values: original.bit_values.clone(),
            }],
        )
        .await?;
    let restored = client.read_block(&word_blocks, &bit_blocks).await?;
    ensure_eq(
        "block word restore",
        &original.word_values,
        &restored.word_values,
    )?;
    ensure_eq(
        "block bit restore",
        &original.bit_values,
        &restored.bit_values,
    )?;
    observe_multi_word_block_candidate(client).await;
    result
}

async fn observe_multi_word_block_candidate(client: &SlmpClient) {
    let probe = async {
        let first = device(SlmpDeviceCode::D, 12_020);
        let second = device(SlmpDeviceCode::D, 12_040);
        let original_first = client.read_words_raw(first, 4).await?;
        let original_second = client.read_words_raw(second, 4).await?;
        let values_first = word_pattern(4, 0x4400);
        let values_second = word_pattern(4, 0x4500);
        let result: Result<(), plc_comm_slmp::SlmpError> = async {
            client
                .write_block(
                    &[
                        SlmpBlockWrite {
                            device: first,
                            values: values_first.clone(),
                        },
                        SlmpBlockWrite {
                            device: second,
                            values: values_second.clone(),
                        },
                    ],
                    &[],
                )
                .await?;
            let actual_first = client.read_words_raw(first, 4).await?;
            let actual_second = client.read_words_raw(second, 4).await?;
            if actual_first != values_first || actual_second != values_second {
                return Err(plc_comm_slmp::SlmpError::new(format!(
                    "multi word-block readback mismatch first={actual_first:?} second={actual_second:?}"
                )));
            }
            Ok(())
        }
        .await;
        let restore_first = client.write_words(first, &original_first).await;
        let restore_second = client.write_words(second, &original_second).await;
        restore_first?;
        restore_second?;
        result
    }
    .await;

    match probe {
        Ok(()) => println!("OBSERVED-OK block two word-block write candidate"),
        Err(error) => println!("OBSERVED-NG block two word-block write candidate: {error}"),
    }
}

async fn typed_roundtrip(client: &SlmpClient) -> Result<(), Box<dyn Error>> {
    typed_one(
        client,
        device(SlmpDeviceCode::D, 13_000),
        "U",
        SlmpValue::U16(0x4321),
    )
    .await?;
    typed_one(
        client,
        device(SlmpDeviceCode::D, 13_002),
        "D",
        SlmpValue::U32(0x5566_7788),
    )
    .await?;
    typed_one(
        client,
        device(SlmpDeviceCode::D, 13_004),
        "F",
        SlmpValue::F32(12.5),
    )
    .await?;
    typed_one(
        client,
        device(SlmpDeviceCode::M, 3_000),
        "BIT",
        SlmpValue::Bool(true),
    )
    .await?;
    typed_one(
        client,
        device(SlmpDeviceCode::LZ, 0),
        "D",
        SlmpValue::U32(0x1020_3040),
    )
    .await?;
    typed_one(
        client,
        device(SlmpDeviceCode::RD, 0),
        "U",
        SlmpValue::U16(0x2468),
    )
    .await
}

async fn typed_one(
    client: &SlmpClient,
    address: SlmpDeviceAddress,
    dtype: &str,
    value: SlmpValue,
) -> Result<(), Box<dyn Error>> {
    let original = read_typed(client, address, dtype).await?;
    let result = async {
        write_typed(client, address, dtype, &value).await?;
        let actual = read_typed(client, address, dtype).await?;
        ensure_value_eq(
            &format!("typed {address}:{dtype} readback"),
            &value,
            &actual,
        )
    }
    .await;
    write_typed(client, address, dtype, &original).await?;
    let restored = read_typed(client, address, dtype).await?;
    ensure_value_eq(
        &format!("typed {address}:{dtype} restore"),
        &original,
        &restored,
    )?;
    result
}

async fn expected_limit_errors(client: &SlmpClient) -> Result<(), Box<dyn Error>> {
    expect_err(
        "read_words_single_request count 961",
        read_words_single_request(client, device(SlmpDeviceCode::D, 14_000), 961).await,
    )?;
    expect_err(
        "read_dwords_single_request count 481",
        read_dwords_single_request(client, device(SlmpDeviceCode::D, 15_000), 481).await,
    )?;
    expect_err(
        "write_words_single_request len 961",
        write_words_single_request(client, device(SlmpDeviceCode::D, 14_000), &vec![0; 961]).await,
    )?;
    let too_many_random = (0..256)
        .map(|index| device(SlmpDeviceCode::D, 10_000 + index))
        .collect::<Vec<_>>();
    expect_err(
        "read_random word count 256",
        client.read_random(&too_many_random, &[]).await,
    )?;
    Ok(())
}

async fn timeout_and_reconnect(
    host: &str,
    port: u16,
    transport_mode: SlmpTransportMode,
) -> Result<(), Box<dyn Error>> {
    tokio::time::sleep(Duration::from_millis(1_500)).await;
    let mut transient_errors = 0usize;
    for cycle in 0..10 {
        let mut cycle_ok = false;
        let mut last_error = String::new();
        for attempt in 1..=3 {
            let result = async {
                let client =
                    SlmpClient::connect(options(host, port, transport_mode, 3_000)?).await?;
                let _ = client
                    .read_words_raw(device(SlmpDeviceCode::D, 0), 1)
                    .await?;
                client.close().await?;
                Ok::<(), Box<dyn Error>>(())
            }
            .await;
            match result {
                Ok(()) => {
                    cycle_ok = true;
                    break;
                }
                Err(error) => {
                    transient_errors += 1;
                    last_error = error.to_string();
                    println!(
                        "OBSERVED-NG {transport_mode:?}:{port} reconnect cycle={} attempt={} {error}",
                        cycle + 1,
                        attempt
                    );
                    tokio::time::sleep(Duration::from_millis(1_000)).await;
                }
            }
        }
        if !cycle_ok {
            return Err(make_error(format!(
                "{transport_mode:?}:{port} reconnect cycle {} failed after retries: {last_error}",
                cycle + 1
            )));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    if transient_errors > 0 {
        println!(
            "OBSERVED-NG {transport_mode:?}:{port} reconnect transient_errors={transient_errors}"
        );
    }

    let bad_port = match transport_mode {
        SlmpTransportMode::Tcp => 65_000,
        SlmpTransportMode::Udp => port + 100,
    };
    let bad_result = async {
        let client = SlmpClient::connect(options(host, bad_port, transport_mode, 500)?).await?;
        client
            .read_words_raw(device(SlmpDeviceCode::D, 0), 1)
            .await?;
        Ok::<(), Box<dyn Error>>(())
    }
    .await;
    if bad_result.is_ok() {
        return Err(make_error(format!(
            "{transport_mode:?} bad port {bad_port} unexpectedly succeeded"
        )));
    }
    println!(
        "EXPECTED-ERR {transport_mode:?}:{bad_port} {}",
        bad_result.unwrap_err()
    );

    let client = SlmpClient::connect(options(host, port, transport_mode, 3_000)?).await?;
    let _ = client
        .read_words_raw(device(SlmpDeviceCode::D, 0), 1)
        .await?;
    client.close().await?;
    Ok(())
}

async fn restore_words(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    original: &[u16],
) -> Result<(), Box<dyn Error>> {
    write_words_single_request(client, start, original).await?;
    let restored = read_words_single_request(client, start, original.len()).await?;
    ensure_eq("direct words restore", original, &restored)
}

async fn restore_dwords(
    client: &SlmpClient,
    start: SlmpDeviceAddress,
    original: &[u32],
) -> Result<(), Box<dyn Error>> {
    write_dwords_single_request(client, start, original).await?;
    let restored = read_dwords_single_request(client, start, original.len()).await?;
    ensure_eq("direct dwords restore", original, &restored)
}

fn options(
    host: &str,
    port: u16,
    transport_mode: SlmpTransportMode,
    timeout_ms: u64,
) -> Result<SlmpConnectionOptions, Box<dyn Error>> {
    let target = parse_named_target(&required_env("SLMP_TARGET")?)?.target;
    let mut options =
        SlmpConnectionOptions::new(host, port, transport_mode, target, SlmpPlcProfile::IqL)?;
    options.timeout = Duration::from_millis(timeout_ms);
    Ok(options)
}

fn required_env(key: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(key).map_err(|_| make_error(format!("{key} is required")))
}

fn required_port(key: &str) -> Result<u16, Box<dyn Error>> {
    let port = required_env(key)?.parse::<u16>()?;
    if port == 0 {
        return Err(make_error(format!("{key} must be in 1..=65535")));
    }
    Ok(port)
}

fn device(code: SlmpDeviceCode, number: u32) -> SlmpDeviceAddress {
    SlmpDeviceAddress::new(code, number, SlmpPlcProfile::IqL)
}

fn word_pattern(count: usize, seed: u16) -> Vec<u16> {
    (0..count)
        .map(|index| seed.wrapping_add((index as u16).wrapping_mul(37)))
        .collect()
}

fn dword_pattern(count: usize, seed: u32) -> Vec<u32> {
    (0..count)
        .map(|index| seed.wrapping_add((index as u32).wrapping_mul(65_537)))
        .collect()
}

fn ensure_eq<T>(label: &str, expected: &[T], actual: &[T]) -> Result<(), Box<dyn Error>>
where
    T: std::fmt::Debug + PartialEq,
{
    if expected == actual {
        return Ok(());
    }
    let first_mismatch = expected
        .iter()
        .zip(actual.iter())
        .position(|(left, right)| left != right);
    Err(make_error(format!(
        "{label} mismatch len expected={} actual={} first_mismatch={first_mismatch:?}",
        expected.len(),
        actual.len()
    )))
}

fn ensure_value_eq(
    label: &str,
    expected: &SlmpValue,
    actual: &SlmpValue,
) -> Result<(), Box<dyn Error>> {
    if expected == actual {
        Ok(())
    } else {
        Err(make_error(format!(
            "{label} mismatch expected={expected:?} actual={actual:?}"
        )))
    }
}

fn expect_err<T>(
    label: &str,
    result: Result<T, plc_comm_slmp::SlmpError>,
) -> Result<(), Box<dyn Error>> {
    match result {
        Ok(_) => Err(make_error(format!("{label} unexpectedly succeeded"))),
        Err(error) => {
            println!("EXPECTED-ERR {label}: {error}");
            Ok(())
        }
    }
}
