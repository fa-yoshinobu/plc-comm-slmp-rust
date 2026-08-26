use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpErrorKind,
    SlmpPlcProfile,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn qcpu_unit_catalog_uses_base_rules_in_one_client_turn() {
    let sd_values = [
        123u16, 456, 50000, 789, 50000, 50, 60, 70, 80, 90, 100, 110, 50000, 60000, 120,
    ];

    let server = MultiResponseServer::start_with_first_response_delay(
        vec![
            (0, build_word_payload(&sd_values)),
            (0xD123, Vec::new()),
            (0xC061, Vec::new()),
            (0, build_word_payload(&[0x1234])),
        ],
        std::time::Duration::from_millis(100),
    )
    .await
    .unwrap();

    let client = connect_client(server.port, SlmpPlcProfile::QCpuQj71E71100).await;

    let catalog_client = client.clone();
    let catalog_task =
        tokio::spawn(async move { catalog_client.read_device_range_catalog().await });
    wait_for_request_count(&server, 1).await;
    let user_read = tokio::spawn(async move {
        client
            .read_words_raw(
                SlmpDeviceAddress::new(SlmpDeviceCode::D, 999, SlmpPlcProfile::QCpuQj71E71100),
                1,
            )
            .await
    });
    let catalog = catalog_task.await.unwrap().unwrap();
    assert_eq!(user_read.await.unwrap().unwrap(), vec![0x1234]);

    assert_eq!(server.request_count().await, 4);
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
    assert_eq!(entry(&catalog, "ZR").point_count, Some(0));
    assert_eq!(entry(&catalog, "ZR").upper_bound, None);
    assert_eq!(entry(&catalog, "R").point_count, Some(0));

    let requests = server.requests().await;
    assert_direct_read(&requests[0], SlmpDeviceCode::SD, 290, 15);
    assert_direct_read(&requests[1], SlmpDeviceCode::Z, 15, 1);
    assert_direct_read(&requests[2], SlmpDeviceCode::ZR, 0, 1);
    assert_direct_read(&requests[3], SlmpDeviceCode::D, 999, 1);
}

#[tokio::test]
async fn qnudv_profiles_probe_zr_with_the_canonical_request_sequence() {
    for plc_profile in [SlmpPlcProfile::QnUDV, SlmpPlcProfile::QnUDVQj71E71100] {
        let sd_values = vec![0u16; 26];
        let mut responses = vec![(0, build_word_payload(&sd_values))];
        responses.extend((0..5).map(|_| (0, build_word_payload(&[0]))));
        responses.extend((0..5).map(|_| (0x4031, Vec::new())));
        let server = MultiResponseServer::start_with_end_codes(responses)
            .await
            .unwrap();

        let client = connect_client(server.port, plc_profile).await;

        let catalog = client
            .read_device_range_catalog_for_plc_profile(plc_profile)
            .await
            .unwrap();

        assert_eq!(catalog.plc_profile, plc_profile);
        assert_eq!(entry(&catalog, "Z").point_count, Some(20));
        assert_eq!(entry(&catalog, "ZR").point_count, Some(16));
        assert_eq!(entry(&catalog, "ZR").upper_bound, Some(15));
        assert_eq!(entry(&catalog, "ZR").source, "Runtime access check");
        assert_eq!(entry(&catalog, "R").point_count, Some(16));
        assert_eq!(entry(&catalog, "R").upper_bound, Some(15));

        let requests = server.requests().await;
        assert_eq!(requests.len(), 11);
        assert_direct_read(&requests[0], SlmpDeviceCode::SD, 286, 26);
        let expected_zr_addresses = [0, 1, 3, 7, 15, 31, 23, 19, 17, 16];
        for (request, address) in requests[1..].iter().zip(expected_zr_addresses) {
            assert_direct_read(request, SlmpDeviceCode::ZR, address, 1);
        }
    }
}

#[tokio::test]
async fn qcpu_runtime_probe_uses_z16_and_caps_zr_and_derived_r() {
    let sd_values = vec![0u16; 15];
    let mut responses = vec![(0, build_word_payload(&sd_values))];
    responses.push((0, build_word_payload(&[0]))); // Z15
    responses.extend((0..21).map(|_| (0, build_word_payload(&[0]))));
    let server = MultiResponseServer::start_with_end_codes(responses)
        .await
        .unwrap();

    let client = connect_client(server.port, SlmpPlcProfile::QCpuQj71E71100).await;

    let catalog = client.read_device_range_catalog().await.unwrap();

    assert_eq!(entry(&catalog, "Z").point_count, Some(16));
    assert_eq!(entry(&catalog, "ZR").point_count, Some(1_048_576));
    assert_eq!(entry(&catalog, "ZR").upper_bound, Some(1_048_575));
    assert_eq!(entry(&catalog, "R").point_count, Some(32_768));
    assert_eq!(entry(&catalog, "R").upper_bound, Some(32_767));

    let requests = server.requests().await;
    assert_eq!(requests.len(), 23);
    assert_direct_read(&requests[0], SlmpDeviceCode::SD, 290, 15);
    assert_direct_read(&requests[1], SlmpDeviceCode::Z, 15, 1);
    assert_direct_read(requests.last().unwrap(), SlmpDeviceCode::ZR, 1_048_575, 1);
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

#[tokio::test]
async fn device_range_catalog_propagates_plc_end_codes_without_boundary_inference() {
    for end_code in [0xC061, 0xC200, 0xCEE0, 0xD123] {
        let server = MultiResponseServer::start_with_end_codes(vec![(end_code, Vec::new())])
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

        let error = client.read_device_range_catalog().await.unwrap_err();

        assert_eq!(error.end_code, Some(end_code));
        assert_eq!(server.request_count().await, 1);
    }
}

#[tokio::test]
async fn device_range_catalog_propagates_protocol_failure_without_a_probe() {
    let server = MultiResponseServer::start(vec![vec![0x12]]).await.unwrap();
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

    let error = client.read_device_range_catalog().await.unwrap_err();

    assert!(
        error.message.contains("payload size mismatch"),
        "unexpected protocol error: {}",
        error.message
    );
    assert_eq!(server.request_count().await, 1);
}

#[tokio::test]
async fn device_range_catalog_propagates_timeout_without_a_probe() {
    let server = MultiResponseServer::start_silent(std::time::Duration::from_millis(100))
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
    options.timeout = std::time::Duration::from_millis(10);
    let client = SlmpClient::connect(options).await.unwrap();

    let error = client.read_device_range_catalog().await.unwrap_err();

    assert!(error.is_timeout());
    assert_eq!(server.request_count().await, 1);
}

#[tokio::test]
async fn runtime_probe_propagates_malformed_success_instead_of_inferring_a_boundary() {
    let server = MultiResponseServer::start(vec![build_word_payload(&[0u16; 26]), vec![0x12]])
        .await
        .unwrap();
    let client = connect_client(server.port, SlmpPlcProfile::QnUDV).await;

    let error = client.read_device_range_catalog().await.unwrap_err();

    assert_eq!(error.kind, SlmpErrorKind::MalformedResponse);
    assert!(error.message.contains("payload size mismatch"));
    assert_eq!(server.request_count().await, 2);
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

async fn connect_client(port: u16, plc_profile: SlmpPlcProfile) -> SlmpClient {
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        port,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        plc_profile,
    )
    .unwrap();
    SlmpClient::connect(options).await.unwrap()
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
        Self::start_with_end_codes(
            response_payloads
                .into_iter()
                .map(|payload| (0, payload))
                .collect(),
        )
        .await
    }

    async fn start_with_end_codes(responses: Vec<(u16, Vec<u8>)>) -> std::io::Result<Self> {
        Self::start_with_first_response_delay(responses, std::time::Duration::ZERO).await
    }

    async fn start_with_first_response_delay(
        responses: Vec<(u16, Vec<u8>)>,
        first_response_delay: std::time::Duration,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_sink = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut pending = std::collections::VecDeque::from(responses);
                let mut first_response = true;
                while let Some((end_code, payload)) = pending.pop_front() {
                    let Some(request) = read_request(&mut stream).await else {
                        return;
                    };
                    request_sink.lock().await.push(request.clone());

                    if first_response {
                        tokio::time::sleep(first_response_delay).await;
                        first_response = false;
                    }

                    let response = build_response(&request, end_code, &payload);
                    if stream.write_all(&response).await.is_err() {
                        return;
                    }
                }
            }
        });
        Ok(Self { port, requests })
    }

    async fn start_silent(delay: std::time::Duration) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_sink = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                if let Some(request) = read_request(&mut stream).await {
                    request_sink.lock().await.push(request);
                    tokio::time::sleep(delay).await;
                }
            }
        });
        Ok(Self { port, requests })
    }

    async fn request_count(&self) -> usize {
        self.requests.lock().await.len()
    }

    async fn requests(&self) -> Vec<Vec<u8>> {
        self.requests.lock().await.clone()
    }
}

async fn wait_for_request_count(server: &MultiResponseServer, expected: usize) {
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        loop {
            if server.request_count().await >= expected {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("server did not observe the expected request count");
}

fn assert_direct_read(request: &[u8], code: SlmpDeviceCode, number: u32, points: u16) {
    let header_size = match &request[..2] {
        [0x54, 0x00] => 13,
        [0x50, 0x00] => 9,
        prefix => panic!("unexpected request prefix: {prefix:02X?}"),
    };
    let body = &request[header_size..];
    assert_eq!(u16::from_le_bytes([body[2], body[3]]), 0x0401);
    assert_eq!(u16::from_le_bytes([body[4], body[5]]), 0x0000);
    assert_eq!(u32::from_le_bytes([body[6], body[7], body[8], 0]), number);
    assert_eq!(body[9], code.as_u8());
    assert_eq!(u16::from_le_bytes([body[10], body[11]]), points);
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

fn build_response(request: &[u8], end_code: u16, response_data: &[u8]) -> Vec<u8> {
    let mut payload = vec![0u8; 2 + response_data.len()];
    payload[..2].copy_from_slice(&end_code.to_le_bytes());
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
