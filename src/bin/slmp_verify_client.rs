use futures_util::StreamExt;
use plc_comm_slmp::{
    NamedAddress, SlmpAddress, SlmpBlockRead, SlmpBlockWrite, SlmpClient, SlmpCompatibilityMode,
    SlmpConnectionOptions, SlmpExtensionSpec, SlmpFrameType, SlmpPlcFamily, SlmpTargetAddress,
    parse_qualified_device, parse_scalar_for_named_with_family, parse_target_auto_number, poll_named,
    read_named, write_named,
};
use serde_json::json;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        return;
    }

    let host = args[1].clone();
    let port: u16 = args[2].parse().unwrap_or(1025);
    let command = args[3].as_str();
    let address = args.get(4).cloned().unwrap_or_default();
    let mut extras = Vec::new();
    let mut family = SlmpPlcFamily::IqR;
    let mut frame = SlmpFrameType::Frame3E;
    let mut series = SlmpCompatibilityMode::Legacy;
    let mut target = None;
    let mut mode = "word".to_string();
    let mut word_devs = String::new();
    let mut dword_devs = String::new();
    let mut words = String::new();
    let mut dwords = String::new();
    let mut bits = String::new();
    let mut word_blocks = String::new();
    let mut bit_blocks = String::new();

    let mut index = 5usize;
    while index < args.len() {
        match args[index].as_str() {
            "--family" => {
                index += 1;
                family = parse_plc_family(args.get(index).map(String::as_str).unwrap_or("iq-r"));
            }
            "--frame" => {
                index += 1;
                frame = if args.get(index).map(String::as_str) == Some("4e") {
                    SlmpFrameType::Frame4E
                } else {
                    SlmpFrameType::Frame3E
                };
            }
            "--series" => {
                index += 1;
                series = if args.get(index).map(String::as_str) == Some("iqr") {
                    SlmpCompatibilityMode::Iqr
                } else {
                    SlmpCompatibilityMode::Legacy
                };
            }
            "--mode" => {
                index += 1;
                mode = args
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| "word".to_string());
            }
            "--target" => {
                index += 1;
                if let Some(value) = args.get(index) {
                    let parts: Vec<_> = value.split(',').collect();
                    if parts.len() == 4 {
                        target = Some(SlmpTargetAddress {
                            network: parse_target_auto_number(parts[0]).unwrap_or(0) as u8,
                            station: parse_target_auto_number(parts[1]).unwrap_or(0xFF) as u8,
                            module_io: parse_target_auto_number(parts[2]).unwrap_or(0x03FF) as u16,
                            multidrop: parse_target_auto_number(parts[3]).unwrap_or(0) as u8,
                        });
                    }
                }
            }
            "--word-devs" => {
                index += 1;
                word_devs = args.get(index).cloned().unwrap_or_default();
            }
            "--dword-devs" => {
                index += 1;
                dword_devs = args.get(index).cloned().unwrap_or_default();
            }
            "--words" => {
                index += 1;
                words = args.get(index).cloned().unwrap_or_default();
            }
            "--dwords" => {
                index += 1;
                dwords = args.get(index).cloned().unwrap_or_default();
            }
            "--bits" => {
                index += 1;
                bits = args.get(index).cloned().unwrap_or_default();
            }
            "--word-blocks" => {
                index += 1;
                word_blocks = args.get(index).cloned().unwrap_or_default();
            }
            "--bit-blocks" => {
                index += 1;
                bit_blocks = args.get(index).cloned().unwrap_or_default();
            }
            other => extras.push(other.to_string()),
        }
        index += 1;
    }

    let mut options = SlmpConnectionOptions::new(host, family);
    options.port = port;
    options.frame_type = frame;
    options.compatibility_mode = series;
    if let Some(target) = target {
        options.target = target;
    }

    let result = match SlmpClient::connect(options).await {
        Ok(client) => {
            run_command(
                &client,
                command,
                &address,
                &extras,
                &mode,
                &word_devs,
                &dword_devs,
                &words,
                &dwords,
                &bits,
                &word_blocks,
                &bit_blocks,
            )
            .await
        }
        Err(error) => Err(error),
    };

    match result {
        Ok(value) => println!("{}", value),
        Err(error) => println!(
            "{}",
            json!({"status": "error", "message": error.message}).to_string()
        ),
    }
}

fn parse_plc_family(value: &str) -> SlmpPlcFamily {
    SlmpPlcFamily::parse_label(value).unwrap_or(SlmpPlcFamily::IqR)
}

#[allow(clippy::too_many_arguments)]
async fn run_command(
    client: &SlmpClient,
    command: &str,
    address: &str,
    extras: &[String],
    mode: &str,
    word_devs: &str,
    dword_devs: &str,
    words: &str,
    dwords: &str,
    bits: &str,
    word_blocks: &str,
    bit_blocks: &str,
) -> Result<String, plc_comm_slmp::SlmpError> {
    let output = match command {
        "read" => {
            let device = SlmpAddress::parse(address)?;
            let count = extras
                .first()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1);
            match mode {
                "bit" => {
                    json!({"status":"success","values": client.read_bits(device, count).await?.into_iter().map(u8::from).collect::<Vec<_>>() })
                }
                "dword" => {
                    json!({"status":"success","values": client.read_dwords_raw(device, count).await? })
                }
                "float" => {
                    json!({"status":"success","values": client.read_float32s(device, count).await? })
                }
                _ => {
                    json!({"status":"success","values": client.read_words_raw(device, count).await? })
                }
            }
        }
        "write" => {
            let device = SlmpAddress::parse(address)?;
            match mode {
                "bit" => {
                    let values: Vec<bool> = extras.iter().map(|value| value == "1").collect();
                    client.write_bits(device, &values).await?;
                }
                "dword" => {
                    let values: Vec<u32> = extras
                        .iter()
                        .map(|value| value.parse().unwrap_or(0))
                        .collect();
                    client.write_dwords(device, &values).await?;
                }
                "float" => {
                    let values: Vec<f32> = extras
                        .iter()
                        .map(|value| value.parse().unwrap_or(0.0))
                        .collect();
                    client.write_float32s(device, &values).await?;
                }
                _ => {
                    let values: Vec<u16> = extras
                        .iter()
                        .map(|value| value.parse().unwrap_or(0))
                        .collect();
                    client.write_words(device, &values).await?;
                }
            }
            json!({"status":"success"})
        }
        "read-type" => {
            let info = client.read_type_name().await?;
            json!({"status":"success","model": info.model, "model_code": format!("0x{:04X}", info.model_code)})
        }
        "read-named" => {
            let addresses = parse_named_addresses(address);
            let values = read_named(client, &addresses).await?;
            json!({
                "status":"success",
                "addresses": addresses,
                "values": addresses.iter().map(|key| serde_json::to_value(values.get(key).unwrap()).unwrap()).collect::<Vec<_>>()
            })
        }
        "write-named" => {
            let updates = parse_named_updates(address, client.plc_family().await)?;
            write_named(client, &updates).await?;
            json!({"status":"success"})
        }
        "poll-once" => {
            let addresses = parse_named_addresses(address);
            let mut stream = Box::pin(poll_named(
                client,
                &addresses,
                std::time::Duration::from_millis(0),
            ));
            let snapshot = stream.next().await.transpose()?.unwrap_or_default();
            json!({
                "status":"success",
                "addresses": addresses,
                "values": addresses.iter().map(|key| serde_json::to_value(snapshot.get(key).unwrap()).unwrap()).collect::<Vec<_>>()
            })
        }
        "remote-run" => {
            client.remote_run(false, 2).await?;
            json!({"status":"success"})
        }
        "remote-stop" => {
            client.remote_stop().await?;
            json!({"status":"success"})
        }
        "remote-pause" => {
            client.remote_pause(false).await?;
            json!({"status":"success"})
        }
        "remote-latch-clear" => {
            client.remote_latch_clear().await?;
            json!({"status":"success"})
        }
        "remote-reset" => {
            client.remote_reset(false).await?;
            json!({"status":"success"})
        }
        "random-read" => {
            let word_devices = parse_device_list(word_devs)?;
            let dword_devices = parse_device_list(dword_devs)?;
            let result = client.read_random(&word_devices, &dword_devices).await?;
            json!({"status":"success","word_values": result.word_values, "dword_values": result.dword_values})
        }
        "random-write-words" => {
            let word_entries: Vec<_> = parse_kv_pairs(words)?
                .into_iter()
                .map(|(device, value)| Ok((SlmpAddress::parse(&device)?, value as u16)))
                .collect::<Result<_, plc_comm_slmp::SlmpError>>()?;
            let dword_entries: Vec<_> = parse_kv_pairs(dwords)?
                .into_iter()
                .map(|(device, value)| Ok((SlmpAddress::parse(&device)?, value as u32)))
                .collect::<Result<_, plc_comm_slmp::SlmpError>>()?;
            client
                .write_random_words(&word_entries, &dword_entries)
                .await?;
            json!({"status":"success"})
        }
        "random-write-bits" => {
            let bit_entries: Vec<_> = parse_kv_pairs(bits)?
                .into_iter()
                .map(|(device, value)| Ok((SlmpAddress::parse(&device)?, value != 0)))
                .collect::<Result<_, plc_comm_slmp::SlmpError>>()?;
            client.write_random_bits(&bit_entries).await?;
            json!({"status":"success"})
        }
        "block-read" => {
            let word_blocks: Vec<_> = parse_dev_count_pairs(word_blocks)?
                .into_iter()
                .map(|(device, count)| {
                    Ok(SlmpBlockRead {
                        device: SlmpAddress::parse(&device)?,
                        points: count as u16,
                    })
                })
                .collect::<Result<_, plc_comm_slmp::SlmpError>>()?;
            let bit_blocks: Vec<_> = parse_dev_count_pairs(bit_blocks)?
                .into_iter()
                .map(|(device, count)| {
                    Ok(SlmpBlockRead {
                        device: SlmpAddress::parse(&device)?,
                        points: count as u16,
                    })
                })
                .collect::<Result<_, plc_comm_slmp::SlmpError>>()?;
            let result = client.read_block(&word_blocks, &bit_blocks).await?;
            json!({"status":"success","word_values": result.word_values, "bit_values": result.bit_values})
        }
        "block-write" => {
            let word_blocks: Vec<_> = parse_dev_values_pairs(word_blocks)?
                .into_iter()
                .map(|(device, values)| {
                    Ok(SlmpBlockWrite {
                        device: SlmpAddress::parse(&device)?,
                        values: values.into_iter().map(|value| value as u16).collect(),
                    })
                })
                .collect::<Result<_, plc_comm_slmp::SlmpError>>()?;
            let bit_blocks: Vec<_> = parse_dev_values_pairs(bit_blocks)?
                .into_iter()
                .map(|(device, values)| {
                    Ok(SlmpBlockWrite {
                        device: SlmpAddress::parse(&device)?,
                        values: values.into_iter().map(|value| value as u16).collect(),
                    })
                })
                .collect::<Result<_, plc_comm_slmp::SlmpError>>()?;
            client.write_block(&word_blocks, &bit_blocks, None).await?;
            json!({"status":"success"})
        }
        "self-test" => {
            let echoed = client
                .self_test_loopback(if address.is_empty() {
                    b"TEST".as_slice()
                } else {
                    address.as_bytes()
                })
                .await?;
            json!({"status":"success","echo": String::from_utf8_lossy(&echoed)})
        }
        "memory-read" => {
            let head = parse_target_auto_number(address)?;
            let count = extras
                .first()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1);
            let values = client.memory_read_words(head, count).await?;
            json!({"status":"success","values": values})
        }
        "memory-write" => {
            let head = parse_target_auto_number(address)?;
            let values: Vec<u16> = extras
                .iter()
                .map(|value| value.parse().unwrap_or(0))
                .collect();
            client.memory_write_words(head, &values).await?;
            json!({"status":"success"})
        }
        "extend-unit-read" => {
            let (module_no, head) = parse_module_head(address)?;
            let count = extras
                .first()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1);
            let values = client
                .extend_unit_read_words(head, count, module_no)
                .await?;
            json!({"status":"success","values": values})
        }
        "extend-unit-write" => {
            let (module_no, head) = parse_module_head(address)?;
            let values: Vec<u16> = extras
                .iter()
                .map(|value| value.parse().unwrap_or(0))
                .collect();
            client
                .extend_unit_write_words(head, module_no, &values)
                .await?;
            json!({"status":"success"})
        }
        "read-ext" => {
            let device = parse_qualified_device(address)?;
            let count = extras
                .first()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1);
            let extension = SlmpExtensionSpec::default();
            match mode {
                "bit" => {
                    json!({"status":"success","values": client.read_bits_extended(device, count, extension).await?.into_iter().map(u8::from).collect::<Vec<_>>() })
                }
                _ => {
                    json!({"status":"success","values": client.read_words_extended(device, count, extension).await? })
                }
            }
        }
        "write-ext" => {
            let device = parse_qualified_device(address)?;
            let extension = SlmpExtensionSpec::default();
            match mode {
                "bit" => {
                    let values: Vec<bool> = extras.iter().map(|value| value == "1").collect();
                    client
                        .write_bits_extended(device, &values, extension)
                        .await?;
                }
                _ => {
                    let values: Vec<u16> = extras
                        .iter()
                        .map(|value| value.parse().unwrap_or(0))
                        .collect();
                    client
                        .write_words_extended(device, &values, extension)
                        .await?;
                }
            }
            json!({"status":"success"})
        }
        _ => json!({"status":"error","message":"unsupported command"}),
    };

    Ok(output.to_string())
}

fn parse_kv_pairs(text: &str) -> Result<Vec<(String, i64)>, plc_comm_slmp::SlmpError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    text.split(',')
        .map(|item| {
            let (key, value) = item
                .split_once('=')
                .ok_or_else(|| plc_comm_slmp::SlmpError::new("Invalid key/value pair."))?;
            Ok((key.trim().to_string(), value.trim().parse().unwrap_or(0)))
        })
        .collect()
}

fn parse_dev_count_pairs(text: &str) -> Result<Vec<(String, i64)>, plc_comm_slmp::SlmpError> {
    parse_kv_pairs(text)
}

fn parse_dev_values_pairs(text: &str) -> Result<Vec<(String, Vec<i64>)>, plc_comm_slmp::SlmpError> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    text.split(',')
        .map(|item| {
            let (key, values) = item
                .split_once('=')
                .ok_or_else(|| plc_comm_slmp::SlmpError::new("Invalid device/values pair."))?;
            let values = values
                .split(':')
                .map(|value| value.trim().parse().unwrap_or(0))
                .collect();
            Ok((key.trim().to_string(), values))
        })
        .collect()
}

fn parse_named_addresses(text: &str) -> Vec<String> {
    if text.trim().is_empty() {
        Vec::new()
    } else {
        text.split(',')
            .map(|value| value.trim().to_string())
            .collect()
    }
}

fn parse_named_updates(
    text: &str,
    plc_family: SlmpPlcFamily,
) -> Result<NamedAddress, plc_comm_slmp::SlmpError> {
    let mut updates = BTreeMap::new();
    for item in parse_named_addresses(text) {
        let (key, value) = item
            .split_once('=')
            .ok_or_else(|| plc_comm_slmp::SlmpError::new("Invalid named update."))?;
        updates.insert(
            key.trim().to_string(),
            parse_scalar_for_named_with_family(key.trim(), value.trim(), Some(plc_family))?,
        );
    }
    Ok(updates)
}

fn parse_device_list(
    text: &str,
) -> Result<Vec<plc_comm_slmp::SlmpDeviceAddress>, plc_comm_slmp::SlmpError> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    text.split(',')
        .map(|value| SlmpAddress::parse(value.trim()))
        .collect()
}

fn parse_module_head(text: &str) -> Result<(u16, u32), plc_comm_slmp::SlmpError> {
    let parts: Vec<_> = text.split(':').collect();
    let module_no = parse_target_auto_number(parts[0])? as u16;
    let head = if parts.len() > 1 {
        parse_target_auto_number(parts[1])?
    } else {
        0
    };
    Ok((module_no, head))
}
