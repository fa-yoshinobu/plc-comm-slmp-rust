use plc_comm_slmp::{
    NamedAddress, SlmpBlockRead, SlmpBlockWrite, SlmpClient, SlmpCommand, SlmpConnectionOptions,
    SlmpDeviceAddress, SlmpDeviceCode, SlmpErrorKind, SlmpPlcProfile, SlmpQualifiedDeviceAddress,
    SlmpTransportMode, SlmpValue, parse_qualified_device, parse_scalar_for_named,
    read_dwords_single_request, read_named, read_typed, write_named, write_typed,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};

async fn udp_client() -> SlmpClient {
    udp_client_with_profile(SlmpPlcProfile::IqR).await
}

async fn udp_client_with_profile(plc_profile: SlmpPlcProfile) -> SlmpClient {
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        plc_profile,
    )
    .unwrap();
    options.transport_mode = SlmpTransportMode::Udp;
    options.port = 9;
    SlmpClient::connect(options).await.unwrap()
}

fn qualified_device_for(
    plc_profile: SlmpPlcProfile,
    code: SlmpDeviceCode,
    number: u32,
) -> SlmpQualifiedDeviceAddress {
    SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(code, number, plc_profile))
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
    let response_data = response_data_with_error_info(request, end_code, response_data);
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[0..2].copy_from_slice(&end_code.to_le_bytes());
    payload[2..].copy_from_slice(&response_data);

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
    let response_data = response_data_with_error_info(request, end_code, response_data);
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[0..2].copy_from_slice(&end_code.to_le_bytes());
    payload[2..].copy_from_slice(&response_data);

    let mut response = vec![0u8; 13 + payload.len()];
    response[0] = 0xD4;
    response[1] = 0x00;
    response[2..4].copy_from_slice(&serial.to_le_bytes());
    response[6..11].copy_from_slice(&request[6..11]);
    response[11..13].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    response[13..].copy_from_slice(&payload);
    response
}

fn response_data_with_error_info(request: &[u8], end_code: u16, response_data: &[u8]) -> Vec<u8> {
    if end_code == 0 {
        return response_data.to_vec();
    }
    let target_offset = if request.starts_with(&[0x50, 0x00]) {
        2
    } else {
        6
    };
    let command_offset = if request.starts_with(&[0x50, 0x00]) {
        11
    } else {
        15
    };
    let mut data = Vec::with_capacity(9 + response_data.len());
    data.extend_from_slice(&request[target_offset..target_offset + 5]);
    data.extend_from_slice(&request[command_offset..command_offset + 4]);
    data.extend_from_slice(response_data);
    data
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

    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let words = client
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR),
            1,
        )
        .await
        .unwrap();

    assert_eq!(words, vec![0x2222]);
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn direct_bit_read_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .read_bits(
            SlmpDeviceAddress::new(SlmpDeviceCode::LTS, 10, SlmpPlcProfile::IqR),
            1,
        )
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

    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqL,
    )
    .unwrap();
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
async fn self_test_loopback_requires_exact_declared_length_size_and_echo() {
    let server = CapturingResponseServer::start(vec![
        (0x0000, vec![0x04, 0x00, b'A', b'1', b'B', b'2']),
        (0x0000, vec![0x03, 0x00, b'A', b'1', b'B']),
        (0x0000, vec![0x04, 0x00, b'A', b'1', b'B', b'2', b'F']),
        (0x0000, vec![0x04, 0x00, b'A', b'1', b'B', b'3']),
    ])
    .await
    .unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();

    assert_eq!(client.self_test_loopback(b"A1B2").await.unwrap(), b"A1B2");

    let declared = client.self_test_loopback(b"A1B2").await.unwrap_err();
    assert!(declared.to_string().contains("declared length mismatch"));

    let trailing = client.self_test_loopback(b"A1B2").await.unwrap_err();
    assert!(trailing.to_string().contains("size mismatch"));

    let payload = client.self_test_loopback(b"A1B2").await.unwrap_err();
    assert!(payload.to_string().contains("payload mismatch"));

    assert_eq!(server.requests().await.len(), 4);
}

#[tokio::test]
async fn clear_error_sends_one_fixed_empty_command() {
    let server = CapturingResponseServer::start(vec![(0x0000, vec![])])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    client.clear_error().await.unwrap();

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        u16::from_le_bytes([requests[0][15], requests[0][16]]),
        0x1617
    );
    assert_eq!(
        u16::from_le_bytes([requests[0][17], requests[0][18]]),
        0x0000
    );
    assert_eq!(requests[0].len(), 19);
}

#[tokio::test]
async fn monitor_semantic_apis_register_and_decode_three_cycles() {
    let monitor_data = vec![0x11, 0x11, 0x78, 0x56, 0x34, 0x12];
    let server = CapturingResponseServer::start(vec![
        (0x0000, vec![]),
        (0x0000, monitor_data.clone()),
        (0x0000, monitor_data.clone()),
        (0x0000, monitor_data),
    ])
    .await
    .unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();
    let word = SlmpDeviceAddress::new(SlmpDeviceCode::D, 120, SlmpPlcProfile::IqR);
    let dword = SlmpDeviceAddress::new(SlmpDeviceCode::D, 200, SlmpPlcProfile::IqR);

    client
        .register_monitor_devices(&[word], &[dword])
        .await
        .unwrap();
    for _ in 0..3 {
        let result = client.run_monitor_cycle(1, 1).await.unwrap();
        assert_eq!(result.word_values, vec![0x1111]);
        assert_eq!(result.dword_values, vec![0x1234_5678]);
    }
    assert!(
        client
            .run_monitor_cycle(0, 0)
            .await
            .unwrap_err()
            .to_string()
            .contains("out of range")
    );
    assert!(
        client
            .run_monitor_cycle(97, 0)
            .await
            .unwrap_err()
            .to_string()
            .contains("out of range")
    );

    let requests = server.requests().await;
    assert_eq!(requests.len(), 4);
    assert_eq!(
        u16::from_le_bytes([requests[0][15], requests[0][16]]),
        0x0801
    );
    assert_eq!(
        &requests[0][19..33],
        &[
            0x01, 0x01, 0x78, 0x00, 0x00, 0x00, 0xA8, 0x00, 0xC8, 0x00, 0x00, 0x00, 0xA8, 0x00
        ]
    );
    for request in &requests[1..] {
        assert_eq!(u16::from_le_bytes([request[15], request[16]]), 0x0802);
        assert_eq!(request.len(), 19);
    }
}

#[tokio::test]
async fn legacy_monitor_registration_pins_counts_and_device_order() {
    let server = CapturingResponseServer::start(vec![(0x0000, vec![])])
        .await
        .unwrap();
    let profile = SlmpPlcProfile::QnUQj71E71100;
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        profile,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();

    client
        .register_monitor_devices(
            &[SlmpDeviceAddress::new(SlmpDeviceCode::D, 120, profile)],
            &[SlmpDeviceAddress::new(SlmpDeviceCode::D, 200, profile)],
        )
        .await
        .unwrap();

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        &requests[0][19..29],
        &[0x01, 0x01, 0x78, 0x00, 0x00, 0xA8, 0xC8, 0x00, 0x00, 0xA8]
    );
}

#[tokio::test]
async fn monitor_registration_rejects_empty_over_limit_and_long_state_before_transport() {
    let server = CapturingResponseServer::start(vec![]).await.unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();

    assert!(client.register_monitor_devices(&[], &[]).await.is_err());
    assert!(client.register_monitor_devices_ext(&[], &[]).await.is_err());
    let normal = (0..97)
        .map(|number| SlmpDeviceAddress::new(SlmpDeviceCode::D, number, SlmpPlcProfile::IqR))
        .collect::<Vec<_>>();
    assert!(client.register_monitor_devices(&normal, &[]).await.is_err());
    let extended = (0..97)
        .map(|number| qualified_device_for(SlmpPlcProfile::IqR, SlmpDeviceCode::D, number))
        .collect::<Vec<_>>();
    assert!(
        client
            .register_monitor_devices_ext(&extended, &[])
            .await
            .is_err()
    );
    let long_state = SlmpDeviceAddress::new(SlmpDeviceCode::LCS, 0, SlmpPlcProfile::IqR);
    let error = client
        .register_monitor_devices(&[long_state], &[])
        .await
        .unwrap_err();
    assert!(error.to_string().contains("0x0801"));
    assert!(server.requests().await.is_empty());
}

#[tokio::test]
async fn extended_monitor_registration_uses_qualified_subcommand() {
    let server = CapturingResponseServer::start(vec![(0x0000, vec![])])
        .await
        .unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();
    let hg = parse_qualified_device(r"U3E0\HG0", SlmpPlcProfile::IqR).unwrap();

    client
        .register_monitor_devices_ext(&[hg], &[])
        .await
        .unwrap();

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        u16::from_le_bytes([requests[0][15], requests[0][16]]),
        0x0801
    );
    assert_eq!(
        u16::from_le_bytes([requests[0][17], requests[0][18]]),
        0x0082
    );
}

#[tokio::test]
async fn hg_qualified_device_never_changes_user_selected_request_target() {
    async fn write_once(target_module_io: u16) -> Vec<u8> {
        let server = CapturingResponseServer::start(vec![(0x0000, vec![])])
            .await
            .unwrap();
        let target = plc_comm_slmp::SlmpTargetAddress {
            module_io: target_module_io,
            ..Default::default()
        };
        let options = SlmpConnectionOptions::new(
            "127.0.0.1",
            server.port,
            SlmpTransportMode::Tcp,
            target,
            SlmpPlcProfile::IqR,
        )
        .unwrap();
        let client = SlmpClient::connect(options).await.unwrap();
        let hg = parse_qualified_device(r"U3E1\HG100", SlmpPlcProfile::IqR).unwrap();

        client.write_words_extended(hg, &[0x1234]).await.unwrap();

        server.requests().await.into_iter().next().unwrap()
    }

    let own_station = write_once(plc_comm_slmp::SlmpModuleIo::OWN_STATION).await;
    let cpu_2 = write_once(plc_comm_slmp::SlmpModuleIo::MULTIPLE_CPU_2).await;

    assert_eq!(u16::from_le_bytes([own_station[8], own_station[9]]), 0x03FF);
    assert_eq!(u16::from_le_bytes([cpu_2[8], cpu_2[9]]), 0x03E1);
    assert_eq!(
        u16::from_le_bytes([own_station[15], own_station[16]]),
        0x1401
    );
    assert_eq!(u16::from_le_bytes([cpu_2[15], cpu_2[16]]), 0x1401);
}

#[tokio::test]
async fn monitor_semantic_api_propagates_plc_ng_without_fallback() {
    let server = CapturingResponseServer::start(vec![(0xC051, vec![])])
        .await
        .unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client.run_monitor_cycle(1, 0).await.unwrap_err();

    assert_eq!(error.end_code, Some(0xC051));
    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(
        u16::from_le_bytes([requests[0][15], requests[0][16]]),
        0x0802
    );
}

#[tokio::test]
async fn monitor_semantic_api_rejects_response_size_mismatch() {
    let server = CapturingResponseServer::start(vec![(0x0000, vec![0x11])])
        .await
        .unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client.run_monitor_cycle(1, 0).await.unwrap_err();

    assert!(error.to_string().contains("monitor response size mismatch"));
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn monitor_semantic_api_rejects_trailing_response_bytes() {
    let server = CapturingResponseServer::start(vec![(0x0000, vec![0x11, 0x11, 0x22])])
        .await
        .unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client.run_monitor_cycle(1, 0).await.unwrap_err();

    assert!(error.to_string().contains("monitor response size mismatch"));
    assert_eq!(server.requests().await.len(), 1);
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

    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.transport_mode = SlmpTransportMode::Udp;
    options.port = port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = client
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR),
            960,
        )
        .await
        .unwrap();

    assert_eq!(values.len(), 960);
    assert_eq!(values[0], 0);
    assert_eq!(values[959], 959);
}

#[tokio::test]
async fn profile_mismatch_is_rejected_before_transport() {
    let client = udp_client_with_profile(SlmpPlcProfile::IqF).await;
    let error = client
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR),
            1,
        )
        .await
        .unwrap_err();

    assert!(error.message.contains("profile mismatch"));
    assert!(error.message.contains("melsec:iq-r"));
    assert!(error.message.contains("melsec:iq-f"));
    assert_eq!(client.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn long_timer_typed_read_rejects_profile_mismatch_before_transport() {
    let client = udp_client_with_profile(SlmpPlcProfile::IqR).await;
    let error = read_typed(
        &client,
        SlmpDeviceAddress::new(SlmpDeviceCode::LTN, 10, SlmpPlcProfile::IqF),
        "D",
    )
    .await
    .unwrap_err();

    assert!(error.message.contains("does not match client"));
    assert_eq!(client.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn typed_writes_reject_cross_type_coercion_before_transport() {
    let client = udp_client().await;
    let device = SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR);
    let invalid = [
        ("U", SlmpValue::U32(70_000)),
        ("U", SlmpValue::I32(-1)),
        ("U", SlmpValue::F32(1.0)),
        ("D", SlmpValue::I32(-1)),
        ("L", SlmpValue::U32(u32::MAX)),
        ("F", SlmpValue::F32(f32::INFINITY)),
        ("BIT", SlmpValue::U16(1)),
    ];
    for (dtype, value) in invalid {
        assert!(write_typed(&client, device, dtype, &value).await.is_err());
    }
    assert_eq!(client.traffic_stats().await.request_count, 0);
}

#[test]
fn named_scalar_parser_rejects_out_of_range_and_ambiguous_values() {
    let profile = SlmpPlcProfile::IqR;
    assert!(parse_scalar_for_named("D0:U", "70000", profile).is_err());
    assert!(parse_scalar_for_named("D0:U", "-1", profile).is_err());
    assert!(parse_scalar_for_named("D0:S", "32768", profile).is_err());
    assert!(parse_scalar_for_named("D0:D", "4294967296", profile).is_err());
    assert!(parse_scalar_for_named("D0:L", "2147483648", profile).is_err());
    assert!(parse_scalar_for_named("D0:F", "inf", profile).is_err());
    assert!(parse_scalar_for_named("M0:BIT", "yes", profile).is_err());
}

#[tokio::test]
async fn named_write_batches_one_family_and_rejects_hidden_multi_request_routes() {
    let server = CapturingResponseServer::start(vec![(0, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();
    let mut updates = NamedAddress::new();
    updates.insert("D100:U".to_string(), SlmpValue::U16(1));
    updates.insert("D101:U".to_string(), SlmpValue::U16(2));
    write_named(&client, &updates).await.unwrap();
    assert_eq!(server.requests().await.len(), 1);

    let pretransport = udp_client().await;
    let mut mixed = NamedAddress::new();
    mixed.insert("D100:U".to_string(), SlmpValue::U16(1));
    mixed.insert("M100:BIT".to_string(), SlmpValue::Bool(true));
    assert!(write_named(&pretransport, &mixed).await.is_err());
    let mut bit_in_word = NamedAddress::new();
    bit_in_word.insert("D100.1".to_string(), SlmpValue::Bool(true));
    assert!(write_named(&pretransport, &bit_in_word).await.is_err());
    assert_eq!(pretransport.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn random_and_block_writes_reject_duplicate_or_overlapping_ranges() {
    let client = udp_client().await;
    let d100 = SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR);
    let d101 = SlmpDeviceAddress::new(SlmpDeviceCode::D, 101, SlmpPlcProfile::IqR);
    let m100 = SlmpDeviceAddress::new(SlmpDeviceCode::M, 100, SlmpPlcProfile::IqR);

    assert!(
        client
            .write_random_u16s(&[(d100, 1), (d100, 2)])
            .await
            .is_err()
    );
    assert!(
        client
            .write_random_words(&[(d101, 1)], &[(d100, 2)])
            .await
            .is_err()
    );
    assert!(
        client
            .write_random_u32s(&[(d100, 1), (d101, 2)])
            .await
            .is_err()
    );
    assert!(
        client
            .write_random_bits(&[(m100, true), (m100, false)])
            .await
            .is_err()
    );
    assert!(
        client
            .write_word_blocks(&[
                SlmpBlockWrite {
                    device: d100,
                    values: vec![1, 2]
                },
                SlmpBlockWrite {
                    device: d101,
                    values: vec![3]
                },
            ])
            .await
            .is_err()
    );

    let q100 = SlmpQualifiedDeviceAddress::module_access(d100, 1).unwrap();
    let q101 = SlmpQualifiedDeviceAddress::module_access(d101, 1).unwrap();
    assert!(
        client
            .write_random_words_ext(&[(q101, 1)], &[(q100, 2)])
            .await
            .is_err()
    );
    assert_eq!(client.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn extended_random_overlap_identity_includes_the_qualified_route() {
    let server = CapturingResponseServer::start(vec![(0, Vec::new())])
        .await
        .unwrap();
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.port,
        SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();
    let device = SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR);
    let unit1 = SlmpQualifiedDeviceAddress::module_access(device, 1).unwrap();
    let unit2 = SlmpQualifiedDeviceAddress::module_access(device, 2).unwrap();

    client
        .write_random_u16s_extended(&[(unit1, 1), (unit2, 2)])
        .await
        .unwrap();
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn aggregate_operations_reject_all_empty_inputs_before_transport() {
    let client = udp_client().await;

    assert!(client.read_random(&[], &[]).await.is_err());
    assert!(client.read_random_ext(&[], &[]).await.is_err());
    assert!(client.write_random_words(&[], &[]).await.is_err());
    assert!(client.write_random_words_ext(&[], &[]).await.is_err());
    assert!(client.write_random_bits(&[]).await.is_err());
    assert!(client.write_random_bits_ext(&[]).await.is_err());
    assert!(client.read_block(&[], &[]).await.is_err());
    assert!(client.write_block(&[], &[]).await.is_err());
    assert_eq!(client.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn udp_timeout_closes_transport_before_another_request() {
    let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        server.local_addr().unwrap().port(),
        SlmpTransportMode::Udp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.timeout = std::time::Duration::from_millis(20);
    let client = SlmpClient::connect(options).await.unwrap();
    let address = SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR);

    let first = client.read_words_raw(address, 1).await.unwrap_err();
    assert!(first.message.contains("udp receive timed out"));
    let second = client.read_words_raw(address, 1).await.unwrap_err();
    assert!(second.message.contains("transport is closed"));
    assert_eq!(client.traffic_stats().await.request_count, 1);
}

#[tokio::test]
async fn direct_bit_write_rejects_long_counter_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits(
            SlmpDeviceAddress::new(SlmpDeviceCode::LCC, 10, SlmpPlcProfile::IqR),
            &[true],
        )
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
        .write_bits(
            SlmpDeviceAddress::new(SlmpDeviceCode::S, 10, SlmpPlcProfile::IqR),
            &[true],
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("S is read-only"));

    let err = client
        .write_random_bits(&[(
            SlmpDeviceAddress::new(SlmpDeviceCode::S, 10, SlmpPlcProfile::IqR),
            true,
        )])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("profile read-only devices"));
}

#[tokio::test]
async fn direct_bit_write_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits(
            SlmpDeviceAddress::new(SlmpDeviceCode::LTC, 10, SlmpPlcProfile::IqR),
            &[true],
        )
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
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10, SlmpPlcProfile::IqR),
            4,
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word read is not supported")
    );

    let err = client
        .write_words(
            SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10, SlmpPlcProfile::IqR),
            &[1],
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word write is not supported")
    );

    let err = client
        .write_words(
            SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1, SlmpPlcProfile::IqR),
            &[1],
        )
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
        .read_dwords_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10, SlmpPlcProfile::IqR),
            1,
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct dword read is not supported")
    );

    let err = client
        .write_dwords(
            SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1, SlmpPlcProfile::IqR),
            &[1],
        )
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
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqL,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = read_dwords_single_request(
        &client,
        SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0, SlmpPlcProfile::IqL),
        2,
    )
    .await
    .unwrap();
    assert_eq!(values, vec![0x1234_5678, 0x9ABC_DEF0]);
}

#[tokio::test]
async fn dword_helpers_apply_lz_random_read_limits() {
    let client = udp_client().await;
    let err = read_dwords_single_request(
        &client,
        SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0, SlmpPlcProfile::IqR),
        97,
    )
    .await
    .unwrap_err();
    assert!(err.to_string().contains("1-96"));
}

#[tokio::test]
async fn typed_lz_routes_reject_non_dword_dtypes() {
    let client = udp_client().await;
    for dtype in ["U", "S", "F", "BIT"] {
        let err = read_typed(
            &client,
            SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1, SlmpPlcProfile::IqR),
            dtype,
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("32-bit device"));

        let err = write_typed(
            &client,
            SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1, SlmpPlcProfile::IqR),
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
            SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(
                SlmpDeviceCode::LSTS,
                10,
                SlmpPlcProfile::IqR,
            )),
            &[true],
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
            SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(
                SlmpDeviceCode::LCS,
                10,
                SlmpPlcProfile::IqR,
            )),
            &[true],
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
            SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(
                SlmpDeviceCode::LCN,
                10,
                SlmpPlcProfile::IqR,
            )),
            4,
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
            SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(
                SlmpDeviceCode::LTN,
                10,
                SlmpPlcProfile::IqR,
            )),
            &[1],
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("Direct word write is not supported")
    );

    let err = client
        .write_words_extended(
            SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(
                SlmpDeviceCode::LZ,
                1,
                SlmpPlcProfile::IqR,
            )),
            &[1],
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
    let g = parse_qualified_device(r"U1\G0", plc_comm_slmp::SlmpPlcProfile::IqR).unwrap();
    assert_eq!(g.extension_specification(), Some(0x0001));
    assert_eq!(g.direct_memory_specification(), Some(0xF8));

    let hg = parse_qualified_device(r"U3E0\HG0", plc_comm_slmp::SlmpPlcProfile::IqR).unwrap();
    assert_eq!(hg.extension_specification(), Some(0x03E0));
    assert_eq!(hg.direct_memory_specification(), Some(0xFA));

    let err = parse_qualified_device(r"U1\HG0", plc_comm_slmp::SlmpPlcProfile::IqR).unwrap_err();
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
            SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(
                SlmpDeviceCode::G,
                0,
                SlmpPlcProfile::IqR,
            )),
            1,
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("G Extended Device access requires U-qualified")
    );

    let err = client
        .read_words_extended(
            SlmpQualifiedDeviceAddress::new(SlmpDeviceAddress::new(
                SlmpDeviceCode::HG,
                0,
                SlmpPlcProfile::IqR,
            )),
            1,
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
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = client
        .read_bits_extended(
            parse_qualified_device(r"U3E0\G10", plc_comm_slmp::SlmpPlcProfile::IqR).unwrap(),
            1,
        )
        .await
        .unwrap();
    assert_eq!(values, vec![true]);

    client
        .write_bits_extended(
            parse_qualified_device(r"U3E0\HG11", plc_comm_slmp::SlmpPlcProfile::IqR).unwrap(),
            &[true],
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
async fn extended_random_uses_profile_ext_limit_keys_before_transport() {
    let iqf = udp_client_with_profile(SlmpPlcProfile::IqF).await;

    let read_devices: Vec<_> = (0..97)
        .map(|index| qualified_device_for(SlmpPlcProfile::IqF, SlmpDeviceCode::D, index))
        .collect();
    assert!(
        iqf.read_random_ext(&read_devices, &[])
            .await
            .unwrap_err()
            .to_string()
            .contains("1..96")
    );

    let word_entries: Vec<_> = (0..81)
        .map(|index| {
            (
                qualified_device_for(SlmpPlcProfile::IqF, SlmpDeviceCode::D, 8000 + index),
                0,
            )
        })
        .collect();
    assert!(
        iqf.write_random_words_ext(&word_entries, &[])
            .await
            .unwrap_err()
            .to_string()
            .contains("1..80")
    );

    let bit_entries: Vec<_> = (0..95)
        .map(|index| {
            (
                qualified_device_for(SlmpPlcProfile::IqF, SlmpDeviceCode::M, 4000 + index),
                false,
            )
        })
        .collect();
    assert!(
        iqf.write_random_bits_ext(&bit_entries)
            .await
            .unwrap_err()
            .to_string()
            .contains("1..94")
    );

    let qcpu = udp_client_with_profile(SlmpPlcProfile::QCpuQj71E71100).await;
    let qcpu_read_devices: Vec<_> = (0..186)
        .map(|index| qualified_device_for(SlmpPlcProfile::QCpuQj71E71100, SlmpDeviceCode::D, index))
        .collect();
    assert!(
        qcpu.read_random_ext(&qcpu_read_devices, &[])
            .await
            .unwrap_err()
            .to_string()
            .contains("1..185")
    );
}

#[tokio::test]
async fn standalone_g_hg_random_bit_writes_are_rejected() {
    let client = udp_client().await;
    let err = client
        .write_random_bits(&[(
            SlmpDeviceAddress::new(SlmpDeviceCode::G, 10, SlmpPlcProfile::IqR),
            true,
        )])
        .await
        .unwrap_err();
    assert!(err.to_string().contains("standalone G/HG bit entries"));

    let err = client
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::G, 10, SlmpPlcProfile::IqR),
            1,
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("standalone G/HG"));
}

#[tokio::test]
async fn random_read_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .read_random(
            &[SlmpDeviceAddress::new(
                SlmpDeviceCode::LTC,
                10,
                SlmpPlcProfile::IqR,
            )],
            &[],
        )
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
        .read_random(
            &[SlmpDeviceAddress::new(
                SlmpDeviceCode::LCN,
                10,
                SlmpPlcProfile::IqR,
            )],
            &[],
        )
        .await
        .unwrap_err();
    assert!(
        err.to_string()
            .contains("does not support LTN/LSTN/LCN/LZ as word entries")
    );

    let err = client
        .write_random_words(
            &[(
                SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1, SlmpPlcProfile::IqR),
                1,
            )],
            &[],
        )
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
            .read_words_raw(
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR),
                961
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("1..960")
    );
    assert!(
        client
            .write_words(
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR),
                &vec![0; 961]
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("1..960")
    );
    assert!(
        client
            .read_bits(
                SlmpDeviceAddress::new(SlmpDeviceCode::M, 0, SlmpPlcProfile::IqR),
                7169
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("1..7168")
    );
    assert!(
        client
            .write_bits(
                SlmpDeviceAddress::new(SlmpDeviceCode::M, 0, SlmpPlcProfile::IqR),
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
            .read_bits(
                SlmpDeviceAddress::new(SlmpDeviceCode::M, 0, SlmpPlcProfile::IqF),
                3585
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("1..3584")
    );
    assert!(
        iqf_client
            .write_bits(
                SlmpDeviceAddress::new(SlmpDeviceCode::M, 0, SlmpPlcProfile::IqF),
                &vec![false; 3585]
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("1..3584")
    );

    let random_words: Vec<_> = (0..81)
        .map(|i| {
            (
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 8000 + i, SlmpPlcProfile::IqR),
                0,
            )
        })
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
        .map(|i| {
            (
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 8000 + (i * 2), SlmpPlcProfile::IqR),
                0,
            )
        })
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
        .map(|i| {
            (
                SlmpDeviceAddress::new(SlmpDeviceCode::M, 4000 + i, SlmpPlcProfile::IqR),
                false,
            )
        })
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
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqR),
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
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 8000, SlmpPlcProfile::IqR),
                    values: vec![0; 952],
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
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10, SlmpPlcProfile::IqR),
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
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0, SlmpPlcProfile::IqR),
                values: vec![1, 0],
            }],
            &[],
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("does not support LTN/LSTN/LCN/LZ"));
}

#[tokio::test]
async fn ql_builtin_profiles_use_profile_feature_guard_before_transport() {
    for profile in [SlmpPlcProfile::LCpu, SlmpPlcProfile::QnU] {
        let client = udp_client_with_profile(profile).await;
        let profile_name = profile.canonical_name();
        let err = client
            .read_block(
                &[SlmpBlockRead {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, profile),
                    points: 1,
                }],
                &[SlmpBlockRead {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 100, profile),
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
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, profile),
                    values: vec![1],
                }],
                &[SlmpBlockWrite {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 100, profile),
                    values: vec![1],
                }],
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
        assert_eq!(client.traffic_stats().await.request_count, 0);

        let err = client
            .read_block(
                &[SlmpBlockRead {
                    device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, profile),
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
async fn raw_request_is_not_profile_feature_guarded() {
    let server = CapturingResponseServer::start(vec![(0, word_payload(&[0x5555]))])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::QnUDV,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();
    let payload = [
        0x01, 0x00, // one word block, no bit blocks
        0x64, 0x00, 0x00, 0xA8, // D100 legacy device spec
        0x01, 0x00, // one point
    ];

    let data = client
        .raw_command(SlmpCommand::DeviceReadBlock, 0x0000, &payload)
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
            parse_qualified_device(r"J1\W0", SlmpPlcProfile::IqF).unwrap(),
            1,
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
            parse_qualified_device(r"U3E0\HG0", SlmpPlcProfile::IqL).unwrap(),
            1,
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
            parse_qualified_device(r"U2\G100", SlmpPlcProfile::QnUDV).unwrap(),
            1,
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
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqF,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = client
        .read_words_extended(
            parse_qualified_device(r"U1\G0", SlmpPlcProfile::IqF).unwrap(),
            1,
        )
        .await
        .unwrap();

    assert_eq!(values, vec![0x0007]);
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn profile_write_policy_is_enforced() {
    let iqr = udp_client_with_profile(SlmpPlcProfile::IqR).await;
    let err = iqr
        .write_bits(
            SlmpDeviceAddress::new(SlmpDeviceCode::S, 0, SlmpPlcProfile::IqR),
            &[true],
        )
        .await
        .unwrap_err();
    assert_eq!(err.kind, SlmpErrorKind::General);
    assert!(err.message.contains("S is read-only"));
    assert!(err.message.contains("melsec:iq-r"));
    assert_eq!(iqr.traffic_stats().await.request_count, 0);

    let server = CapturingResponseServer::start(vec![(0, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqF,
    )
    .unwrap();
    options.port = server.port;
    let iqf = SlmpClient::connect(options).await.unwrap();
    iqf.write_bits(
        SlmpDeviceAddress::new(SlmpDeviceCode::S, 0, SlmpPlcProfile::IqF),
        &[true],
    )
    .await
    .unwrap();
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn profile_limits_are_enforced_from_canonical_table() {
    let iqr = udp_client_with_profile(SlmpPlcProfile::IqR).await;
    let words: Vec<_> = (0..97)
        .map(|i| SlmpDeviceAddress::new(SlmpDeviceCode::D, 1000 + i, SlmpPlcProfile::IqR))
        .collect();
    let err = iqr.read_random(&words, &[]).await.unwrap_err();
    assert!(err.message.contains("1..96"));
    assert_eq!(iqr.traffic_stats().await.request_count, 0);

    let iql = udp_client_with_profile(SlmpPlcProfile::IqL).await;
    let entries: Vec<_> = (0..81)
        .map(|i| {
            (
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 2000 + i, SlmpPlcProfile::IqL),
                0,
            )
        })
        .collect();
    let err = iql.write_random_words(&entries, &[]).await.unwrap_err();
    assert!(err.message.contains("1..80"));
    assert_eq!(iql.traffic_stats().await.request_count, 0);

    let word_entries: Vec<_> = (0..40)
        .map(|i| {
            (
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 8100 + i, SlmpPlcProfile::IqL),
                0,
            )
        })
        .collect();
    let dword_entries: Vec<_> = (0..40)
        .map(|i| {
            (
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 8200 + (i * 2), SlmpPlcProfile::IqL),
                0,
            )
        })
        .collect();
    let err = iql
        .write_random_words(&word_entries, &dword_entries)
        .await
        .unwrap_err();
    assert!(err.message.contains("limit=960"));
    assert_eq!(iql.traffic_stats().await.request_count, 0);

    let iqf = udp_client_with_profile(SlmpPlcProfile::IqF).await;
    let iqf_dword_entries: Vec<_> = (0..138)
        .map(|i| {
            (
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 9000 + (i * 2), SlmpPlcProfile::IqF),
                0,
            )
        })
        .collect();
    let err = iqf
        .write_random_words(&[], &iqf_dword_entries)
        .await
        .unwrap_err();
    assert!(err.message.contains("limit=1920"));
    assert_eq!(iqf.traffic_stats().await.request_count, 0);
}

#[tokio::test]
async fn direct_access_does_not_use_device_range_upper_bounds_as_send_guard() {
    let server = CapturingResponseServer::start(vec![(0, vec![0x34, 0x12]), (0, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let values = client
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 999_999, SlmpPlcProfile::IqR),
            1,
        )
        .await
        .unwrap();
    assert_eq!(values, vec![0x1234]);

    client
        .write_words(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 999_999, SlmpPlcProfile::IqR),
            &[0x5678],
        )
        .await
        .unwrap();
    assert_eq!(server.requests().await.len(), 2);
}

#[tokio::test]
async fn mixed_block_write_does_not_retry_c05b_as_split_requests() {
    let server = CapturingResponseServer::start(vec![(0xC05B, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client
        .write_block(
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR),
                values: vec![0x1234],
            }],
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 200, SlmpPlcProfile::IqR),
                values: vec![0x0005],
            }],
        )
        .await
        .unwrap_err();
    assert_eq!(error.end_code, Some(0xC05B));
    let info = error.error_info.as_ref().expect("mock error info");
    assert_eq!(info.command, 0x1406);
    assert_eq!(info.subcommand, 0x0002);

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
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client
        .write_block(
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR),
                values: vec![0x1234],
            }],
            &[SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::M, 200, SlmpPlcProfile::IqR),
                values: vec![0x0005],
            }],
        )
        .await
        .unwrap_err();
    assert_eq!(error.end_code, Some(0xC056));
    let info = error.error_info.as_ref().expect("mock error info");
    assert_eq!(info.command, 0x1406);
    assert_eq!(info.subcommand, 0x0002);

    let requests = server.requests().await;
    assert_eq!(requests.len(), 1);
    assert_block_write_shape(&requests[0], 1, 1);
}

#[tokio::test]
async fn frame_3e_mock_nonzero_end_code_includes_error_info() {
    let server = CapturingResponseServer::start(vec![(0xC056, Vec::new())])
        .await
        .unwrap();
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqF,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client
        .read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, SlmpPlcProfile::IqF),
            1,
        )
        .await
        .unwrap_err();

    assert_eq!(error.end_code, Some(0xC056));
    let info = error.error_info.as_ref().expect("mock error info");
    assert_eq!(info.network, 0x00);
    assert_eq!(info.station, 0xFF);
    assert_eq!(info.module_io, 0x03FF);
    assert_eq!(info.command, 0x0401);
    assert_eq!(info.subcommand, 0x0000);
}

fn assert_block_write_shape(request: &[u8], word_blocks: u8, bit_blocks: u8) {
    let body = &request[13..];
    assert_eq!(u16::from_le_bytes([body[2], body[3]]), 0x1406);
    assert_eq!(u16::from_le_bytes([body[4], body[5]]), 0x0002);
    assert_eq!(body[6], word_blocks);
    assert_eq!(body[7], bit_blocks);
}
