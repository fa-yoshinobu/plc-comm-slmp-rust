use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpTargetAddress, SlmpTransportMode,
};
use std::net::IpAddr;
use tokio::net::TcpListener;

#[test]
fn connection_options_reject_ipv6_literals() {
    for host in ["::1", "[::1]", "::ffff:127.0.0.1"] {
        let error = SlmpConnectionOptions::new(
            host,
            1025,
            SlmpTransportMode::Tcp,
            SlmpTargetAddress::default(),
            SlmpPlcProfile::IqR,
        )
        .unwrap_err();
        assert!(error.to_string().contains("IPv6 is unsupported"));
    }
}

#[tokio::test]
async fn hostname_tcp_connection_uses_ipv4() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accept = tokio::spawn(async move {
        let (_stream, peer) = listener.accept().await.unwrap();
        peer.ip()
    });
    let options = SlmpConnectionOptions::new(
        "localhost",
        port,
        SlmpTransportMode::Tcp,
        SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();

    let client = SlmpClient::connect(options).await.unwrap();
    assert!(matches!(accept.await.unwrap(), IpAddr::V4(_)));
    drop(client);
}

#[tokio::test]
async fn hostname_udp_connection_uses_ipv4_resolution() {
    let options = SlmpConnectionOptions::new(
        "localhost",
        9,
        SlmpTransportMode::Udp,
        SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();

    let client = SlmpClient::connect(options).await.unwrap();
    drop(client);
}
