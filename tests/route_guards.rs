use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpExtensionSpec,
    SlmpPlcFamily, SlmpQualifiedDeviceAddress, SlmpTransportMode, SlmpValue, read_dwords_chunked,
    read_dwords_single_request, read_named, read_typed, write_typed,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn udp_client() -> SlmpClient {
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqR);
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

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqL);
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
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqL);
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
        read_dwords_single_request(&client, SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0), 256)
            .await
            .unwrap_err();
    assert!(err.to_string().contains("1-255"));

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
