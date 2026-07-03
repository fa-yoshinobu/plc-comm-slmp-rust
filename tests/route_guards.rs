use plc_comm_slmp::{
    SlmpBlockRead, SlmpBlockWrite, SlmpBlockWriteOptions, SlmpClient, SlmpCommand,
    SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpErrorKind, SlmpExtensionSpec,
    SlmpPlcProfile, SlmpQualifiedDeviceAddress, SlmpTransportMode, SlmpValue,
    parse_qualified_device, read_dwords_chunked, read_dwords_single_request, read_named,
    read_typed, write_typed,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};

async fn udp_client() -> SlmpClient {
    udp_client_with_profile(SlmpPlcProfile::IqR).await
}

async fn udp_client_with_profile(plc_profile: SlmpPlcProfile) -> SlmpClient {
    udp_client_with_profile_and_strict(plc_profile, true).await
}

async fn udp_client_with_profile_and_strict(
    plc_profile: SlmpPlcProfile,
    strict_profile: bool,
) -> SlmpClient {
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.set_plc_profile(plc_profile);
    options.transport_mode = SlmpTransportMode::Udp;
    options.port = 9;
    options.strict_profile = strict_profile;
    SlmpClient::connect(options).await.unwrap()
}

struct MultiResponseServer {
    port: u16,
}

impl MultiResponseServer {
    async fn start(response_payloads: Vec<Vec<u8>>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut pending = std::collections::VecDeque::from(response_payloads);
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
                    let response = build_4e_response(&request, &payload);
                    if stream.write_all(&response).await.is_err() {
                        return;
                    }
                }
            }
        });
        Ok(Self { port })
    }
}

struct CapturingResponseServer {
    port: u16,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl CapturingResponseServer {
    async fn start(responses: Vec<(u16, Vec<u8>)>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_clone = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut pending = std::collections::VecDeque::from(responses);
                while let Some((end_code, payload)) = pending.pop_front() {
                    let Some(request) = read_slmp_request_frame(&mut stream).await else {
                        return;
                    };
                    request_clone.lock().await.push(request.clone());
                    let response = build_response_with_end_code(&request, end_code, &payload);
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

struct SerialSkewResponseServer {
    port: u16,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl SerialSkewResponseServer {
    async fn start(stale_payload: Vec<u8>, matching_payload: Vec<u8>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_clone = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
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
                request_clone.lock().await.push(request.clone());

                let request_serial = u16::from_le_bytes([request[2], request[3]]);
                let stale = build_4e_response_with_serial(
                    &request,
                    request_serial.wrapping_add(1),
                    0,
                    &stale_payload,
                );
                let matching =
                    build_4e_response_with_serial(&request, request_serial, 0, &matching_payload);
                if stream.write_all(&stale).await.is_err() {
                    return;
                }
                let _ = stream.write_all(&matching).await;
            }
        });
        Ok(Self { port, requests })
    }

    async fn requests(&self) -> Vec<Vec<u8>> {
        self.requests.lock().await.clone()
    }
}

async fn read_slmp_request_frame(stream: &mut tokio::net::TcpStream) -> Option<Vec<u8>> {
    let mut prefix = [0u8; 2];
    stream.read_exact(&mut prefix).await.ok()?;
    let (prefix_len, length_index) = match prefix {
        [0x54, 0x00] => (13usize, 11usize),
        [0x50, 0x00] => (9usize, 7usize),
        _ => return None,
    };
    let mut request = vec![0u8; prefix_len];
    request[0..2].copy_from_slice(&prefix);
    stream.read_exact(&mut request[2..prefix_len]).await.ok()?;
    let body_len = u16::from_le_bytes([request[length_index], request[length_index + 1]]) as usize;
    let mut body = vec![0u8; body_len];
    stream.read_exact(&mut body).await.ok()?;
    request.extend_from_slice(&body);
    Some(request)
}

fn build_4e_response(request: &[u8], response_data: &[u8]) -> Vec<u8> {
    build_4e_response_with_end_code(request, 0, response_data)
}

fn build_response_with_end_code(request: &[u8], end_code: u16, response_data: &[u8]) -> Vec<u8> {
    if request.starts_with(&[0x50, 0x00]) {
        return build_3e_response_with_end_code(request, end_code, response_data);
    }
    build_4e_response_with_end_code(request, end_code, response_data)
}

fn build_3e_response_with_end_code(request: &[u8], end_code: u16, response_data: &[u8]) -> Vec<u8> {
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[0..2].copy_from_slice(&end_code.to_le_bytes());
    payload[2..].copy_from_slice(response_data);

    let mut response = vec![0u8; 9 + payload.len()];
    response[0] = 0xD0;
    response[1] = 0x00;
    response[2..7].copy_from_slice(&request[2..7]);
    response[7..9].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    response[9..].copy_from_slice(&payload);
    response
}

fn build_4e_response_with_end_code(request: &[u8], end_code: u16, response_data: &[u8]) -> Vec<u8> {
    let serial = u16::from_le_bytes([request[2], request[3]]);
    build_4e_response_with_serial(request, serial, end_code, response_data)
}

fn build_4e_response_with_serial(
    request: &[u8],
    serial: u16,
    end_code: u16,
    response_data: &[u8],
) -> Vec<u8> {
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[0..2].copy_from_slice(&end_code.to_le_bytes());
    payload[2..].copy_from_slice(response_data);

    let mut response = vec![0u8; 13 + payload.len()];
    response[0] = 0xD4;
    response[1] = 0x00;
    response[2..4].copy_from_slice(&serial.to_le_bytes());
    response[6..11].copy_from_slice(&request[6..11]);
    response[11..13].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    response[13..].copy_from_slice(&payload);
    response
}

fn build_dword_payload(values: &[u32]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(values.len() * 4);
    for value in values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload
}

fn word_payload(values: &[u16]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(values.len() * 2);
    for value in values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload
}

#[tokio::test]
async fn frame_4e_ignores_mismatched_serial_response() {
    let server = SerialSkewResponseServer::start(vec![0x11, 0x11], vec![0x22, 0x22])
        .await
        .unwrap();

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let words = client
        .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::D, 0), 1)
        .await
        .unwrap();

    assert_eq!(words, vec![0x2222]);
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn direct_bit_read_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .read_bits(SlmpDeviceAddress::new(SlmpDeviceCode::LTS, 10), 1)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Direct bit read is not supported"));
}

#[tokio::test]
async fn close_shuts_down_tcp_stream() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (sender, receiver) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buffer = [0u8; 1];
        let read_result = stream.read(&mut buffer).await;
        let _ = sender.send(read_result);
    });

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqL);
    options.port = port;
    let client = SlmpClient::connect(options).await.unwrap();
    client.close().await.unwrap();

    let read_result = tokio::time::timeout(std::time::Duration::from_secs(1), receiver)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(read_result, 0);
}

#[tokio::test]
async fn self_test_loopback_rejects_manual_invalid_payloads_before_transport() {
    let client = udp_client().await;

    let err = client.self_test_loopback(b"HELLO").await.unwrap_err();
    assert!(err.to_string().contains("ASCII 0-9/A-F"));

    let err = client.self_test_loopback(&[0x00, 0xFF]).await.unwrap_err();
    assert!(err.to_string().contains("ASCII 0-9/A-F"));

    let err = client.self_test_loopback(b"").await.unwrap_err();
    assert!(err.to_string().contains("1..960"));

    let too_long = vec![b'A'; 961];
    let err = client.self_test_loopback(&too_long).await.unwrap_err();
    assert!(err.to_string().contains("1..960"));
}

#[tokio::test]
async fn udp_read_words_accepts_manual_limit_datagram_response() {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut request = vec![0u8; 1024];
        let (read, peer) = socket.recv_from(&mut request).await.unwrap();
        let mut response_data = Vec::new();
        for value in 0..960u16 {
            response_data.extend_from_slice(&value.to_le_bytes());
        }
        let response = build_4e_response(&request[..read], &response_data);
        socket.send_to(&response, peer).await.unwrap();
    });

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.transport_mode = SlmpTransportMode::Udp;
    options.port = port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = client
        .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::D, 0), 960)
        .await
        .unwrap();

    assert_eq!(values.len(), 960);
    assert_eq!(values[0], 0);
    assert_eq!(values[959], 959);
}

#[tokio::test]
async fn direct_bit_write_rejects_long_counter_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits(SlmpDeviceAddress::new(SlmpDeviceCode::LCC, 10), &[true])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct bit write is not supported")
    );
}

#[tokio::test]
async fn s_device_writes_are_rejected_before_transport() {
    let client = udp_client().await;
    let err = client
        .write_bits(SlmpDeviceAddress::new(SlmpDeviceCode::S, 10), &[true])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("S is read-only"));

    let err = client
        .write_random_bits(&[(SlmpDeviceAddress::new(SlmpDeviceCode::S, 10), true)])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("read-only devices such as S"));
}

#[tokio::test]
async fn direct_bit_write_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits(SlmpDeviceAddress::new(SlmpDeviceCode::LTC, 10), &[true])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct bit write is not supported")
    );
}

#[tokio::test]
async fn direct_word_write_rejects_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10), 4)
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word read is not supported")
    );

    let err = client
        .write_words(SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10), &[1])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word write is not supported")
    );

    let err = client
        .write_words(SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1), &[1])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word write is not supported")
    );
}

#[tokio::test]
async fn direct_dword_routes_reject_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .read_dwords_raw(SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10), 1)
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct dword read is not supported")
    );

    let err = client
        .write_dwords(SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1), &[1])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct dword write is not supported")
    );
}

#[tokio::test]
async fn dword_helpers_use_random_dword_route_for_lz() {
    let server = MultiResponseServer::start(vec![build_dword_payload(&[0x1234_5678, 0x9ABC_DEF0])])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqL);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values =
        read_dwords_single_request(&client, SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0), 2)
            .await
            .unwrap();
    assert_eq!(values, vec![0x1234_5678, 0x9ABC_DEF0]);
}

#[tokio::test]
async fn dword_helpers_apply_lz_random_read_limits() {
    let client = udp_client().await;
    let err =
        read_dwords_single_request(&client, SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0), 97)
            .await
            .unwrap_err();
    assert!(err.to_string().contains("1-96"));

    let err = read_dwords_chunked(&client, SlmpDeviceAddress::new(SlmpDeviceCode::D, 0), 1, 0)
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("max_dwords_per_request must be at least 1")
    );
}

#[tokio::test]
async fn typed_lz_routes_reject_non_dword_dtypes() {
    let client = udp_client().await;
    for dtype in ["U", "S", "F", "BIT"] {
        let err = read_typed(
            &client,
            SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1),
            dtype,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("32-bit device"));

        let err = write_typed(
            &client,
            SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1),
            dtype,
            &SlmpValue::U16(1),
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("32-bit device"));
    }
}

#[tokio::test]
async fn read_named_rejects_explicit_word_dtype_for_lz() {
    let client = udp_client().await;
    let err = read_named(&client, &["LZ1:U".to_string()])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("32-bit device"));
}

#[tokio::test]
async fn direct_extended_bit_write_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LSTS, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[true],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct bit write is not supported")
    );
}

#[tokio::test]
async fn direct_extended_bit_write_rejects_long_counter_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LCS, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[true],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct bit write is not supported")
    );
}

#[tokio::test]
async fn direct_extended_word_read_rejects_long_counter_current_devices() {
    let client = udp_client().await;
    let err = client
        .read_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            4,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word read is not supported")
    );
}

#[tokio::test]
async fn direct_extended_word_write_rejects_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .write_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LTN, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[1],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word write is not supported")
    );

    let err = client
        .write_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[1],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word write is not supported")
    );
}

#[test]
fn parse_qualified_device_rejects_hg_outside_iqr_cpu_range() {
    let g = parse_qualified_device(r"U1\G0").unwrap();
    assert_eq!(g.extension_specification, Some(0x0001));
    assert_eq!(g.direct_memory_specification, Some(0xF8));

    let hg = parse_qualified_device(r"U3E0\HG0").unwrap();
    assert_eq!(hg.extension_specification, Some(0x03E0));
    assert_eq!(hg.direct_memory_specification, Some(0xFA));

    let err = parse_qualified_device(r"U1\HG0").unwrap_err();
    assert!(
        err.to_string()
            .contains("HG Extended Device access is valid only for U3E0\\HG through U3E3\\HG")
    );
}

#[tokio::test]
async fn extended_g_hg_reject_unqualified_device_addresses() {
    let client = udp_client().await;

    let err = client
        .read_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::G, 0),
                extension_specification: None,
                direct_memory_specification: None,
            },
            1,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("G Extended Device access requires U-qualified")
    );

    let err = client
        .read_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::HG, 0),
                extension_specification: None,
                direct_memory_specification: None,
            },
            1,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("HG Extended Device access requires U-qualified")
    );
}

#[tokio::test]
async fn qualified_g_hg_extended_bit_routes_reach_transport() {
    let server = CapturingResponseServer::start(vec![(0, vec![0x10]), (0, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = client
        .read_bits_extended(
            parse_qualified_device(r"U3E0\G10").unwrap(),
            1,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap();
    assert_eq!(values, vec![true]);

    client
        .write_bits_extended(
            parse_qualified_device(r"U3E0\HG11").unwrap(),
            &[true],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap();

    let requests = server.requests().await;
    assert_eq!(requests.len(), 2);
    let read_body = &requests[0][13..];
    assert_eq!(u16::from_le_bytes([read_body[2], read_body[3]]), 0x0401);
    assert_eq!(u16::from_le_bytes([read_body[4], read_body[5]]), 0x0083);
    let write_body = &requests[1][13..];
    assert_eq!(u16::from_le_bytes([write_body[2], write_body[3]]), 0x1401);
    assert_eq!(u16::from_le_bytes([write_body[4], write_body[5]]), 0x0083);
}

#[tokio::test]
async fn standalone_g_hg_random_bit_writes_are_rejected() {
    let client = udp_client().await;
    let err = client
        .write_random_bits(&[(SlmpDeviceAddress::new(SlmpDeviceCode::G, 10), true)])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("standalone G/HG bit entries"));

    let err = client
        .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::G, 10), 1)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("standalone G/HG"));
}

#[tokio::test]
async fn random_read_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .read_random(&[SlmpDeviceAddress::new(SlmpDeviceCode::LTC, 10)], &[])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Read Random (0x0403) does not support LTS/LTC/LSTS/LSTC")
    );
}

#[tokio::test]
async fn random_word_routes_reject_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .read_random(&[SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10)], &[])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("does not support LTN/LSTN/LCN/LZ as word entries")
    );

    let err = client
        .write_random_words(&[(SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1), 1)], &[])
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("does not support LTN/LSTN/LCN/LZ as word entries")
    );
}

#[tokio::test]
async fn manual_point_limits_reject_overruns_before_transport() {
    let client = udp_client().await;

    assert!(
        client
            .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::D, 0), 961)
            .await
            .unwrap_err()
            .to_string()
            .contains("1..960")
    );
    assert!(
        client
            .write_words(SlmpDeviceAddress::new(SlmpDeviceCode::D, 0), &vec![0; 961])
            .await
            .unwrap_err()
            .to_string()
            .contains("1..960")
    );
    assert!(
        client
            .read_bits(SlmpDeviceAddress::new(SlmpDeviceCode::M, 0), 7169)
            .await
            .unwrap_err()
            .to_string()
            .contains("1..7168")
    );
    assert!(
        client
            .write_bits(
                SlmpDeviceAddress::new(SlmpDeviceCode::M, 0),
                &vec![false; 7169]
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("1..7168")
    );

    let iqf_client = udp_client_with_profile(SlmpPlcProfile::IqF).await;
    assert!(
        iqf_client
            .read_bits(SlmpDeviceAddress::new(SlmpDeviceCode::M, 0), 3585)
            .await
            .unwrap_err()
            .to_string()
            .contains("1..3584")
    );
    assert!(
        iqf_client
            .write_bits(
                SlmpDeviceAddress::new(SlmpDeviceCode::M, 0),
                &vec![false; 3585]
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("1..3584")
    );

    let random_words: Vec<_> = (0..81)
        .map(|i| (SlmpDeviceAddress::new(SlmpDeviceCode::D, 8000 + i), 0))
        .collect();
    assert!(
        client
            .write_random_words(&random_words, &[])
            .await
            .unwrap_err()
            .to_string()
            .contains("word/dword access points out of range")
    );

    let random_dwords: Vec<_> = (0..69)
        .map(|i| (SlmpDeviceAddress::new(SlmpDeviceCode::D, 8000 + (i * 2)), 0))
        .collect();
    assert!(
        client
            .write_random_words(&[], &random_dwords)
            .await
            .unwrap_err()
            .to_string()
            .contains("word/dword access points out of range")
    );

    let random_bits: Vec<_> = (0..95)
        .map(|i| (SlmpDeviceAddress::new(SlmpDeviceCode::M, 4000 + i), false))
        .collect();
    assert!(
        client
            .write_random_bits(&random_bits)
            .await
            .unwrap_err()
            .to_string()
            .contains("1..94")
    );

    assert!(
        client
            .read_block(
                &[SlmpBlockRead {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 0),
                    points: 961,
                }],
                &[],
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("total device points")
    );
    assert!(
        client
            .write_block(
                &[SlmpBlockWrite {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 8000),
                    values: vec![0; 952],
                }],
                &[],
                None,
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("total device points")
    );

    assert!(
        client
            .memory_read_words(0, 481)
            .await
            .unwrap_err()
            .to_string()
            .contains("1..480")
    );
    assert!(
        client
            .memory_write_words(0, &vec![0; 481])
            .await
            .unwrap_err()
            .to_string()
            .contains("1..480")
    );
    assert!(
        client
            .extend_unit_read_words(0, 961, 0x03E0)
            .await
            .unwrap_err()
            .to_string()
            .contains("1..960")
    );
    assert!(
        client
            .extend_unit_write_words(0, 0x03E0, &vec![0; 961])
            .await
            .unwrap_err()
            .to_string()
            .contains("1..960")
    );
}

#[tokio::test]
async fn block_routes_reject_lcn_lz_and_long_current_write_blocks() {
    let client = udp_client().await;
    let err = client
        .read_block(
            &[plc_comm_slmp::SlmpBlockRead {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10),
                points: 4,
            }],
            &[],
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("does not support LCN/LZ"));

    let err = client
        .write_block(
            &[plc_comm_slmp::SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0),
                values: vec![1, 0],
            }],
            &[],
            None,
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("does not support LTN/LSTN/LCN/LZ"));
}

#[tokio::test]
async fn qcpu_and_qnu_use_profile_feature_guard_before_transport() {
    for profile in [SlmpPlcProfile::QCpu, SlmpPlcProfile::QnU] {
        let client = udp_client_with_profile(profile).await;
        let profile_name = profile.canonical_name();
        let err = client
            .read_block(
                &[SlmpBlockRead {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100),
                    points: 1,
                }],
                &[SlmpBlockRead {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 100),
                    points: 1,
                }],
            )
            .await
            .unwrap_err();
        assert_eq!(err.kind, SlmpErrorKind::ProfileFeature);
        let info = err.profile_feature.as_ref().unwrap();
        assert_eq!(info.profile_id, profile_name);
        assert_eq!(info.feature_key, "block");
        assert_eq!(info.state, "blocked");

        let err = client
            .write_block(
                &[SlmpBlockWrite {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100),
                    values: vec![1],
                }],
                &[SlmpBlockWrite {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 100),
                    values: vec![1],
                }],
                None,
            )
            .await
            .unwrap_err();
        assert_eq!(err.kind, SlmpErrorKind::ProfileFeature);
        let info = err.profile_feature.as_ref().unwrap();
        assert_eq!(info.profile_id, profile_name);
        assert_eq!(info.feature_key, "block");
        assert_eq!(info.state, "blocked");
    }
}

#[tokio::test]
async fn ql_measured_profiles_use_profile_feature_guard_for_type_name_and_block() {
    for profile in [SlmpPlcProfile::LCpu, SlmpPlcProfile::QnUDV] {
        let client = udp_client_with_profile(profile).await;
        let profile_id = profile.canonical_name();

        let err = client.read_type_name().await.unwrap_err();
        assert_eq!(err.kind, SlmpErrorKind::ProfileFeature);
        let info = err.profile_feature.as_ref().unwrap();
        assert_eq!(info.profile_id, profile_id);
        assert_eq!(info.feature_key, "type_name");
        assert_eq!(info.state, "blocked");
        assert!(err.message.contains("C059"));
        assert!(err.message.contains("strict_profile=false"));
        assert_eq!(client.traffic_stats().await.request_count, 0);

        let err = client
            .read_block(
                &[SlmpBlockRead {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100),
                    points: 1,
                }],
                &[],
            )
            .await
            .unwrap_err();
        assert_eq!(err.kind, SlmpErrorKind::ProfileFeature);
        let info = err.profile_feature.as_ref().unwrap();
        assert_eq!(info.profile_id, profile_id);
        assert_eq!(info.feature_key, "block");
        assert_eq!(info.state, "blocked");
        assert!(err.message.contains("C059"));
        assert_eq!(client.traffic_stats().await.request_count, 0);
    }
}

#[tokio::test]
async fn qnudv_strict_profile_false_sends_high_level_block_request() {
    let server = CapturingResponseServer::start(vec![(0, word_payload(&[0x1234]))])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::QnUDV);
    options.port = server.port;
    options.strict_profile = false;
    let client = SlmpClient::connect(options).await.unwrap();

    let result = client
        .read_block(
            &[SlmpBlockRead {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100),
                points: 1,
            }],
            &[],
        )
        .await
        .unwrap();

    assert_eq!(result.word_values, vec![0x1234]);
    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(&requests[0][0..2], &[0x50, 0x00]);
    let body = request_body(&requests[0]);
    assert_eq!(u16::from_le_bytes([body[2], body[3]]), 0x0406);
    assert_eq!(u16::from_le_bytes([body[4], body[5]]), 0x0000);
}

#[tokio::test]
async fn raw_request_is_not_profile_feature_guarded() {
    let server = CapturingResponseServer::start(vec![(0, word_payload(&[0x5555]))])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::QnUDV);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();
    let payload = [
        0x01, 0x00, // one word block, no bit blocks
        0x64, 0x00, 0x00, 0xA8, // D100 legacy device spec
        0x01, 0x00, // one point
    ];

    let data = client
        .request(SlmpCommand::DeviceReadBlock, 0x0000, &payload, true)
        .await
        .unwrap();

    assert_eq!(data, word_payload(&[0x5555]));
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn profile_extended_feature_guards_match_canonical_states() {
    let iqf = udp_client_with_profile(SlmpPlcProfile::IqF).await;
    let err = iqf
        .read_words_extended(
            parse_qualified_device(r"J1\W0").unwrap(),
            1,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.kind, SlmpErrorKind::ProfileFeature);
    let info = err.profile_feature.as_ref().unwrap();
    assert_eq!(info.profile_id, "melsec:iq-f");
    assert_eq!(info.feature_key, "ext_link_direct");
    assert_eq!(info.state, "blocked");

    let iql = udp_client_with_profile(SlmpPlcProfile::IqL).await;
    let err = iql
        .read_words_extended(
            parse_qualified_device(r"U3E0\HG0").unwrap(),
            1,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.kind, SlmpErrorKind::ProfileFeature);
    let info = err.profile_feature.as_ref().unwrap();
    assert_eq!(info.profile_id, "melsec:iq-l");
    assert_eq!(info.feature_key, "hg_cpu_buffer");
    assert_eq!(info.state, "blocked");

    let qnudv = udp_client_with_profile(SlmpPlcProfile::QnUDV).await;
    let err = qnudv
        .read_words_extended(
            parse_qualified_device(r"U2\G100").unwrap(),
            1,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.kind, SlmpErrorKind::ProfileFeature);
    let info = err.profile_feature.as_ref().unwrap();
    assert_eq!(info.profile_id, "melsec:qnudv");
    assert_eq!(info.feature_key, "ext_module_access");
    assert_eq!(info.state, "blocked");
}

#[tokio::test]
async fn iqf_config_dependent_g_route_is_not_guarded() {
    let server = CapturingResponseServer::start(vec![(0, word_payload(&[0x0007]))])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqF);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = client
        .read_words_extended(
            parse_qualified_device(r"U1\G0").unwrap(),
            1,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap();

    assert_eq!(values, vec![0x0007]);
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn profile_write_policy_is_enforced_even_when_strict_profile_is_false() {
    let iqr = udp_client_with_profile_and_strict(SlmpPlcProfile::IqR, false).await;
    let err = iqr
        .write_bits(SlmpDeviceAddress::new(SlmpDeviceCode::S, 0), &[true])
        .await
        .unwrap_err();
    assert_eq!(err.kind, SlmpErrorKind::General);
    assert!(err.message.contains("S is read-only"));
    assert!(err.message.contains("melsec:iq-r"));
    assert_eq!(iqr.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn profile_limits_are_enforced_from_canonical_table() {
    let iqr = udp_client_with_profile(SlmpPlcProfile::IqR).await;
    let words: Vec<_> = (0..97)
        .map(|i| SlmpDeviceAddress::new(SlmpDeviceCode::D, 1000 + i))
        .collect();
    let err = iqr.read_random(&words, &[]).await.unwrap_err();
    assert!(err.message.contains("1..96"));
    assert_eq!(iqr.traffic_stats().await.request_count, 0);

    let iql = udp_client_with_profile(SlmpPlcProfile::IqL).await;
    let entries: Vec<_> = (0..81)
        .map(|i| (SlmpDeviceAddress::new(SlmpDeviceCode::D, 2000 + i), 0))
        .collect();
    let err = iql.write_random_words(&entries, &[]).await.unwrap_err();
    assert!(err.message.contains("1..80"));
    assert_eq!(iql.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn mixed_block_write_does_not_retry_c05b_as_split_requests() {
    let server = CapturingResponseServer::start(vec![(0xC05B, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client
        .write_block(
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100),
                values: vec![0x1234],
            }],
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 200),
                values: vec![0x0005],
            }],
            Some(SlmpBlockWriteOptions {
                split_mixed_blocks: false,
            }),
        )
        .await
        .unwrap_err();
    assert_eq!(error.end_code, Some(0xC05B));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_block_write_shape(&requests[0], 1, 1);
    // Manual-conformant layout: each block's data follows its own spec.
    assert_eq!(
        &requests[0][13 + 8..],
        &[
            0x64, 0x00, 0x00, 0x00, 0xA8, 0x00, 0x01, 0x00, 0x34, 0x12, // D100 x1 + data
            0xC8, 0x00, 0x00, 0x00, 0x90, 0x00, 0x01, 0x00, 0x05, 0x00, // M200 x1 + data
        ]
    );
}

#[tokio::test]
async fn mixed_block_write_does_not_retry_c056_as_split_requests() {
    let server = CapturingResponseServer::start(vec![(0xC056, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client
        .write_block(
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100),
                values: vec![0x1234],
            }],
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 200),
                values: vec![0x0005],
            }],
            Some(SlmpBlockWriteOptions {
                split_mixed_blocks: false,
            }),
        )
        .await
        .unwrap_err();
    assert_eq!(error.end_code, Some(0xC056));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_block_write_shape(&requests[0], 1, 1);
}

fn assert_block_write_shape(request: &[u8], word_blocks: u8, bit_blocks: u8) {
    let body = &request[13..];
    assert_eq!(u16::from_le_bytes([body[2], body[3]]), 0x1406);
    assert_eq!(u16::from_le_bytes([body[4], body[5]]), 0x0002);
    assert_eq!(body[6], word_blocks);
    assert_eq!(body[7], bit_blocks);
}

fn request_body(request: &[u8]) -> &[u8] {
    if request.starts_with(&[0x50, 0x00]) {
        &request[9..]
    } else {
        &request[13..]
    }
}
