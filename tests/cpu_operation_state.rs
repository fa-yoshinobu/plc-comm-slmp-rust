use plc_comm_slmp::{
    SlmpClient, SlmpCompatibilityMode, SlmpConnectionOptions, SlmpCpuOperationStatus,
    SlmpFrameType,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn read_cpu_operation_state_masks_upper_bits_of_sd203() {
    let server = SingleWordServer::start(0x00A2).await.unwrap();

    let mut options = SlmpConnectionOptions::new("127.0.0.1");
    options.port = server.port;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;
    let client = SlmpClient::connect(options).await.unwrap();

    let state = client.read_cpu_operation_state().await.unwrap();

    assert_eq!(server.request_count().await, 1);
    assert_eq!(state.status, SlmpCpuOperationStatus::Stop);
    assert_eq!(state.raw_status_word, 0x00A2);
    assert_eq!(state.raw_code, 0x02);
}

#[tokio::test]
async fn read_cpu_operation_state_returns_unknown_for_unhandled_code() {
    let server = SingleWordServer::start(0x00F5).await.unwrap();

    let mut options = SlmpConnectionOptions::new("127.0.0.1");
    options.port = server.port;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;
    let client = SlmpClient::connect(options).await.unwrap();

    let state = client.read_cpu_operation_state().await.unwrap();

    assert_eq!(state.status, SlmpCpuOperationStatus::Unknown);
    assert_eq!(state.raw_status_word, 0x00F5);
    assert_eq!(state.raw_code, 0x05);
}

struct SingleWordServer {
    port: u16,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl SingleWordServer {
    async fn start(word: u16) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_sink = requests.clone();
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
                request_sink.lock().await.push(request.clone());

                let response = build_4e_response(&request, &word.to_le_bytes());
                let _ = stream.write_all(&response).await;
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
