use plc_comm_slmp::{
    SlmpAddress, SlmpBlockRead, SlmpClient, SlmpCompatibilityMode, SlmpConnectionOptions,
    SlmpDeviceAddress, SlmpFrameType, encode_device_spec, normalize_named_address,
    parse_named_address,
};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn spec_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../plc-comm-slmp-cross-verify/specs/shared")
}

fn load_json(name: &str) -> Value {
    let path = spec_dir().join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn device_spec_vectors_match_shared_json() {
    let doc = load_json("device_spec_vectors.json");
    for vector in doc["vectors"].as_array().unwrap() {
        let device = SlmpAddress::parse(vector["device"].as_str().unwrap()).unwrap();
        let mode = match vector["series"].as_str().unwrap() {
            "legacy" => SlmpCompatibilityMode::Legacy,
            "iqr" => SlmpCompatibilityMode::Iqr,
            other => panic!("unsupported mode {other}"),
        };
        let actual = hex_upper(&encode_device_spec(mode, device));
        assert_eq!(
            actual,
            vector["hex"].as_str().unwrap(),
            "case {}",
            vector["id"].as_str().unwrap()
        );
    }
}

#[test]
fn shared_address_parse_vectors_match() {
    let doc = load_json("high_level_address_parse_vectors.json");
    for case in doc["cases"].as_array().unwrap() {
        let parsed = parse_named_address(case["input"].as_str().unwrap()).unwrap();
        let expected = &case["expected"];
        assert_eq!(
            parsed.base,
            expected["base"].as_str().unwrap(),
            "case {}",
            case["id"].as_str().unwrap()
        );
        assert_eq!(
            parsed.dtype,
            expected["dtype"].as_str().unwrap(),
            "case {}",
            case["id"].as_str().unwrap()
        );
        let actual_bit = parsed.bit_index.map(i64::from);
        let expected_bit = expected["bit_index"].as_i64();
        assert_eq!(
            actual_bit,
            expected_bit,
            "case {}",
            case["id"].as_str().unwrap()
        );
    }
}

#[test]
fn shared_address_normalize_vectors_match() {
    let doc = load_json("high_level_address_normalize_vectors.json");
    for case in doc["cases"].as_array().unwrap() {
        let actual = normalize_named_address(case["input"].as_str().unwrap()).unwrap();
        assert_eq!(
            actual,
            case["expected"].as_str().unwrap(),
            "case {}",
            case["id"].as_str().unwrap()
        );
    }
}

#[tokio::test]
async fn frame_golden_vectors_match_shared_json() {
    let doc = load_json("frame_golden_vectors.json");
    for case in doc["cases"].as_array().unwrap() {
        let response_data = hex_decode(case["response_data_hex"].as_str().unwrap());
        let server = SingleShotServer::start(response_data).await.unwrap();

        let mut options = SlmpConnectionOptions::new("127.0.0.1");
        options.port = server.port;
        options.frame_type = SlmpFrameType::Frame4E;
        options.compatibility_mode = SlmpCompatibilityMode::Iqr;
        let client = SlmpClient::connect(options).await.unwrap();

        dispatch_case(&client, case).await.unwrap();
        let actual = hex_upper(&server.take_request().await.unwrap());
        assert_eq!(
            actual,
            case["request_hex"].as_str().unwrap(),
            "case {}",
            case["id"].as_str().unwrap()
        );
    }
}

async fn dispatch_case(client: &SlmpClient, case: &Value) -> Result<(), plc_comm_slmp::SlmpError> {
    match case["operation"].as_str().unwrap() {
        "read_type_name" => {
            let info = client.read_type_name().await?;
            assert_eq!(info.model, "Q03UDVCPU");
        }
        "read_words" => {
            let args = &case["args"];
            let device = SlmpAddress::parse(args["device"].as_str().unwrap())?;
            let points = args["points"].as_u64().unwrap() as u16;
            let values = client.read_words_raw(device, points).await?;
            assert!(!values.is_empty());
        }
        "write_bits" => {
            let args = &case["args"];
            let device = SlmpAddress::parse(args["device"].as_str().unwrap())?;
            let values: Vec<bool> = args["values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_bool().unwrap())
                .collect();
            client.write_bits(device, &values).await?;
        }
        "read_random" => {
            let args = &case["args"];
            let words: Vec<SlmpDeviceAddress> = args["word_devices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| SlmpAddress::parse(value.as_str().unwrap()).unwrap())
                .collect();
            let dwords: Vec<SlmpDeviceAddress> = args["dword_devices"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| SlmpAddress::parse(value.as_str().unwrap()).unwrap())
                .collect();
            let result = client.read_random(&words, &dwords).await?;
            assert!(!result.word_values.is_empty());
        }
        "write_random_bits" => {
            let args = &case["args"];
            let entries: Vec<(SlmpDeviceAddress, bool)> = args["bit_values"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    let device = SlmpAddress::parse(item["device"].as_str().unwrap()).unwrap();
                    let value = item["value"].as_bool().unwrap();
                    (device, value)
                })
                .collect();
            client.write_random_bits(&entries).await?;
        }
        "read_block" => {
            let args = &case["args"];
            let word_blocks: Vec<SlmpBlockRead> = args["word_blocks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| SlmpBlockRead {
                    device: SlmpAddress::parse(item["device"].as_str().unwrap()).unwrap(),
                    points: item["points"].as_u64().unwrap() as u16,
                })
                .collect();
            let bit_blocks: Vec<SlmpBlockRead> = args["bit_blocks"]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| SlmpBlockRead {
                    device: SlmpAddress::parse(item["device"].as_str().unwrap()).unwrap(),
                    points: item["points"].as_u64().unwrap() as u16,
                })
                .collect();
            let result = client.read_block(&word_blocks, &bit_blocks).await?;
            assert!(!result.word_values.is_empty() || !result.bit_values.is_empty());
        }
        "remote_password_unlock" => {
            let args = &case["args"];
            client
                .remote_password_unlock(args["password"].as_str().unwrap())
                .await?;
        }
        other => panic!("unsupported operation {other}"),
    }
    Ok(())
}

fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

fn hex_decode(value: &str) -> Vec<u8> {
    if value.is_empty() {
        return Vec::new();
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).unwrap())
        .collect()
}

struct SingleShotServer {
    port: u16,
    request: std::sync::Arc<tokio::sync::Mutex<Option<Vec<u8>>>>,
}

impl SingleShotServer {
    async fn start(response_data: Vec<u8>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let request = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let request_clone = request.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut header = [0u8; 19];
                if stream.read_exact(&mut header[0..2]).await.is_err() {
                    return;
                }
                if header[0] == 0x54 && header[1] == 0x00 {
                    if stream.read_exact(&mut header[2..19]).await.is_err() {
                        return;
                    }
                    let payload_len = u16::from_le_bytes([header[11], header[12]]) as usize - 6;
                    let mut payload = vec![0u8; payload_len];
                    if stream.read_exact(&mut payload).await.is_err() {
                        return;
                    }
                    let mut request_frame = header.to_vec();
                    request_frame.extend_from_slice(&payload);
                    *request_clone.lock().await = Some(request_frame.clone());

                    let mut response = vec![0u8; 13 + 2 + response_data.len()];
                    response[0] = 0xD4;
                    response[1] = 0x00;
                    response[2] = header[2];
                    response[3] = header[3];
                    response[6] = header[6];
                    response[7] = header[7];
                    response[8] = header[8];
                    response[9] = header[9];
                    response[10] = header[10];
                    let len = 2 + response_data.len() as u16;
                    response[11..13].copy_from_slice(&len.to_le_bytes());
                    response[13..15].copy_from_slice(&0u16.to_le_bytes());
                    response[15..].copy_from_slice(&response_data);
                    let _ = stream.write_all(&response).await;
                }
            }
        });
        Ok(Self { port, request })
    }

    async fn take_request(&self) -> Option<Vec<u8>> {
        self.request.lock().await.take()
    }
}
