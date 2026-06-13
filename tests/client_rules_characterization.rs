use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpPlcProfile,
};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn read_bits_unpacks_high_nibble_then_low_nibble_and_trims_odd_points() {
    let server = ResponseServer::start(vec![vec![0x10, 0x01, 0x11]])
        .await
        .unwrap();
    let client = connect_client(server.port).await;

    let values = client
        .read_bits(SlmpDeviceAddress::new(SlmpDeviceCode::M, 100), 5)
        .await
        .unwrap();

    assert_eq!(values, vec![true, false, false, true, true]);
    assert_eq!(server.requests().await.len(), 1);
}

#[tokio::test]
async fn read_long_timer_decodes_four_word_blocks() {
    let server = ResponseServer::start(vec![word_payload(&[
        0x5678, 0x1234, 0x0003, 0xAAAA, 0x0001, 0x0000, 0x0002, 0xBBBB,
    ])])
    .await
    .unwrap();
    let client = connect_client(server.port).await;

    let values = client.read_long_timer(10, 2).await.unwrap();

    assert_eq!(values.len(), 2);
    assert_eq!(values[0].index, 10);
    assert_eq!(values[0].device, "LTN10");
    assert_eq!(values[0].current_value, 0x1234_5678);
    assert!(values[0].contact);
    assert!(values[0].coil);
    assert_eq!(values[0].status_word, 0x0003);
    assert_eq!(values[0].raw_words, vec![0x5678, 0x1234, 0x0003, 0xAAAA]);

    assert_eq!(values[1].index, 11);
    assert_eq!(values[1].device, "LTN11");
    assert_eq!(values[1].current_value, 0x0000_0001);
    assert!(values[1].contact);
    assert!(!values[1].coil);
    assert_eq!(values[1].status_word, 0x0002);
}

#[tokio::test]
async fn read_long_retentive_timer_uses_lstn_device_prefix() {
    let server = ResponseServer::start(vec![word_payload(&[0x0007, 0x0000, 0x0001, 0x0000])])
        .await
        .unwrap();
    let client = connect_client(server.port).await;

    let values = client.read_long_retentive_timer(20, 1).await.unwrap();

    assert_eq!(values.len(), 1);
    assert_eq!(values[0].index, 20);
    assert_eq!(values[0].device, "LSTN20");
    assert_eq!(values[0].current_value, 7);
    assert!(!values[0].contact);
    assert!(values[0].coil);
}

struct ResponseServer {
    port: u16,
    requests: Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl ResponseServer {
    async fn start(response_payloads: Vec<Vec<u8>>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_sink = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut pending = VecDeque::from(response_payloads);
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

    async fn requests(&self) -> Vec<Vec<u8>> {
        self.requests.lock().await.clone()
    }
}

async fn connect_client(port: u16) -> SlmpClient {
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcProfile::IqR);
    options.port = port;
    SlmpClient::connect(options).await.unwrap()
}

fn word_payload(values: &[u16]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(values.len() * 2);
    for value in values {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload
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
