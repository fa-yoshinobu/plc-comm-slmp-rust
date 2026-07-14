use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpError, SlmpErrorKind,
    SlmpPlcProfile, SlmpTargetAddress, SlmpTransportMode,
};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::time::{Instant, sleep};

const TARGET: SlmpTargetAddress = SlmpTargetAddress {
    network: 0x12,
    station: 0x34,
    module_io: 0x5678,
    multidrop: 0x9A,
};

#[derive(Clone, Copy)]
enum TestFrame {
    ThreeE,
    FourE,
}

impl TestFrame {
    fn profile(self) -> SlmpPlcProfile {
        match self {
            Self::ThreeE => SlmpPlcProfile::IqF,
            Self::FourE => SlmpPlcProfile::IqR,
        }
    }

    fn request_header_size(self) -> usize {
        match self {
            Self::ThreeE => 9,
            Self::FourE => 13,
        }
    }

    fn request_length_offset(self) -> usize {
        match self {
            Self::ThreeE => 7,
            Self::FourE => 11,
        }
    }

    fn request_target_offset(self) -> usize {
        match self {
            Self::ThreeE => 2,
            Self::FourE => 6,
        }
    }

    fn response_target_offset(self) -> usize {
        self.request_target_offset()
    }
}

#[derive(Clone, Copy)]
enum RouteField {
    Network,
    Station,
    ModuleIo,
    Multidrop,
}

#[derive(Clone, Copy)]
enum TestTransport {
    Tcp,
    Udp,
}

async fn read_tcp_request(stream: &mut TcpStream, frame: TestFrame) -> Vec<u8> {
    let header_size = frame.request_header_size();
    let mut request = vec![0; header_size];
    stream.read_exact(&mut request).await.unwrap();
    let length_offset = frame.request_length_offset();
    let body_length =
        u16::from_le_bytes([request[length_offset], request[length_offset + 1]]) as usize;
    let mut body = vec![0; body_length];
    stream.read_exact(&mut body).await.unwrap();
    request.extend_from_slice(&body);
    request
}

fn build_response(request: &[u8], frame: TestFrame, word: u16) -> Vec<u8> {
    let payload = [0, 0, word as u8, (word >> 8) as u8];
    match frame {
        TestFrame::ThreeE => {
            let mut response = vec![0; 9 + payload.len()];
            response[0..2].copy_from_slice(&[0xD0, 0x00]);
            response[2..7].copy_from_slice(&request[2..7]);
            response[7..9].copy_from_slice(&(payload.len() as u16).to_le_bytes());
            response[9..].copy_from_slice(&payload);
            response
        }
        TestFrame::FourE => {
            let mut response = vec![0; 13 + payload.len()];
            response[0..2].copy_from_slice(&[0xD4, 0x00]);
            response[2..4].copy_from_slice(&request[2..4]);
            response[6..11].copy_from_slice(&request[6..11]);
            response[11..13].copy_from_slice(&(payload.len() as u16).to_le_bytes());
            response[13..].copy_from_slice(&payload);
            response
        }
    }
}

fn change_route(response: &mut [u8], frame: TestFrame, field: RouteField) {
    let offset = frame.response_target_offset();
    let byte_offset = match field {
        RouteField::Network => offset,
        RouteField::Station => offset + 1,
        RouteField::ModuleIo => offset + 2,
        RouteField::Multidrop => offset + 4,
    };
    response[byte_offset] ^= 0x01;
}

fn change_serial(response: &mut [u8]) {
    let serial = u16::from_le_bytes([response[2], response[3]]).wrapping_add(1);
    response[2..4].copy_from_slice(&serial.to_le_bytes());
}

fn options(
    port: u16,
    frame: TestFrame,
    transport: TestTransport,
    timeout: Duration,
) -> SlmpConnectionOptions {
    let mode = match transport {
        TestTransport::Tcp => SlmpTransportMode::Tcp,
        TestTransport::Udp => SlmpTransportMode::Udp,
    };
    let mut options =
        SlmpConnectionOptions::new("127.0.0.1", port, mode, TARGET, frame.profile()).unwrap();
    options.timeout = timeout;
    options
}

async fn read_one_word(
    client: &SlmpClient,
    profile: SlmpPlcProfile,
) -> Result<Vec<u16>, SlmpError> {
    client
        .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::D, 0, profile), 1)
        .await
}

async fn assert_route_mismatch_then_match(
    transport: TestTransport,
    frame: TestFrame,
    field: RouteField,
) {
    match transport {
        TestTransport::Tcp => {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_tcp_request(&mut stream, frame).await;
                let matching = build_response(&request, frame, 0x2345);
                let mut foreign = matching.clone();
                change_route(&mut foreign, frame, field);
                stream.write_all(&foreign).await.unwrap();
                stream.write_all(&matching).await.unwrap();
            });
            let client =
                SlmpClient::connect(options(port, frame, transport, Duration::from_millis(500)))
                    .await
                    .unwrap();
            assert_eq!(
                read_one_word(&client, frame.profile()).await.unwrap(),
                [0x2345]
            );
            assert_eq!(
                client.traffic_stats().await.rx_bytes,
                ((frame.request_header_size() + 4) * 2) as u64
            );
        }
        TestTransport::Udp => {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = socket.local_addr().unwrap().port();
            tokio::spawn(async move {
                let mut request = [0; 1024];
                let (length, peer) = socket.recv_from(&mut request).await.unwrap();
                let matching = build_response(&request[..length], frame, 0x2345);
                let mut foreign = matching.clone();
                change_route(&mut foreign, frame, field);
                socket.send_to(&foreign, peer).await.unwrap();
                socket.send_to(&matching, peer).await.unwrap();
            });
            let client =
                SlmpClient::connect(options(port, frame, transport, Duration::from_millis(500)))
                    .await
                    .unwrap();
            assert_eq!(
                read_one_word(&client, frame.profile()).await.unwrap(),
                [0x2345]
            );
            assert_eq!(
                client.traffic_stats().await.rx_bytes,
                ((frame.request_header_size() + 4) * 2) as u64
            );
        }
    }
}

async fn assert_all_route_fields(transport: TestTransport, frame: TestFrame) {
    for field in [
        RouteField::Network,
        RouteField::Station,
        RouteField::ModuleIo,
        RouteField::Multidrop,
    ] {
        assert_route_mismatch_then_match(transport, frame, field).await;
    }
}

#[tokio::test]
async fn tcp_3e_discards_each_foreign_route_field() {
    assert_all_route_fields(TestTransport::Tcp, TestFrame::ThreeE).await;
}

#[tokio::test]
async fn tcp_4e_discards_each_foreign_route_field() {
    assert_all_route_fields(TestTransport::Tcp, TestFrame::FourE).await;
}

#[tokio::test]
async fn udp_3e_discards_each_foreign_route_field() {
    assert_all_route_fields(TestTransport::Udp, TestFrame::ThreeE).await;
}

#[tokio::test]
async fn udp_4e_discards_each_foreign_route_field() {
    assert_all_route_fields(TestTransport::Udp, TestFrame::FourE).await;
}

async fn assert_wrong_serial_flood_obeys_deadline(transport: TestTransport) {
    let configured_timeout = Duration::from_millis(100);
    let client = match transport {
        TestTransport::Tcp => {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_tcp_request(&mut stream, TestFrame::FourE).await;
                let mut wrong = build_response(&request, TestFrame::FourE, 0x1111);
                change_serial(&mut wrong);
                for _ in 0..30 {
                    if stream.write_all(&wrong).await.is_err() {
                        break;
                    }
                    sleep(Duration::from_millis(10)).await;
                }
            });
            SlmpClient::connect(options(
                port,
                TestFrame::FourE,
                transport,
                configured_timeout,
            ))
            .await
            .unwrap()
        }
        TestTransport::Udp => {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = socket.local_addr().unwrap().port();
            tokio::spawn(async move {
                let mut request = [0; 1024];
                let (length, peer) = socket.recv_from(&mut request).await.unwrap();
                let mut wrong = build_response(&request[..length], TestFrame::FourE, 0x1111);
                change_serial(&mut wrong);
                for _ in 0..30 {
                    if socket.send_to(&wrong, peer).await.is_err() {
                        break;
                    }
                    sleep(Duration::from_millis(10)).await;
                }
            });
            SlmpClient::connect(options(
                port,
                TestFrame::FourE,
                transport,
                configured_timeout,
            ))
            .await
            .unwrap()
        }
    };

    let started = Instant::now();
    let error = read_one_word(&client, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    let elapsed = started.elapsed();
    assert_eq!(error.kind, SlmpErrorKind::Timeout);
    assert!(
        error.message.contains("timed out"),
        "unexpected error: {error}"
    );
    assert!(
        elapsed >= Duration::from_millis(90),
        "request timed out too early: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_millis(220),
        "unrelated responses extended the deadline: {elapsed:?}"
    );
    let closed = read_one_word(&client, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    assert!(closed.message.contains("transport is closed"));
}

#[tokio::test]
async fn wrong_serial_flood_cannot_extend_tcp_or_udp_deadline() {
    assert_wrong_serial_flood_obeys_deadline(TestTransport::Tcp).await;
    assert_wrong_serial_flood_obeys_deadline(TestTransport::Udp).await;
}

async fn assert_saturated_foreign_route_flood_obeys_deadline_and_new_session_is_clean(
    transport: TestTransport,
) {
    let configured_timeout = Duration::from_millis(80);
    let flood_duration = Duration::from_millis(300);
    let (port, server) = match transport {
        TestTransport::Tcp => {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            let server = tokio::spawn(async move {
                let (mut first, _) = listener.accept().await.unwrap();
                let request = read_tcp_request(&mut first, TestFrame::FourE).await;
                let mut foreign = build_response(&request, TestFrame::FourE, 0x1111);
                change_route(&mut foreign, TestFrame::FourE, RouteField::Network);
                let flood_until = Instant::now() + flood_duration;
                while Instant::now() < flood_until {
                    if first.write_all(&foreign).await.is_err() {
                        break;
                    }
                }

                let (mut second, _) = listener.accept().await.unwrap();
                let request = read_tcp_request(&mut second, TestFrame::FourE).await;
                let matching = build_response(&request, TestFrame::FourE, 0x5678);
                second.write_all(&matching).await.unwrap();
            });
            (port, server)
        }
        TestTransport::Udp => {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = socket.local_addr().unwrap().port();
            let server = tokio::spawn(async move {
                let mut request = [0; 1024];
                let (length, first_peer) = socket.recv_from(&mut request).await.unwrap();
                let mut foreign = build_response(&request[..length], TestFrame::FourE, 0x1111);
                change_route(&mut foreign, TestFrame::FourE, RouteField::Network);
                let flood_until = Instant::now() + flood_duration;
                while Instant::now() < flood_until {
                    socket.send_to(&foreign, first_peer).await.unwrap();
                }

                let (length, second_peer) = loop {
                    match socket.recv_from(&mut request).await {
                        Ok(received) => break received,
                        Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => {
                            continue;
                        }
                        Err(error) => panic!("unexpected UDP receive failure: {error}"),
                    }
                };
                let matching = build_response(&request[..length], TestFrame::FourE, 0x5678);
                socket.send_to(&matching, second_peer).await.unwrap();
            });
            (port, server)
        }
    };

    let first = SlmpClient::connect(options(
        port,
        TestFrame::FourE,
        transport,
        configured_timeout,
    ))
    .await
    .unwrap();
    let started = Instant::now();
    let error = read_one_word(&first, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    let elapsed = started.elapsed();

    assert_eq!(error.kind, SlmpErrorKind::Timeout);
    assert!(
        elapsed >= Duration::from_millis(70),
        "saturated flood timed out too early: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_millis(180),
        "saturated foreign responses starved the deadline: {elapsed:?}"
    );
    assert!(first.traffic_stats().await.rx_bytes > 0);
    let closed = read_one_word(&first, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    assert!(closed.message.contains("transport is closed"));

    let second = SlmpClient::connect(options(
        port,
        TestFrame::FourE,
        transport,
        Duration::from_millis(500),
    ))
    .await
    .unwrap();
    assert_eq!(
        read_one_word(&second, TestFrame::FourE.profile())
            .await
            .unwrap(),
        [0x5678]
    );
    server.await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn saturated_foreign_route_flood_cannot_starve_deadline_or_leak_to_new_session() {
    assert_saturated_foreign_route_flood_obeys_deadline_and_new_session_is_clean(
        TestTransport::Tcp,
    )
    .await;
    assert_saturated_foreign_route_flood_obeys_deadline_and_new_session_is_clean(
        TestTransport::Udp,
    )
    .await;
}

async fn assert_matching_serial_within_deadline_succeeds(transport: TestTransport) {
    let client = match transport {
        TestTransport::Tcp => {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_tcp_request(&mut stream, TestFrame::FourE).await;
                let matching = build_response(&request, TestFrame::FourE, 0x3456);
                let mut wrong = matching.clone();
                change_serial(&mut wrong);
                sleep(Duration::from_millis(20)).await;
                stream.write_all(&wrong).await.unwrap();
                sleep(Duration::from_millis(40)).await;
                stream.write_all(&matching).await.unwrap();
            });
            SlmpClient::connect(options(
                port,
                TestFrame::FourE,
                transport,
                Duration::from_millis(150),
            ))
            .await
            .unwrap()
        }
        TestTransport::Udp => {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = socket.local_addr().unwrap().port();
            tokio::spawn(async move {
                let mut request = [0; 1024];
                let (length, peer) = socket.recv_from(&mut request).await.unwrap();
                let matching = build_response(&request[..length], TestFrame::FourE, 0x3456);
                let mut wrong = matching.clone();
                change_serial(&mut wrong);
                sleep(Duration::from_millis(20)).await;
                socket.send_to(&wrong, peer).await.unwrap();
                sleep(Duration::from_millis(40)).await;
                socket.send_to(&matching, peer).await.unwrap();
            });
            SlmpClient::connect(options(
                port,
                TestFrame::FourE,
                transport,
                Duration::from_millis(150),
            ))
            .await
            .unwrap()
        }
    };

    assert_eq!(
        read_one_word(&client, TestFrame::FourE.profile())
            .await
            .unwrap(),
        [0x3456]
    );
}

#[tokio::test]
async fn matching_serial_before_absolute_deadline_succeeds_on_tcp_and_udp() {
    assert_matching_serial_within_deadline_succeeds(TestTransport::Tcp).await;
    assert_matching_serial_within_deadline_succeeds(TestTransport::Udp).await;
}

#[tokio::test]
async fn segmented_tcp_response_assembly_does_not_restart_the_deadline() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_tcp_request(&mut stream, TestFrame::FourE).await;
        let response = build_response(&request, TestFrame::FourE, 0x4567);
        sleep(Duration::from_millis(40)).await;
        if stream.write_all(&response[..2]).await.is_err() {
            return;
        }
        sleep(Duration::from_millis(40)).await;
        if stream.write_all(&response[2..13]).await.is_err() {
            return;
        }
        sleep(Duration::from_millis(40)).await;
        let _ = stream.write_all(&response[13..]).await;
    });
    let client = SlmpClient::connect(options(
        port,
        TestFrame::FourE,
        TestTransport::Tcp,
        Duration::from_millis(100),
    ))
    .await
    .unwrap();

    let started = Instant::now();
    let error = read_one_word(&client, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    let elapsed = started.elapsed();
    assert_eq!(error.kind, SlmpErrorKind::Timeout);
    assert!(error.message.contains("tcp read timed out"));
    assert!(
        elapsed >= Duration::from_millis(90),
        "segmented response timed out too early: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_millis(220),
        "segmented reads restarted the deadline: {elapsed:?}"
    );
    let closed = read_one_word(&client, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    assert!(closed.message.contains("transport is closed"));
}

async fn assert_malformed_frame_closes_transport(transport: TestTransport, frame: TestFrame) {
    let client = match transport {
        TestTransport::Tcp => {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_tcp_request(&mut stream, frame).await;
                let mut malformed = build_response(&request, frame, 0x1234);
                let length_offset = frame.request_length_offset();
                malformed[length_offset..length_offset + 2].copy_from_slice(&1u16.to_le_bytes());
                malformed.truncate(frame.request_header_size() + 1);
                stream.write_all(&malformed).await.unwrap();
            });
            SlmpClient::connect(options(port, frame, transport, Duration::from_millis(500)))
                .await
                .unwrap()
        }
        TestTransport::Udp => {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = socket.local_addr().unwrap().port();
            tokio::spawn(async move {
                let mut request = [0; 1024];
                let (length, peer) = socket.recv_from(&mut request).await.unwrap();
                let mut malformed = build_response(&request[..length], frame, 0x1234);
                let length_offset = frame.request_length_offset();
                malformed[length_offset..length_offset + 2].copy_from_slice(&1u16.to_le_bytes());
                malformed.truncate(frame.request_header_size() + 1);
                socket.send_to(&malformed, peer).await.unwrap();
            });
            SlmpClient::connect(options(port, frame, transport, Duration::from_millis(500)))
                .await
                .unwrap()
        }
    };

    let malformed = read_one_word(&client, frame.profile()).await.unwrap_err();
    assert!(malformed.message.contains("malformed response"));
    let closed = read_one_word(&client, frame.profile()).await.unwrap_err();
    assert!(closed.message.contains("transport is closed"));
}

#[tokio::test]
async fn malformed_tcp_and_udp_frames_close_the_transport() {
    for frame in [TestFrame::ThreeE, TestFrame::FourE] {
        assert_malformed_frame_closes_transport(TestTransport::Tcp, frame).await;
        assert_malformed_frame_closes_transport(TestTransport::Udp, frame).await;
    }
}

async fn assert_short_udp_datagram_is_malformed(frame: TestFrame) {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut request = [0; 1024];
        let (length, peer) = socket.recv_from(&mut request).await.unwrap();
        let response = build_response(&request[..length], frame, 0x1234);
        socket
            .send_to(&response[..frame.request_header_size() - 1], peer)
            .await
            .unwrap();
    });
    let client = SlmpClient::connect(options(
        port,
        frame,
        TestTransport::Udp,
        Duration::from_millis(500),
    ))
    .await
    .unwrap();

    let malformed = read_one_word(&client, frame.profile()).await.unwrap_err();
    assert!(malformed.message.contains("malformed response"));
    let closed = read_one_word(&client, frame.profile()).await.unwrap_err();
    assert!(closed.message.contains("transport is closed"));
}

#[tokio::test]
async fn short_udp_datagram_is_malformed_for_3e_and_4e() {
    assert_short_udp_datagram_is_malformed(TestFrame::ThreeE).await;
    assert_short_udp_datagram_is_malformed(TestFrame::FourE).await;
}

async fn assert_nonzero_4e_reserved_field_closes_transport(transport: TestTransport) {
    let client = match transport {
        TestTransport::Tcp => {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let port = listener.local_addr().unwrap().port();
            tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = read_tcp_request(&mut stream, TestFrame::FourE).await;
                let mut malformed = build_response(&request, TestFrame::FourE, 0x1234);
                malformed[4] = 1;
                stream.write_all(&malformed).await.unwrap();
            });
            SlmpClient::connect(options(
                port,
                TestFrame::FourE,
                transport,
                Duration::from_millis(500),
            ))
            .await
            .unwrap()
        }
        TestTransport::Udp => {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let port = socket.local_addr().unwrap().port();
            tokio::spawn(async move {
                let mut request = [0; 1024];
                let (length, peer) = socket.recv_from(&mut request).await.unwrap();
                let mut malformed = build_response(&request[..length], TestFrame::FourE, 0x1234);
                malformed[4] = 1;
                socket.send_to(&malformed, peer).await.unwrap();
            });
            SlmpClient::connect(options(
                port,
                TestFrame::FourE,
                transport,
                Duration::from_millis(500),
            ))
            .await
            .unwrap()
        }
    };

    let malformed = read_one_word(&client, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    assert!(malformed.message.contains("reserved field"));
    let closed = read_one_word(&client, TestFrame::FourE.profile())
        .await
        .unwrap_err();
    assert!(closed.message.contains("transport is closed"));
}

#[tokio::test]
async fn nonzero_4e_reserved_field_is_malformed_on_tcp_and_udp() {
    assert_nonzero_4e_reserved_field_closes_transport(TestTransport::Tcp).await;
    assert_nonzero_4e_reserved_field_closes_transport(TestTransport::Udp).await;
}

#[tokio::test]
async fn timeout_too_large_for_an_absolute_deadline_is_rejected_before_transport() {
    let mut options = options(
        9,
        TestFrame::FourE,
        TestTransport::Udp,
        Duration::from_secs(1),
    );
    options.timeout = Duration::MAX;

    let error = match SlmpClient::connect(options).await {
        Ok(_) => panic!("excessive timeout must be rejected"),
        Err(error) => error,
    };

    assert!(error.message.contains("timeout is too large"));
    assert_eq!(error.kind, SlmpErrorKind::General);
    assert!(!error.is_timeout());
}
