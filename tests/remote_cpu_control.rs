use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpRemoteClearMode, SlmpRemoteMode,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn cpu_operations_keep_remote_stop_on_manual_fixed_mode() {
    let server = MultiShotServer::start(3).await.unwrap();
    let client = connect(server.port).await;

    client
        .remote_run(SlmpRemoteMode::Force, SlmpRemoteClearMode::NoClear)
        .await
        .unwrap();
    client.remote_stop().await.unwrap();
    client.remote_pause(SlmpRemoteMode::Force).await.unwrap();

    let requests = server.take_requests().await;
    assert_eq!(requests.len(), 3);

    assert_eq!(&requests[0][15..17], &[0x01, 0x10]);
    assert_eq!(&requests[0][19..], &[0x03, 0x00, 0x00, 0x00]);

    assert_eq!(&requests[1][15..17], &[0x02, 0x10]);
    assert_eq!(&requests[1][19..], &[0x01, 0x00]);

    assert_eq!(&requests[2][15..17], &[0x03, 0x10]);
    assert_eq!(&requests[2][19..], &[0x03, 0x00]);
}

#[tokio::test]
async fn remote_run_clear_modes_have_distinct_wire_values() {
    let server = MultiShotServer::start(3).await.unwrap();
    let client = connect(server.port).await;

    client
        .remote_run(SlmpRemoteMode::Normal, SlmpRemoteClearMode::NoClear)
        .await
        .unwrap();
    client
        .remote_run(
            SlmpRemoteMode::Normal,
            SlmpRemoteClearMode::ClearExceptLatch,
        )
        .await
        .unwrap();
    client
        .remote_run(SlmpRemoteMode::Normal, SlmpRemoteClearMode::ClearAll)
        .await
        .unwrap();

    let requests = server.take_requests().await;
    assert_eq!(&requests[0][19..], &[0x01, 0x00, 0x00, 0x00]);
    assert_eq!(&requests[1][19..], &[0x01, 0x00, 0x01, 0x00]);
    assert_eq!(&requests[2][19..], &[0x01, 0x00, 0x02, 0x00]);
}

#[tokio::test]
async fn remote_reset_sends_fixed_reset_data() {
    let server = MultiShotServer::start(1).await.unwrap();
    let client = connect(server.port).await;

    client.remote_reset().await.unwrap();

    let request = client.last_request_frame().await;
    assert_eq!(&request[15..17], &[0x06, 0x10]);
    assert_eq!(&request[17..19], &[0x00, 0x00]);
    assert_eq!(&request[19..], &[0x01, 0x00]);
    let error = client.read_type_name().await.unwrap_err();
    assert!(error.message.contains("transport is closed"));
}

async fn connect(port: u16) -> SlmpClient {
    let mut options = SlmpConnectionOptions::new(
        "127.0.0.1",
        1025,
        plc_comm_slmp::SlmpTransportMode::Tcp,
        plc_comm_slmp::SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    options.port = port;
    SlmpClient::connect(options).await.unwrap()
}

struct MultiShotServer {
    port: u16,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl MultiShotServer {
    async fn start(response_count: usize) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let requests_clone = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                for _ in 0..response_count {
                    let Some(request) = read_request(&mut stream).await else {
                        return;
                    };
                    requests_clone.lock().await.push(request.clone());
                    let _ = stream.write_all(&build_4e_response(&request)).await;
                }
            }
        });
        Ok(Self { port, requests })
    }

    async fn take_requests(&self) -> Vec<Vec<u8>> {
        self.requests.lock().await.clone()
    }
}

async fn read_request(stream: &mut tokio::net::TcpStream) -> Option<Vec<u8>> {
    let mut header = [0u8; 19];
    stream.read_exact(&mut header[0..2]).await.ok()?;
    stream.read_exact(&mut header[2..19]).await.ok()?;
    let payload_len = u16::from_le_bytes([header[11], header[12]]) as usize - 6;
    let mut payload = vec![0u8; payload_len];
    stream.read_exact(&mut payload).await.ok()?;
    let mut request = header.to_vec();
    request.extend_from_slice(&payload);
    Some(request)
}

fn build_4e_response(request: &[u8]) -> Vec<u8> {
    let payload = [0u8; 2];
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
