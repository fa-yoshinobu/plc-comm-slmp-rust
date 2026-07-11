use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpPlcProfile,
    SlmpTargetAddress, SlmpTransportMode,
};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

#[tokio::test]
async fn externally_cancelled_exchange_closes_transport() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut request = [0u8; 64];
            let _ = stream.read(&mut request).await;
            std::future::pending::<()>().await;
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
    let address = SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR);

    let cancelled =
        tokio::time::timeout(Duration::from_millis(50), client.read_words_raw(address, 1)).await;
    assert!(cancelled.is_err(), "the outer timeout must cancel the call");
    assert_eq!(client.traffic_stats().await.request_count, 1);

    let error = client.read_words_raw(address, 1).await.unwrap_err();
    assert!(error.message.contains("transport is closed"));
    assert_eq!(client.traffic_stats().await.request_count, 1);
}
