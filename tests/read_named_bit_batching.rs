use plc_comm_slmp::{
    SlmpClient, SlmpCommand, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode,
    SlmpPlcProfile, SlmpValue, read_named,
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
    let client = connect_client(server.port, SlmpPlcProfile::IqR).await;
    let addresses = strings(&["M100:BIT", "M101:BIT", "M102:BIT", "M111:BIT"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["M100:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["M101:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["M102:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["M111:BIT"], SlmpValue::Bool(true));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[SlmpDeviceAddress::new(
            SlmpDeviceCode::M,
            96,
            SlmpPlcProfile::IqR,
        )],
        &[],
    );
}

#[tokio::test]
async fn read_named_preserves_plain_bit_values_for_representative_word_patterns() {
    for pattern in [0x0000, 0xFFFF, 0xAAAA, 0x5A3C, 0xC001] {
        let server = CapturingServer::start(vec![word_payload(&[pattern])])
            .await
            .unwrap();
        let client = connect_client(server.port, SlmpPlcProfile::IqR).await;
        let addresses = strings(&[
            "M100:BIT", "M101:BIT", "M102:BIT", "M103:BIT", "M110:BIT", "M111:BIT",
        ]);

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
            &[SlmpDeviceAddress::new(
                SlmpDeviceCode::M,
                96,
                SlmpPlcProfile::IqR,
            )],
            &[],
        );
    }
}

#[tokio::test]
async fn read_named_batches_plain_bits_across_words_and_iqf_octal_xy_boundaries() {
    let server = CapturingServer::start(vec![word_payload(&[0x8000, 0x0001])])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::IqF).await;
    let addresses = strings(&["X17:BIT", "X20:BIT"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["X17:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["X20:BIT"], SlmpValue::Bool(true));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 0, SlmpPlcProfile::IqR),
            SlmpDeviceAddress::new(SlmpDeviceCode::X, 16, SlmpPlcProfile::IqR),
        ],
        &[],
    );
}

#[tokio::test]
async fn read_named_batches_mixed_plain_bit_device_kinds() {
    let server = CapturingServer::start(vec![word_payload(&[0x0010, 0x8000, 0x0004])])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::IqR).await;
    let addresses = strings(&["M100:BIT", "B1F:BIT", "SB2:BIT"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["M100:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["B1F:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["SB2:BIT"], SlmpValue::Bool(true));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[
            SlmpDeviceAddress::new(SlmpDeviceCode::M, 96, SlmpPlcProfile::IqR),
            SlmpDeviceAddress::new(SlmpDeviceCode::B, 0x10, SlmpPlcProfile::IqR),
            SlmpDeviceAddress::new(SlmpDeviceCode::SB, 0, SlmpPlcProfile::IqR),
        ],
        &[],
    );
}

#[tokio::test]
async fn read_named_batches_each_supported_plain_bit_device_code() {
    let addresses = strings(&[
        "SM17:BIT", "X1:BIT", "Y1:BIT", "M17:BIT", "L17:BIT", "F17:BIT", "V17:BIT", "B1F:BIT",
        "SB1F:BIT",
    ]);
    let expected_devices = [
        SlmpDeviceAddress::new(SlmpDeviceCode::SM, 16, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::X, 0, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::Y, 0, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::M, 16, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::L, 16, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::F, 16, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::V, 16, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::B, 0x10, SlmpPlcProfile::IqR),
        SlmpDeviceAddress::new(SlmpDeviceCode::SB, 0x10, SlmpPlcProfile::IqR),
    ];
    let mut words = vec![0x0002; addresses.len()];
    words[7] = 0x8000;
    words[8] = 0x8000;
    let server = CapturingServer::start(vec![word_payload(&words)])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::IqR).await;

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
async fn read_named_rejects_direct_bit_fallback_routes_before_transport() {
    let addresses = strings(&[
        "TS10:BIT",
        "TC10:BIT",
        "STS10:BIT",
        "STC10:BIT",
        "CS10:BIT",
        "CC10:BIT",
        "DX10:BIT",
        "DY10:BIT",
    ]);
    let server = CapturingServer::start(vec![]).await.unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::IqR).await;

    let error = read_named(&client, &addresses).await.unwrap_err();

    let requests = server.requests().await;
    assert!(error.message.contains("one random-read request"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_named_rejects_random_read_over_limit_without_chunking() {
    let server = CapturingServer::start(vec![]).await.unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::IqR).await;
    let addresses: Vec<String> = (0..97)
        .map(|index| format!("M{}:BIT", index * 16))
        .collect();

    let error = read_named(&client, &addresses).await.unwrap_err();

    let requests = server.requests().await;
    assert!(error.message.contains("out of range"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_named_uses_profile_random_read_limit_for_ql_profiles() {
    let word_count = 160usize;
    let server = CapturingServer::start(vec![
        word_payload(&vec![0x0001; word_count]),
        word_payload(&vec![0x0001; word_count - 96]),
    ])
    .await
    .unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::QnUDV).await;
    let addresses: Vec<String> = (0..word_count)
        .map(|index| format!("M{}:BIT", index * 16))
        .collect();

    let values = read_named(&client, &addresses).await.unwrap();

    assert!(
        addresses
            .iter()
            .all(|address| values[address] == SlmpValue::Bool(true))
    );

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_counts(&requests[0], word_count as u8, 0);
}

#[tokio::test]
async fn read_named_rejects_long_counter_state_direct_bit_fallback() {
    let server = CapturingServer::start(vec![]).await.unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::IqR).await;
    let addresses = strings(&["LCS30:BIT"]);

    let error = read_named(&client, &addresses).await.unwrap_err();

    let requests = server.requests().await;
    assert!(error.message.contains("one random-read request"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_named_mixes_plain_bits_bit_in_word_words_and_dwords_in_one_random_read() {
    let server = CapturingServer::start(vec![random_payload(
        &[0x0030, 0x0008, 0x1234],
        &[0x3F80_0000],
    )])
    .await
    .unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::IqR).await;
    let addresses = strings(&["M100:BIT", "M101:BIT", "D50.3", "D51:U", "D52:F"]);

    let values = read_named(&client, &addresses).await.unwrap();

    assert_eq!(values["M100:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["M101:BIT"], SlmpValue::Bool(true));
    assert_eq!(values["D50.3"], SlmpValue::Bool(true));
    assert_eq!(values["D51:U"], SlmpValue::U16(0x1234));
    assert_eq!(values["D52:F"], SlmpValue::F32(1.0));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_random_shape(
        &requests[0],
        &[
            SlmpDeviceAddress::new(SlmpDeviceCode::M, 96, SlmpPlcProfile::IqR),
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 50, SlmpPlcProfile::IqR),
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 51, SlmpPlcProfile::IqR),
        ],
        &[SlmpDeviceAddress::new(
            SlmpDeviceCode::D,
            52,
            SlmpPlcProfile::IqR,
        )],
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
                    let Some(request) = read_request(&mut stream).await else {
                        return;
                    };
                    request_sink.lock().await.push(request.clone());

                    let response = build_response(&request, &payload);
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

async fn read_request(stream: &mut tokio::net::TcpStream) -> Option<Vec<u8>> {
    let mut prefix = [0u8; 2];
    stream.read_exact(&mut prefix).await.ok()?;

    let (header_size, length_index) = match prefix {
        [0x54, 0x00] => (13usize, 11usize),
        [0x50, 0x00] => (9usize, 7usize),
        _ => return None,
    };

    let mut request = vec![0u8; header_size];
    request[0..2].copy_from_slice(&prefix);
    stream.read_exact(&mut request[2..header_size]).await.ok()?;
    let body_len = u16::from_le_bytes([request[length_index], request[length_index + 1]]) as usize;
    request.resize(header_size + body_len, 0);
    stream.read_exact(&mut request[header_size..]).await.ok()?;
    Some(request)
}

async fn connect_client(port: u16, plc_profile: SlmpPlcProfile) -> SlmpClient {
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        plc_profile,
    )
    .unwrap();
    options.port = port;
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

fn build_response(request: &[u8], response_data: &[u8]) -> Vec<u8> {
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[2..].copy_from_slice(response_data);

    if request.starts_with(&[0x50, 0x00]) {
        let mut response = vec![0u8; 9 + payload.len()];
        response[0] = 0xD0;
        response[1] = 0x00;
        response[2..7].copy_from_slice(&request[2..7]);
        response[7..9].copy_from_slice(&(payload.len() as u16).to_le_bytes());
        response[9..].copy_from_slice(&payload);
        return response;
    }

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

fn header_size(request: &[u8]) -> usize {
    if request.starts_with(&[0x50, 0x00]) {
        9
    } else {
        13
    }
}

fn is_iqr_request(request: &[u8]) -> bool {
    request.starts_with(&[0x54, 0x00])
}

fn assert_random_counts(request: &[u8], word_count: u8, dword_count: u8) {
    let body = &request[header_size(request)..];
    assert_eq!(
        u16::from_le_bytes([body[2], body[3]]),
        SlmpCommand::DeviceReadRandom.as_u16()
    );
    let expected_subcommand = if is_iqr_request(request) {
        0x0002
    } else {
        0x0000
    };
    assert_eq!(u16::from_le_bytes([body[4], body[5]]), expected_subcommand);
    assert_eq!(body[6], word_count);
    assert_eq!(body[7], dword_count);
}

fn assert_random_shape(
    request: &[u8],
    word_devices: &[SlmpDeviceAddress],
    dword_devices: &[SlmpDeviceAddress],
) {
    assert_random_counts(request, word_devices.len() as u8, dword_devices.len() as u8);
    let spec_size = if is_iqr_request(request) { 6 } else { 4 };
    let mut offset = header_size(request) + 8;
    for device in word_devices.iter().chain(dword_devices.iter()) {
        let actual = if is_iqr_request(request) {
            decode_iqr_device(&request[offset..offset + spec_size])
        } else {
            decode_legacy_device(&request[offset..offset + spec_size])
        };
        assert_eq!(actual, *device);
        offset += spec_size;
    }
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
    SlmpDeviceAddress::new(code, number, SlmpPlcProfile::IqR)
}

fn decode_legacy_device(bytes: &[u8]) -> SlmpDeviceAddress {
    let number = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]);
    let code = match u16::from(bytes[3]) {
        value if value == SlmpDeviceCode::M.as_u8() as u16 => SlmpDeviceCode::M,
        value if value == SlmpDeviceCode::SM.as_u8() as u16 => SlmpDeviceCode::SM,
        value if value == SlmpDeviceCode::X.as_u8() as u16 => SlmpDeviceCode::X,
        value if value == SlmpDeviceCode::Y.as_u8() as u16 => SlmpDeviceCode::Y,
        value if value == SlmpDeviceCode::L.as_u8() as u16 => SlmpDeviceCode::L,
        value if value == SlmpDeviceCode::F.as_u8() as u16 => SlmpDeviceCode::F,
        value if value == SlmpDeviceCode::V.as_u8() as u16 => SlmpDeviceCode::V,
        value if value == SlmpDeviceCode::B.as_u8() as u16 => SlmpDeviceCode::B,
        value if value == SlmpDeviceCode::TS.as_u8() as u16 => SlmpDeviceCode::TS,
        value if value == SlmpDeviceCode::TC.as_u8() as u16 => SlmpDeviceCode::TC,
        value if value == SlmpDeviceCode::STS.as_u8() as u16 => SlmpDeviceCode::STS,
        value if value == SlmpDeviceCode::STC.as_u8() as u16 => SlmpDeviceCode::STC,
        value if value == SlmpDeviceCode::CS.as_u8() as u16 => SlmpDeviceCode::CS,
        value if value == SlmpDeviceCode::CC.as_u8() as u16 => SlmpDeviceCode::CC,
        value if value == SlmpDeviceCode::SB.as_u8() as u16 => SlmpDeviceCode::SB,
        value if value == SlmpDeviceCode::DX.as_u8() as u16 => SlmpDeviceCode::DX,
        value if value == SlmpDeviceCode::DY.as_u8() as u16 => SlmpDeviceCode::DY,
        value if value == SlmpDeviceCode::D.as_u8() as u16 => SlmpDeviceCode::D,
        value if value == SlmpDeviceCode::LCS.as_u8() as u16 => SlmpDeviceCode::LCS,
        other => panic!("unexpected legacy device code 0x{other:02X}"),
    };
    SlmpDeviceAddress::new(code, number, SlmpPlcProfile::IqR)
}
