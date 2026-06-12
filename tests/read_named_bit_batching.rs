use plc_comm_slmp::{
    SlmpClient, SlmpCommand, SlmpCompatibilityMode, SlmpConnectionOptions, SlmpDeviceAddress,
    SlmpDeviceCode, SlmpFrameType, SlmpPlcFamily, SlmpValue, read_named,
};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn read_named_batches_plain_bits_that_share_one_word() {
    let server = CapturingServer::start(vec![word_payload(&[0b1000_0000_0111_0000])])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcFamily::IqR).await;
    let addresses = strings(&["M100", "M101", "M102", "M111"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["M100"], SlmpValue::Bool(true));
    assert_eq!(values["M101"], SlmpValue::Bool(true));
    assert_eq!(values["M102"], SlmpValue::Bool(true));
    assert_eq!(values["M111"], SlmpValue::Bool(true));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[SlmpDeviceAddress::new(SlmpDeviceCode::M, 96)],
        &[],
    );
}

#[tokio::test]
async fn read_named_preserves_plain_bit_values_for_representative_word_patterns() {
    for pattern in [0x0000, 0xFFFF, 0xAAAA, 0x5A3C, 0xC001] {
        let server = CapturingServer::start(vec![word_payload(&[pattern])])
            .await
            .unwrap();
        let client = connect_client(server.port, SlmpPlcFamily::IqR).await;
        let addresses = strings(&["M100", "M101", "M102", "M103", "M110", "M111"]);

        let values = read_named(&client, &addresses).await.unwrap();

        for (address, bit_index) in addresses.iter().zip([4u8, 5, 6, 7, 14, 15]) {
            assert_eq!(
                values[address],
                SlmpValue::Bool(((pattern >> bit_index) & 1) != 0),
                "pattern=0x{pattern:04X} address={address}"
            );
        }

        let requests = server.requests().await;
        assert_eq!(requests.len(), 1);
        assert_random_shape(
            &requests[0],
            &[SlmpDeviceAddress::new(SlmpDeviceCode::M, 96)],
            &[],
        );
    }
}

#[tokio::test]
async fn read_named_batches_plain_bits_across_words_and_iqf_octal_xy_boundaries() {
    let server = CapturingServer::start(vec![word_payload(&[0x8000, 0x0001])])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcFamily::IqF).await;
    let addresses = strings(&["X17", "X20"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["X17"], SlmpValue::Bool(true));
    assert_eq!(values["X20"], SlmpValue::Bool(true));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 16),
        ],
        &[],
    );
}

#[tokio::test]
async fn read_named_batches_mixed_plain_bit_device_kinds() {
    let server = CapturingServer::start(vec![word_payload(&[0x0010, 0x8000, 0x0004])])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcFamily::IqR).await;
    let addresses = strings(&["M100", "B1F", "SB2"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["M100"], SlmpValue::Bool(true));
    assert_eq!(values["B1F"], SlmpValue::Bool(true));
    assert_eq!(values["SB2"], SlmpValue::Bool(true));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[
            SlmpDeviceAddress::new(SlmpDeviceCode::M, 96),
            SlmpDeviceAddress::new(SlmpDeviceCode::B, 0x10),
            SlmpDeviceAddress::new(SlmpDeviceCode::SB, 0),
        ],
        &[],
    );
}

#[tokio::test]
async fn read_named_batches_each_supported_plain_bit_device_code() {
    let addresses = strings(&[
        "SM17", "X1", "Y1", "M17", "L17", "F17", "V17", "B1F", "TS17", "TC17", "STS17", "STC17",
        "CS17", "CC17", "SB1F", "DX1", "DY1",
    ]);
    let expected_devices = [
        SlmpDeviceAddress::new(SlmpDeviceCode::SM, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::X, 0),
        SlmpDeviceAddress::new(SlmpDeviceCode::Y, 0),
        SlmpDeviceAddress::new(SlmpDeviceCode::M, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::L, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::F, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::V, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::B, 0x10),
        SlmpDeviceAddress::new(SlmpDeviceCode::TS, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::TC, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::STS, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::STC, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::CS, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::CC, 16),
        SlmpDeviceAddress::new(SlmpDeviceCode::SB, 0x10),
        SlmpDeviceAddress::new(SlmpDeviceCode::DX, 0),
        SlmpDeviceAddress::new(SlmpDeviceCode::DY, 0),
    ];
    let mut words = vec![0x0002; addresses.len()];
    words[7] = 0x8000;
    words[14] = 0x8000;
    let server = CapturingServer::start(vec![word_payload(&words)])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcFamily::IqR).await;

    let values = read_named(&client, &addresses).await.unwrap();

    assert!(
        addresses
            .iter()
            .all(|address| values[address] == SlmpValue::Bool(true))
    );

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(&requests[0], &expected_devices, &[]);
}

#[tokio::test]
async fn read_named_chunks_more_than_96_batched_bit_words() {
    let first_chunk = vec![0x0001; 96];
    let second_chunk = vec![0x0001; 96];
    let third_chunk = vec![0x0001; 64];
    let server = CapturingServer::start(vec![
        word_payload(&first_chunk),
        word_payload(&second_chunk),
        word_payload(&third_chunk),
    ])
    .await
    .unwrap();
    let client = connect_client(server.port, SlmpPlcFamily::IqR).await;
    let addresses: Vec<String> = (0..0x100).map(|index| format!("M{}", index * 16)).collect();

    let values = read_named(&client, &addresses).await.unwrap();

    assert!(
        addresses
            .iter()
            .all(|address| values[address] == SlmpValue::Bool(true))
    );

    let requests = server.requests().await;
    assert_eq!(requests.len(), 3);
    assert_random_counts(&requests[0], 96, 0);
    assert_random_counts(&requests[1], 96, 0);
    assert_random_counts(&requests[2], 64, 0);
}

#[tokio::test]
async fn read_named_keeps_long_counter_state_bits_on_direct_bit_fallback() {
    let server = CapturingServer::start(vec![vec![0x10]]).await.unwrap();
    let client = connect_client(server.port, SlmpPlcFamily::IqR).await;
    let addresses = strings(&["LCS30"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["LCS30"], SlmpValue::Bool(true));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_direct_bit_read(
        &requests[0],
        SlmpDeviceAddress::new(SlmpDeviceCode::LCS, 30),
        1,
    );
}

#[tokio::test]
async fn read_named_mixes_plain_bits_bit_in_word_words_and_dwords_in_one_random_read() {
    let server = CapturingServer::start(vec![random_payload(
        &[0x0030, 0x0008, 0x1234],
        &[0x3F80_0000],
    )])
    .await
    .unwrap();
    let client = connect_client(server.port, SlmpPlcFamily::IqR).await;
    let addresses = strings(&["M100", "M101", "D50.3", "D51", "D52:F"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["M100"], SlmpValue::Bool(true));
    assert_eq!(values["M101"], SlmpValue::Bool(true));
    assert_eq!(values["D50.3"], SlmpValue::Bool(true));
    assert_eq!(values["D51"], SlmpValue::U16(0x1234));
    assert_eq!(values["D52:F"], SlmpValue::F32(1.0));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[
            SlmpDeviceAddress::new(SlmpDeviceCode::M, 96),
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 50),
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 51),
        ],
        &[SlmpDeviceAddress::new(SlmpDeviceCode::D, 52)],
    );
}

struct CapturingServer {
    port: u16,
    requests: Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl CapturingServer {
    async fn start(response_payloads: Vec<Vec<u8>>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_sink = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut pending = VecDeque::from(response_payloads);
                while let Some(payload) = pending.pop_front() {
                    let mut header = [0u8; 13];
                    if stream.read_exact(&mut header).await.is_err() {
                        return;
                    }
                    let body_len = u16::from_le_bytes([header[11], header[12]]) as usize;
                    let mut body = vec![0u8; body_len];
                    if stream.read_exact(&mut body).await.is_err() {
                        return;
                    }
                    let mut request = header.to_vec();
                    request.extend_from_slice(&body);
                    request_sink.lock().await.push(request.clone());

                    let response = build_4e_response(&request, &payload);
                    if stream.write_all(&response).await.is_err() {
                        return;
                    }
                }
            }
        });
        Ok(Self { port, requests })
    }

    async fn requests(&self) -> Vec<Vec<u8>> {
        self.requests.lock().await.clone()
    }
}

async fn connect_client(port: u16, plc_family: SlmpPlcFamily) -> SlmpClient {
    let mut options = SlmpConnectionOptions::new("127.0.0.1", plc_family);
    options.port = port;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;
    SlmpClient::connect(options).await.unwrap()
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn word_payload(values: &[u16]) -> Vec<u8> {
    random_payload(values, &[])
}

fn random_payload(word_values: &[u16], dword_values: &[u32]) -> Vec<u8> {
    let mut payload = Vec::with_capacity((word_values.len() * 2) + (dword_values.len() * 4));
    for value in word_values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    for value in dword_values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload
}

fn build_4e_response(request: &[u8], response_data: &[u8]) -> Vec<u8> {
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[2..].copy_from_slice(response_data);

    let mut response = vec![0u8; 13 + payload.len()];
    response[0] = 0xD4;
    response[1] = 0x00;
    response[2] = request[2];
    response[3] = request[3];
    response[6..11].copy_from_slice(&request[6..11]);
    response[11..13].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    response[13..].copy_from_slice(&payload);
    response
}

fn assert_random_counts(request: &[u8], word_count: u8, dword_count: u8) {
    let body = &request[13..];
    assert_eq!(
        u16::from_le_bytes([body[2], body[3]]),
        SlmpCommand::DeviceReadRandom.as_u16()
    );
    assert_eq!(u16::from_le_bytes([body[4], body[5]]), 0x0002);
    assert_eq!(body[6], word_count);
    assert_eq!(body[7], dword_count);
}

fn assert_random_shape(
    request: &[u8],
    word_devices: &[SlmpDeviceAddress],
    dword_devices: &[SlmpDeviceAddress],
) {
    assert_random_counts(request, word_devices.len() as u8, dword_devices.len() as u8);
    let mut offset = 21;
    for device in word_devices.iter().chain(dword_devices.iter()) {
        assert_eq!(decode_iqr_device(&request[offset..offset + 6]), *device);
        offset += 6;
    }
}

fn assert_direct_bit_read(request: &[u8], device: SlmpDeviceAddress, points: u16) {
    let body = &request[13..];
    assert_eq!(
        u16::from_le_bytes([body[2], body[3]]),
        SlmpCommand::DeviceRead.as_u16()
    );
    assert_eq!(u16::from_le_bytes([body[4], body[5]]), 0x0003);
    assert_eq!(decode_iqr_device(&request[19..25]), device);
    assert_eq!(u16::from_le_bytes([request[25], request[26]]), points);
}

fn decode_iqr_device(bytes: &[u8]) -> SlmpDeviceAddress {
    let number = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let code = u16::from_le_bytes([bytes[4], bytes[5]]);
    let code = match code {
        value if value == SlmpDeviceCode::M.as_u16() => SlmpDeviceCode::M,
        value if value == SlmpDeviceCode::SM.as_u16() => SlmpDeviceCode::SM,
        value if value == SlmpDeviceCode::X.as_u16() => SlmpDeviceCode::X,
        value if value == SlmpDeviceCode::Y.as_u16() => SlmpDeviceCode::Y,
        value if value == SlmpDeviceCode::L.as_u16() => SlmpDeviceCode::L,
        value if value == SlmpDeviceCode::F.as_u16() => SlmpDeviceCode::F,
        value if value == SlmpDeviceCode::V.as_u16() => SlmpDeviceCode::V,
        value if value == SlmpDeviceCode::B.as_u16() => SlmpDeviceCode::B,
        value if value == SlmpDeviceCode::TS.as_u16() => SlmpDeviceCode::TS,
        value if value == SlmpDeviceCode::TC.as_u16() => SlmpDeviceCode::TC,
        value if value == SlmpDeviceCode::STS.as_u16() => SlmpDeviceCode::STS,
        value if value == SlmpDeviceCode::STC.as_u16() => SlmpDeviceCode::STC,
        value if value == SlmpDeviceCode::CS.as_u16() => SlmpDeviceCode::CS,
        value if value == SlmpDeviceCode::CC.as_u16() => SlmpDeviceCode::CC,
        value if value == SlmpDeviceCode::SB.as_u16() => SlmpDeviceCode::SB,
        value if value == SlmpDeviceCode::DX.as_u16() => SlmpDeviceCode::DX,
        value if value == SlmpDeviceCode::DY.as_u16() => SlmpDeviceCode::DY,
        value if value == SlmpDeviceCode::D.as_u16() => SlmpDeviceCode::D,
        value if value == SlmpDeviceCode::LCS.as_u16() => SlmpDeviceCode::LCS,
        other => panic!("unexpected device code 0x{other:04X}"),
    };
    SlmpDeviceAddress::new(code, number)
}
