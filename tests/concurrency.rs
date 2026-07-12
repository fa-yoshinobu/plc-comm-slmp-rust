use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpPlcProfile,
    SlmpTargetAddress, SlmpTransportMode,
};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn concurrent_calls_are_serialized_and_use_unique_frame_serials() {
    const REQUESTS: usize = 16;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let serials = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let serials_for_server = serials.clone();
    let pipelined = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let pipelined_for_server = pipelined.clone();

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        for _ in 0..REQUESTS {
            let mut header = [0u8; 13];
            stream.read_exact(&mut header).await.unwrap();
            let body_length = u16::from_le_bytes([header[11], header[12]]) as usize;
            let mut body = vec![0u8; body_length];
            stream.read_exact(&mut body).await.unwrap();
            serials_for_server
                .lock()
                .await
                .push(u16::from_le_bytes([header[2], header[3]]));

            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            let mut probe = [0u8; 1];
            if stream.try_read(&mut probe).is_ok() {
                pipelined_for_server.store(true, std::sync::atomic::Ordering::SeqCst);
            }

            let mut response = vec![0u8; 17];
            response[0..2].copy_from_slice(&[0xD4, 0x00]);
            response[2..4].copy_from_slice(&header[2..4]);
            response[6..11].copy_from_slice(&header[6..11]);
            response[11..13].copy_from_slice(&4u16.to_le_bytes());
            response[13..15].copy_from_slice(&0u16.to_le_bytes());
            response[15..17].copy_from_slice(&0x1234u16.to_le_bytes());
            stream.write_all(&response).await.unwrap();
        }
    });

    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        port,
        SlmpTransportMode::Tcp,
        SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    let client = SlmpClient::connect(options).await.unwrap();
    let mut tasks = Vec::new();
    for number in 0..REQUESTS as u32 {
        let client = client.clone();
        tasks.push(tokio::spawn(async move {
            client
                .read_words_raw(
                    SlmpDeviceAddress::new(SlmpDeviceCode::D, number, SlmpPlcProfile::IqR),
                    1,
                )
                .await
                .unwrap()
        }));
    }
    for task in tasks {
        assert_eq!(task.await.unwrap(), vec![0x1234]);
    }
    server.await.unwrap();

    let mut observed = serials.lock().await.clone();
    observed.sort_unstable();
    assert_eq!(observed, (0..REQUESTS as u16).collect::<Vec<_>>());
    assert!(!pipelined.load(std::sync::atomic::Ordering::SeqCst));
}
