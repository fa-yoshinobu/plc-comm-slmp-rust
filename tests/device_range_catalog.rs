use plc_comm_slmp::{SlmpClient, SlmpConnectionOptions, SlmpPlcProfile};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn read_device_range_catalog_uses_configured_unit_family_sd_window() {
    let sd_values = [
        123u16, 456, 50000, 789, 50000, 50, 60, 70, 80, 90, 100, 110, 50000, 60000, 120,
    ];

    let server = MultiResponseServer::start(vec![build_word_payload(&sd_values)])
        .await
        .unwrap();

    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::QCpuQj71E71100,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let catalog = client.read_device_range_catalog().await.unwrap();

    assert_eq!(server.request_count().await, 1);
    assert_eq!(catalog.plc_profile, SlmpPlcProfile::QCpuQj71E71100);
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
async fn read_device_range_catalog_for_plc_profile_uses_only_profile_specific_sd_window() {
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

    let catalog = client
        .read_device_range_catalog_for_plc_profile(SlmpPlcProfile::IqF)
        .await
        .unwrap();

    assert_eq!(server.request_count().await, 1);
    assert!(!catalog.has_model_code);
    assert_eq!(catalog.model, "IQ-F");
    assert_eq!(catalog.plc_profile, SlmpPlcProfile::IqF);
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
async fn read_device_range_catalog_for_plc_profile_exposes_iql_profile() {
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

    let catalog = client
        .read_device_range_catalog_for_plc_profile(SlmpPlcProfile::IqL)
        .await
        .unwrap();

    assert_eq!(catalog.model, "iQ-L");
    assert_eq!(catalog.plc_profile, SlmpPlcProfile::IqL);
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
async fn read_device_range_catalog_for_plc_profile_caps_iqr_sd_point_counts() {
    let mut sd_values = vec![0u16; 50];
    set_dword(&mut sd_values, 0, 12_289);
    set_dword(&mut sd_values, 4, 94_674_945);
    set_dword(&mut sd_values, 20, 5_917_185);
    set_dword(&mut sd_values, 34, 1_479_297);
    set_dword(&mut sd_values, 38, 2_784_545);

    let server = MultiResponseServer::start(vec![build_word_payload(&sd_values)])
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

    let catalog = client
        .read_device_range_catalog_for_plc_profile(SlmpPlcProfile::IqR)
        .await
        .unwrap();

    assert_eq!(server.request_count().await, 1);
    assert_eq!(catalog.model, "IQ-R");
    assert_eq!(catalog.plc_profile, SlmpPlcProfile::IqR);
    assert_eq!(entry(&catalog, "X").point_count, Some(12_288));
    assert_eq!(
        entry(&catalog, "X").address_range.as_deref(),
        Some("X0000-X2FFF")
    );
    assert_eq!(entry(&catalog, "M").point_count, Some(94_674_944));
    assert_eq!(entry(&catalog, "D").point_count, Some(5_917_184));
    assert_eq!(entry(&catalog, "LTN").point_count, Some(1_479_296));
    assert_eq!(entry(&catalog, "LCN").point_count, Some(2_784_544));
}

#[tokio::test]
async fn read_device_range_catalog_for_iqr_unit_reports_unit_profile() {
    let mut sd_values = vec![0u16; 50];
    set_dword(&mut sd_values, 20, 0x0001_0034);

    let server = MultiResponseServer::start(vec![build_word_payload(&sd_values)])
        .await
        .unwrap();

    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqRRj71En71,
    )
    .unwrap();
    options.port = server.port;
    let client = SlmpClient::connect(options).await.unwrap();

    let catalog = client
        .read_device_range_catalog_for_plc_profile(SlmpPlcProfile::IqRRj71En71)
        .await
        .unwrap();

    assert_eq!(server.request_count().await, 1);
    assert_eq!(catalog.model, "iQ-R via RJ71EN71");
    assert_eq!(catalog.plc_profile, SlmpPlcProfile::IqRRj71En71);
    assert_eq!(entry(&catalog, "D").point_count, Some(0x0001_0034));
    assert_eq!(
        entry(&catalog, "D").address_range.as_deref(),
        Some("D0-D65587")
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

fn set_dword(values: &mut [u16], offset: usize, value: u32) {
    values[offset] = value as u16;
    values[offset + 1] = (value >> 16) as u16;
}

struct MultiResponseServer {
    port: u16,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
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

    async fn request_count(&self) -> usize {
        self.requests.lock().await.len()
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
