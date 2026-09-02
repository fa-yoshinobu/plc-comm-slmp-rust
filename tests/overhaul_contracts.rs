use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpErrorKind,
    SlmpOutcomeUnknownReason, SlmpPlcProfile, SlmpTargetAddress, SlmpTransportMode,
    parse_qualified_device, write_bit_in_word, write_bit_in_word_extended,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, oneshot};

fn options(port: u16, timeout: Duration) -> SlmpConnectionOptions {
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        port,
        SlmpTransportMode::Tcp,
        SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.timeout = timeout;
    options
}

fn device(number: u32) -> SlmpDeviceAddress {
    SlmpDeviceAddress::new(SlmpDeviceCode::D, number, SlmpPlcProfile::IqR)
}

async fn read_request(stream: &mut TcpStream) -> Vec<u8> {
    let mut header = vec![0; 13];
    stream.read_exact(&mut header).await.unwrap();
    let length = u16::from_le_bytes([header[11], header[12]]) as usize;
    let mut body = vec![0; length];
    stream.read_exact(&mut body).await.unwrap();
    header.extend_from_slice(&body);
    header
}

fn command(request: &[u8]) -> u16 {
    u16::from_le_bytes([request[15], request[16]])
}

fn head_device(request: &[u8]) -> u32 {
    u32::from_le_bytes([request[19], request[20], request[21], request[22]])
}

fn response(request: &[u8], end_code: u16, payload: &[u8]) -> Vec<u8> {
    let mut response = vec![0; 15 + payload.len()];
    response[0..2].copy_from_slice(&[0xD4, 0x00]);
    response[2..4].copy_from_slice(&request[2..4]);
    response[6..11].copy_from_slice(&request[6..11]);
    response[11..13].copy_from_slice(&((2 + payload.len()) as u16).to_le_bytes());
    response[13..15].copy_from_slice(&end_code.to_le_bytes());
    response[15..].copy_from_slice(payload);
    response
}

#[test]
fn public_error_kinds_and_outcome_reasons_are_pairwise_distinct() {
    let kinds = [
        SlmpErrorKind::General,
        SlmpErrorKind::Timeout,
        SlmpErrorKind::Cancelled,
        SlmpErrorKind::Closed,
        SlmpErrorKind::NotConnected,
        SlmpErrorKind::Transport,
        SlmpErrorKind::MalformedResponse,
        SlmpErrorKind::PlcEndCode,
        SlmpErrorKind::ProfileFeature,
        SlmpErrorKind::OutcomeUnknown,
    ];
    for (index, left) in kinds.iter().enumerate() {
        for right in &kinds[index + 1..] {
            assert_ne!(left, right);
        }
    }

    let reasons = [
        SlmpOutcomeUnknownReason::Timeout,
        SlmpOutcomeUnknownReason::Cancelled,
        SlmpOutcomeUnknownReason::Closed,
        SlmpOutcomeUnknownReason::Transport,
        SlmpOutcomeUnknownReason::MalformedResponse,
    ];
    for (index, left) in reasons.iter().enumerate() {
        for right in &reasons[index + 1..] {
            assert_ne!(left, right);
        }
    }
}

#[tokio::test]
async fn close_rejects_active_write_as_unknown_and_queued_read_without_sending() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (seen_tx, seen_rx) = oneshot::channel();
    let request_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server_count = request_count.clone();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let _request = read_request(&mut stream).await;
        server_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        seen_tx.send(()).unwrap();
        let mut next = [0; 1];
        let _ = stream.read(&mut next).await;
    });

    let client = SlmpClient::connect(options(port, Duration::from_secs(2)))
        .await
        .unwrap();
    let active_client = client.clone();
    let active = tokio::spawn(async move { active_client.write_words(device(1), &[7]).await });
    seen_rx.await.unwrap();

    let queued_client = client.clone();
    let queued = tokio::spawn(async move { queued_client.read_words(device(2), 1).await });
    tokio::task::yield_now().await;
    tokio::time::timeout(Duration::from_millis(100), client.close())
        .await
        .expect("close must not wait behind the active operation")
        .unwrap();

    let active_error = active.await.unwrap().unwrap_err();
    assert_eq!(active_error.kind, SlmpErrorKind::OutcomeUnknown);
    assert_eq!(
        active_error.outcome_unknown_reason,
        Some(SlmpOutcomeUnknownReason::Closed)
    );
    assert_eq!(
        queued.await.unwrap().unwrap_err().kind,
        SlmpErrorKind::Closed
    );
    server.await.unwrap();
    assert_eq!(request_count.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn cancelling_a_waiter_sends_nothing_and_does_not_delay_its_successor() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (first_seen_tx, first_seen_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let server_observed = observed.clone();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let first = read_request(&mut stream).await;
        server_observed.lock().await.push(head_device(&first));
        first_seen_tx.send(()).unwrap();
        release_rx.await.unwrap();
        stream
            .write_all(&response(&first, 0, &[1, 0]))
            .await
            .unwrap();

        let second = read_request(&mut stream).await;
        server_observed.lock().await.push(head_device(&second));
        stream
            .write_all(&response(&second, 0, &[2, 0]))
            .await
            .unwrap();
    });

    let client = SlmpClient::connect(options(port, Duration::from_secs(2)))
        .await
        .unwrap();
    let active_client = client.clone();
    let active = tokio::spawn(async move { active_client.read_words(device(10), 1).await });
    first_seen_rx.await.unwrap();

    let cancelled_client = client.clone();
    let (started_tx, started_rx) = oneshot::channel();
    let cancelled = tokio::spawn(async move {
        started_tx.send(()).unwrap();
        cancelled_client.read_words(device(11), 1).await
    });
    started_rx.await.unwrap();
    tokio::task::yield_now().await;
    cancelled.abort();
    assert!(cancelled.await.unwrap_err().is_cancelled());

    let successor_client = client.clone();
    let successor = tokio::spawn(async move { successor_client.read_words(device(12), 1).await });
    release_tx.send(()).unwrap();
    assert_eq!(active.await.unwrap().unwrap(), vec![1]);
    assert_eq!(successor.await.unwrap().unwrap(), vec![2]);
    server.await.unwrap();
    assert_eq!(*observed.lock().await, vec![10, 12]);
}

#[tokio::test]
async fn separate_client_instances_make_progress_independently() {
    let slow_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let slow_port = slow_listener.local_addr().unwrap().port();
    let fast_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let fast_port = fast_listener.local_addr().unwrap().port();
    let (slow_seen_tx, slow_seen_rx) = oneshot::channel();
    let (slow_release_tx, slow_release_rx) = oneshot::channel();

    let slow_server = tokio::spawn(async move {
        let (mut stream, _) = slow_listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        slow_seen_tx.send(()).unwrap();
        slow_release_rx.await.unwrap();
        stream
            .write_all(&response(&request, 0, &[1, 0]))
            .await
            .unwrap();
    });
    let fast_server = tokio::spawn(async move {
        let (mut stream, _) = fast_listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        stream
            .write_all(&response(&request, 0, &[2, 0]))
            .await
            .unwrap();
    });

    let slow = SlmpClient::connect(options(slow_port, Duration::from_secs(2)))
        .await
        .unwrap();
    let fast = SlmpClient::connect(options(fast_port, Duration::from_secs(2)))
        .await
        .unwrap();
    let slow_call = tokio::spawn(async move { slow.read_words(device(1), 1).await });
    slow_seen_rx.await.unwrap();
    assert_eq!(
        tokio::time::timeout(Duration::from_millis(100), fast.read_words(device(2), 1))
            .await
            .unwrap()
            .unwrap(),
        vec![2]
    );
    slow_release_tx.send(()).unwrap();
    assert_eq!(slow_call.await.unwrap().unwrap(), vec![1]);
    slow_server.await.unwrap();
    fast_server.await.unwrap();
}

#[tokio::test]
async fn bit_in_word_rmw_owns_one_fifo_turn() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (first_seen_tx, first_seen_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let server_observed = observed.clone();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let read = read_request(&mut stream).await;
        server_observed
            .lock()
            .await
            .push((command(&read), head_device(&read)));
        first_seen_tx.send(()).unwrap();
        release_rx.await.unwrap();
        stream
            .write_all(&response(&read, 0, &[1, 0]))
            .await
            .unwrap();

        let write = read_request(&mut stream).await;
        server_observed
            .lock()
            .await
            .push((command(&write), head_device(&write)));
        assert_eq!(&write[27..29], &[1, 0]);
        stream.write_all(&response(&write, 0, &[])).await.unwrap();

        let queued = read_request(&mut stream).await;
        server_observed
            .lock()
            .await
            .push((command(&queued), head_device(&queued)));
        stream
            .write_all(&response(&queued, 0, &[3, 0]))
            .await
            .unwrap();
    });

    let client = SlmpClient::connect(options(port, Duration::from_secs(2)))
        .await
        .unwrap();
    let rmw_client = client.clone();
    let rmw =
        tokio::spawn(async move { write_bit_in_word(&rmw_client, device(100), 0, true).await });
    first_seen_rx.await.unwrap();
    let queued_client = client.clone();
    let queued = tokio::spawn(async move { queued_client.read_words(device(200), 1).await });
    tokio::task::yield_now().await;
    release_tx.send(()).unwrap();
    rmw.await.unwrap().unwrap();
    assert_eq!(queued.await.unwrap().unwrap(), vec![3]);
    server.await.unwrap();
    assert_eq!(
        *observed.lock().await,
        vec![(0x0401, 100), (0x1401, 100), (0x0401, 200)]
    );
}

#[tokio::test]
async fn bit_in_word_rmw_uses_one_absolute_deadline_for_read_and_write() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let server_observed = observed.clone();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let read = read_request(&mut stream).await;
        server_observed.lock().await.push(command(&read));
        tokio::time::sleep(Duration::from_millis(70)).await;
        stream
            .write_all(&response(&read, 0, &[0, 0]))
            .await
            .unwrap();

        let write = read_request(&mut stream).await;
        server_observed.lock().await.push(command(&write));
        tokio::time::sleep(Duration::from_millis(70)).await;
        let _ = stream.write_all(&response(&write, 0, &[])).await;
    });

    let client = SlmpClient::connect(options(port, Duration::from_millis(100)))
        .await
        .unwrap();
    let error = write_bit_in_word(&client, device(100), 0, true)
        .await
        .unwrap_err();

    assert_eq!(error.kind, SlmpErrorKind::OutcomeUnknown);
    assert_eq!(
        error.outcome_unknown_reason,
        Some(SlmpOutcomeUnknownReason::Timeout)
    );
    server.await.unwrap();
    assert_eq!(*observed.lock().await, vec![0x0401, 0x1401]);
}

#[tokio::test]
async fn qualified_bit_in_word_routes_each_use_one_absolute_deadline() {
    for address in [r"U1\G0", r"J1\W10"] {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let observed = Arc::new(Mutex::new(Vec::new()));
        let server_observed = observed.clone();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let read = read_request(&mut stream).await;
            server_observed.lock().await.push(command(&read));
            tokio::time::sleep(Duration::from_millis(70)).await;
            stream
                .write_all(&response(&read, 0, &[0x08, 0x00]))
                .await
                .unwrap();

            let write = read_request(&mut stream).await;
            server_observed.lock().await.push(command(&write));
            tokio::time::sleep(Duration::from_millis(70)).await;
            let _ = stream.write_all(&response(&write, 0, &[])).await;
        });

        let client = SlmpClient::connect(options(port, Duration::from_millis(100)))
            .await
            .unwrap();
        let qualified = parse_qualified_device(address, SlmpPlcProfile::IqR).unwrap();
        let error = write_bit_in_word_extended(&client, qualified, 3, true)
            .await
            .unwrap_err();

        assert_eq!(error.kind, SlmpErrorKind::OutcomeUnknown);
        assert_eq!(
            error.outcome_unknown_reason,
            Some(SlmpOutcomeUnknownReason::Timeout)
        );
        server.await.unwrap();
        assert_eq!(*observed.lock().await, vec![0x0401, 0x1401]);
    }
}

#[tokio::test]
async fn plc_ng_is_definitive_but_malformed_write_is_outcome_unknown() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        let (mut first_stream, _) = listener.accept().await.unwrap();
        let write = read_request(&mut first_stream).await;
        first_stream
            .write_all(&response(&write, 0xC051, &[]))
            .await
            .unwrap();
        let read = read_request(&mut first_stream).await;
        first_stream
            .write_all(&response(&read, 0, &[9, 0]))
            .await
            .unwrap();

        let (mut second_stream, _) = listener.accept().await.unwrap();
        let malformed_write = read_request(&mut second_stream).await;
        second_stream
            .write_all(&response(&malformed_write, 0, &[0xAA]))
            .await
            .unwrap();
    });

    let definitive = SlmpClient::connect(options(port, Duration::from_secs(2)))
        .await
        .unwrap();
    let plc_ng = definitive.write_words(device(1), &[1]).await.unwrap_err();
    assert_eq!(plc_ng.kind, SlmpErrorKind::PlcEndCode);
    assert_eq!(plc_ng.end_code, Some(0xC051));
    assert_eq!(definitive.read_words(device(2), 1).await.unwrap(), vec![9]);

    let unknown = SlmpClient::connect(options(port, Duration::from_secs(2)))
        .await
        .unwrap();
    let malformed = unknown.write_words(device(3), &[1]).await.unwrap_err();
    assert_eq!(malformed.kind, SlmpErrorKind::OutcomeUnknown);
    assert_eq!(
        malformed.outcome_unknown_reason,
        Some(SlmpOutcomeUnknownReason::MalformedResponse)
    );
    assert_eq!(
        unknown.read_words(device(4), 1).await.unwrap_err().kind,
        SlmpErrorKind::Closed
    );
    server.await.unwrap();
}

#[tokio::test]
async fn transmitted_write_timeout_has_structured_unknown_reason() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        assert_eq!(command(&request), 0x1401);
        tokio::time::sleep(Duration::from_millis(200)).await;
    });

    let client = SlmpClient::connect(options(port, Duration::from_millis(40)))
        .await
        .unwrap();
    let error = client.write_words(device(10), &[1]).await.unwrap_err();
    assert_eq!(error.kind, SlmpErrorKind::OutcomeUnknown);
    assert_eq!(
        error.outcome_unknown_reason,
        Some(SlmpOutcomeUnknownReason::Timeout)
    );
    assert_eq!(
        client.read_words(device(10), 1).await.unwrap_err().kind,
        SlmpErrorKind::Closed
    );
    server.await.unwrap();
}
