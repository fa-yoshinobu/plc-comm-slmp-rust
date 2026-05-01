use plc_comm_slmp::{
    SlmpClient, SlmpCompatibilityMode, SlmpConnectionOptions, SlmpDeviceRangeFamily, SlmpFrameType,
    SlmpPlcFamily, read_device_range_catalog_with_three_e_legacy_fallback,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn read_device_range_catalog_uses_configured_family_sd_window() {
    let sd_values = [
        123u16, 456, 50000, 789, 50000, 50, 60, 70, 80, 90, 100, 110, 50000, 60000, 120,
    ];

    let server = MultiResponseServer::start(vec![build_word_payload(&sd_values)])
        .await
        .unwrap();

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::QCpu);
    options.port = server.port;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;
    let client = SlmpClient::connect(options).await.unwrap();

    let catalog = client.read_device_range_catalog().await.unwrap();

    assert_eq!(server.request_count().await, 1);
    assert_eq!(catalog.family, SlmpDeviceRangeFamily::QCpu);
    assert_eq!(entry(&catalog, "X").point_count, Some(123));
    assert_eq!(entry(&catalog, "X").upper_bound, Some(122));
    assert_eq!(
        entry(&catalog, "X").address_range.as_deref(),
        Some("X000-X07A")
    );
    assert_eq!(entry(&catalog, "M").point_count, Some(32768));
    assert_eq!(entry(&catalog, "M").upper_bound, Some(32767));
    assert_eq!(entry(&catalog, "D").point_count, Some(32768));
    assert_eq!(entry(&catalog, "D").upper_bound, Some(32767));
    assert_eq!(entry(&catalog, "SW").point_count, Some(120));
    assert_eq!(entry(&catalog, "SW").upper_bound, Some(119));
    assert_eq!(
        entry(&catalog, "SW").address_range.as_deref(),
        Some("SW000-SW077")
    );
    assert_eq!(entry(&catalog, "Z").point_count, Some(10));
    assert_eq!(entry(&catalog, "Z").upper_bound, Some(9));
}

#[tokio::test]
async fn read_device_range_catalog_for_family_uses_only_family_specific_sd_window() {
    let mut sd_values = vec![0u16; 46];
    sd_values[0] = 1024;
    sd_values[2] = 1024;
    sd_values[4] = 7680;
    sd_values[10] = 8000;
    sd_values[20] = 10000;
    sd_values[22] = 12000;

    let server = MultiResponseServer::start(vec![build_word_payload(&sd_values)])
        .await
        .unwrap();

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqF);
    options.port = server.port;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;
    let client = SlmpClient::connect(options).await.unwrap();

    let catalog = client
        .read_device_range_catalog_for_family(SlmpDeviceRangeFamily::IqF)
        .await
        .unwrap();

    assert_eq!(server.request_count().await, 1);
    assert!(!catalog.has_model_code);
    assert_eq!(catalog.model, "IQ-F");
    assert_eq!(catalog.family, SlmpDeviceRangeFamily::IqF);
    assert_eq!(
        entry(&catalog, "X").address_range.as_deref(),
        Some("X0000-X1777")
    );
    assert_eq!(
        entry(&catalog, "D").address_range.as_deref(),
        Some("D0-D9999")
    );
    assert_eq!(
        entry(&catalog, "SD").address_range.as_deref(),
        Some("SD0-SD11999")
    );
}

#[tokio::test]
async fn read_device_range_catalog_for_family_exposes_iql_family() {
    let mut sd_values = vec![0u16; 50];
    sd_values[0] = 0x3000;
    sd_values[2] = 0x3000;
    sd_values[4] = 12288;
    sd_values[6] = 0x2000;
    sd_values[20] = 18432;
    sd_values[22] = 0x2000;
    sd_values[24] = 0x0800;
    sd_values[28] = 2048;
    sd_values[30] = 32;
    sd_values[32] = 512;
    sd_values[34] = 1024;
    sd_values[36] = 32;
    sd_values[38] = 512;
    sd_values[40] = 20;
    sd_values[42] = 2;
    sd_values[46] = 0xffff;
    sd_values[47] = 0x000b;
    sd_values[48] = 0x0000;
    sd_values[49] = 0x0008;

    let server = MultiResponseServer::start(vec![build_word_payload(&sd_values)])
        .await
        .unwrap();

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqL);
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let catalog = client
        .read_device_range_catalog_for_family(SlmpDeviceRangeFamily::IqL)
        .await
        .unwrap();

    assert_eq!(catalog.model, "iQ-L");
    assert_eq!(catalog.family, SlmpDeviceRangeFamily::IqL);
    assert_eq!(
        entry(&catalog, "SM").address_range.as_deref(),
        Some("SM0-SM4095")
    );
    assert_eq!(
        entry(&catalog, "SD").address_range.as_deref(),
        Some("SD0-SD4095")
    );
    assert_eq!(
        entry(&catalog, "D").address_range.as_deref(),
        Some("D0-D18431")
    );
    assert_eq!(
        entry(&catalog, "LZ").address_range.as_deref(),
        Some("LZ0-LZ1")
    );
    assert_eq!(
        entry(&catalog, "LTN").address_range.as_deref(),
        Some("LTN0-LTN1023")
    );
    assert_eq!(
        entry(&catalog, "LSTN").address_range.as_deref(),
        Some("LSTN0-LSTN31")
    );
    assert_eq!(
        entry(&catalog, "LCN").address_range.as_deref(),
        Some("LCN0-LCN511")
    );
}

#[tokio::test]
async fn read_device_range_catalog_falls_back_to_three_e_legacy_when_type_name_does_not_return() {
    let mut sd_values = vec![0u16; 46];
    sd_values[0] = 1024;
    sd_values[2] = 1024;
    sd_values[4] = 7680;
    sd_values[10] = 8000;
    sd_values[20] = 10000;
    sd_values[22] = 12000;

    let server = FallbackServer::start(build_word_payload(&sd_values))
        .await
        .unwrap();

    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqF);
    options.port = server.port;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;
    options.timeout = std::time::Duration::from_secs(1);

    let resolved = read_device_range_catalog_with_three_e_legacy_fallback(&options)
        .await
        .unwrap();

    assert!(resolved.used_three_e_legacy_fallback);
    assert_eq!(resolved.frame_type, SlmpFrameType::Frame3E);
    assert_eq!(resolved.compatibility_mode, SlmpCompatibilityMode::Legacy);
    assert_eq!(resolved.catalog.family, SlmpDeviceRangeFamily::IqF);
    assert_eq!(resolved.catalog.model, "IQ-F");
    assert_eq!(
        entry(&resolved.catalog, "X").address_range.as_deref(),
        Some("X0000-X1777")
    );
}

fn entry<'a>(
    catalog: &'a plc_comm_slmp::SlmpDeviceRangeCatalog,
    device: &str,
) -> &'a plc_comm_slmp::SlmpDeviceRangeEntry {
    catalog
        .entries
        .iter()
        .find(|item| item.device == device)
        .unwrap()
}

fn build_word_payload(values: &[u16]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(values.len() * 2);
    for value in values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload
}

struct MultiResponseServer {
    port: u16,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

struct FallbackServer {
    port: u16,
}

impl FallbackServer {
    async fn start(sd_payload: Vec<u8>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        tokio::spawn(async move {
            if let Ok((stream, _)) = listener.accept().await {
                drop(stream);
            } else {
                return;
            }

            if let Ok((mut stream, _)) = listener.accept().await {
                let sd_request = match read_3e_request(&mut stream).await {
                    Ok(request) => request,
                    Err(_) => return,
                };
                let sd_response = build_3e_response(&sd_request, &sd_payload);
                let _ = stream.write_all(&sd_response).await;
            }
        });
        Ok(Self { port })
    }
}

impl MultiResponseServer {
    async fn start(response_payloads: Vec<Vec<u8>>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_sink = requests.clone();
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

    async fn request_count(&self) -> usize {
        self.requests.lock().await.len()
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

async fn read_3e_request(stream: &mut tokio::net::TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut header = [0u8; 15];
    stream.read_exact(&mut header).await?;
    let body_len = u16::from_le_bytes([header[7], header[8]]) as usize - 6;
    let mut body = vec![0u8; body_len];
    stream.read_exact(&mut body).await?;
    let mut request = header.to_vec();
    request.extend_from_slice(&body);
    Ok(request)
}

fn build_3e_response(request: &[u8], response_data: &[u8]) -> Vec<u8> {
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[2..].copy_from_slice(response_data);

    let mut response = vec![0u8; 9 + payload.len()];
    response[0] = 0xD0;
    response[1] = 0x00;
    response[2..7].copy_from_slice(&request[2..7]);
    response[7..9].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    response[9..].copy_from_slice(&payload);
    response
}
