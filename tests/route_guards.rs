use plc_comm_slmp::{
    SlmpBlockRead, SlmpBlockWrite, SlmpBlockWriteOptions, SlmpClient, SlmpConnectionOptions,
    SlmpDeviceAddress, SlmpDeviceCode, SlmpExtensionSpec, SlmpPlcProfile,
    SlmpQualifiedDeviceAddress, SlmpTransportMode, SlmpValue, parse_qualified_device,
    read_dwords_chunked, read_dwords_single_request, read_named, read_typed, write_typed,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, UdpSocket};

async fn udp_client() -> SlmpClient {
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.transport_mode = SlmpTransportMode::Udp;
    options.port = 9;
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
                    let response = build_4e_response_with_end_code(&request, end_code, &payload);
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

fn build_4e_response(request: &[u8], response_data: &[u8]) -> Vec<u8> {
    build_4e_response_with_end_code(request, 0, response_data)
}

fn build_4e_response_with_end_code(request: &[u8], end_code: u16, response_data: &[u8]) -> Vec<u8> {
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[0..2].copy_from_slice(&end_code.to_le_bytes());
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

fn build_dword_payload(values: &[u32]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(values.len() * 4);
    for value in values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload
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
