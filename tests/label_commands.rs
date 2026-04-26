use plc_comm_slmp::{
    SlmpClient, SlmpCompatibilityMode, SlmpConnectionOptions, SlmpFrameType,
    SlmpLabelArrayWritePoint, SlmpLabelRandomWritePoint, SlmpPlcFamily,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn read_random_labels_builds_utf16_payload_and_parses_response() {
    let server = SingleShotServer::start(vec![0x01, 0x00, 0x09, 0x00, 0x02, 0x00, 0x31, 0x00])
        .await
        .unwrap();
    let client = connect(server.port).await;

    let labels = vec!["LabelW".to_string()];
    let values = client.read_random_labels(&labels, &[]).await.unwrap();

    assert_eq!(values[0].read_data_length, 2);
    assert_eq!(values[0].data, vec![0x31, 0x00]);
    let request = server.take_request().await.unwrap();
    assert_eq!(&request[15..17], &[0x1c, 0x04]);
    assert_eq!(
        hex_upper(&request[19..]),
        "0100000006004C006100620065006C005700"
    );
}

#[tokio::test]
async fn label_writes_build_expected_payloads() {
    let server = TwoShotServer::start(vec![Vec::new(), Vec::new()])
        .await
        .unwrap();
    let client = connect(server.port).await;

    client
        .write_random_labels(
            &[SlmpLabelRandomWritePoint {
                label: "LabelW".into(),
                data: vec![0x31, 0x00],
            }],
            &[],
        )
        .await
        .unwrap();
    client
        .write_array_labels(
            &[SlmpLabelArrayWritePoint {
                label: "LabelW".into(),
                unit_specification: 1,
                array_data_length: 2,
                data: vec![0xaa, 0xbb],
            }],
            &[],
        )
        .await
        .unwrap();

    let requests = server.take_requests().await;
    assert_eq!(&requests[0][15..17], &[0x1b, 0x14]);
    assert_eq!(
        hex_upper(&requests[0][19..]),
        "0100000006004C006100620065006C00570002003100"
    );
    assert_eq!(&requests[1][15..17], &[0x1a, 0x14]);
    assert_eq!(
        hex_upper(&requests[1][19..]),
        "0100000006004C006100620065006C00570001000200AABB"
    );
}

async fn connect(port: u16) -> SlmpClient {
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqR);
    options.port = port;
    options.frame_type = SlmpFrameType::Frame4E;
    options.compatibility_mode = SlmpCompatibilityMode::Iqr;
    SlmpClient::connect(options).await.unwrap()
}

fn hex_upper(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

struct SingleShotServer {
    port: u16,
    request: std::sync::Arc<tokio::sync::Mutex<Option<Vec<u8>>>>,
}

impl SingleShotServer {
    async fn start(response_data: Vec<u8>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let request = std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let request_clone = request.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                if let Some(request) = read_request(&mut stream).await {
                    *request_clone.lock().await = Some(request.clone());
                    let _ = stream
                        .write_all(&build_4e_response(&request, &response_data))
                        .await;
                }
            }
        });
        Ok(Self { port, request })
    }

    async fn take_request(&self) -> Option<Vec<u8>> {
        self.request.lock().await.take()
    }
}

struct TwoShotServer {
    port: u16,
    requests: std::sync::Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl TwoShotServer {
    async fn start(responses: Vec<Vec<u8>>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let requests_clone = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                for response_data in responses {
                    let Some(request) = read_request(&mut stream).await else {
                        return;
                    };
                    requests_clone.lock().await.push(request.clone());
                    let _ = stream
                        .write_all(&build_4e_response(&request, &response_data))
                        .await;
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
