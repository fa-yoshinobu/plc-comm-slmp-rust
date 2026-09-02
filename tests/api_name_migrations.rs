use futures_util::StreamExt;
use plc_comm_slmp::{
    NamedAddressParts, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress,
    SlmpDeviceCode, SlmpPlcProfile, SlmpQualifiedDeviceAddress, SlmpRandomReadResult,
    SlmpTargetAddress, SlmpTransportMode, normalize_named_address, parse_named_address,
    parse_qualified_device, poll, poll_named,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[test]
fn public_api_source_contains_canonical_names_and_no_removed_callable() {
    let client = include_str!("../src/client.rs");
    let helpers = include_str!("../src/helpers.rs");
    let model = include_str!("../src/model.rs");
    let lib = include_str!("../src/lib.rs");

    for name in [
        "read_words",
        "read_dwords",
        "read_random_extended",
        "register_monitor_devices_extended",
        "write_random_words_extended",
        "write_random_bits_extended",
    ] {
        assert!(
            client.contains(&format!("pub async fn {name}")),
            "missing {name}"
        );
    }
    for name in [
        "memory_read_words",
        "memory_write_words",
        "extend_unit_read_words",
        "extend_unit_write_words",
    ] {
        assert!(
            !client.contains(&format!("pub async fn {name}")),
            "removed callable remains public: {name}"
        );
    }
    assert!(helpers.contains("pub fn poll<'a>"));
    assert!(model.contains("pub fn parse_canonical_name"));
    assert!(lib.contains("parse_scalar_for_named, poll, poll_named"));
}

#[test]
fn device_address_and_address_spec_surfaces_are_unambiguous() {
    for text in ["D100", "X10"] {
        let parsed = SlmpAddress::parse(text, SlmpPlcProfile::IqR).unwrap();
        assert_eq!(SlmpAddress::format(parsed), text);
        assert_eq!(
            SlmpAddress::normalize(text, SlmpPlcProfile::IqR).unwrap(),
            text
        );
    }

    let word_spec: NamedAddressParts = parse_named_address("D100:U").unwrap();
    assert_eq!(word_spec.base, "D100");
    assert_eq!(word_spec.dtype, "U");
    assert_eq!(word_spec.bit_index, None);
    assert_eq!(
        normalize_named_address("D100:U", SlmpPlcProfile::IqR).unwrap(),
        "D100:U"
    );

    let bit_spec: NamedAddressParts = parse_named_address("D50.A").unwrap();
    assert_eq!(bit_spec.base, "D50");
    assert_eq!(bit_spec.dtype, "BIT_IN_WORD");
    assert_eq!(bit_spec.bit_index, Some(0xA));
    assert_eq!(
        normalize_named_address("D50.a", SlmpPlcProfile::IqR).unwrap(),
        "D50.A"
    );

    assert!(SlmpAddress::parse("D100:U", SlmpPlcProfile::IqR).is_err());
    assert!(SlmpAddress::parse("D50.A", SlmpPlcProfile::IqR).is_err());
    assert!(SlmpAddress::parse(r"J1\X10", SlmpPlcProfile::IqR).is_err());
    assert!(parse_named_address("D100").is_err());
}

#[tokio::test]
async fn decoded_direct_read_legacy_names_match_canonical_result_and_wire() {
    let (canonical_words, canonical_word_request) = read_words_exchange(false).await;
    let (legacy_words, legacy_word_request) = read_words_exchange(true).await;
    assert_eq!(canonical_words, vec![0x1234, 0xABCD]);
    assert_eq!(legacy_words, canonical_words);
    assert_eq!(legacy_word_request, canonical_word_request);

    let (canonical_dwords, canonical_dword_request) = read_dwords_exchange(false).await;
    let (legacy_dwords, legacy_dword_request) = read_dwords_exchange(true).await;
    assert_eq!(canonical_dwords, vec![0x89AB_1234]);
    assert_eq!(legacy_dwords, canonical_dwords);
    assert_eq!(legacy_dword_request, canonical_dword_request);
}

#[tokio::test]
async fn extended_legacy_names_match_canonical_result_and_wire() {
    let (canonical_read, canonical_read_request) = extended_read_exchange(false).await;
    let (legacy_read, legacy_read_request) = extended_read_exchange(true).await;
    assert_eq!(canonical_read.word_values, vec![0x1234]);
    assert_eq!(legacy_read, canonical_read);
    assert_eq!(legacy_read_request, canonical_read_request);

    for operation in [
        ExtendedWriteOperation::RegisterMonitor,
        ExtendedWriteOperation::RandomWords,
        ExtendedWriteOperation::RandomBits,
    ] {
        let canonical_request = extended_write_exchange(operation, false).await;
        let legacy_request = extended_write_exchange(operation, true).await;
        assert_eq!(legacy_request, canonical_request, "operation={operation:?}");
    }
}

#[tokio::test]
async fn poll_named_matches_canonical_first_item_and_wire() {
    let (canonical_value, canonical_request) = poll_exchange(false).await;
    let (legacy_value, legacy_request) = poll_exchange(true).await;

    assert_eq!(legacy_value, canonical_value);
    assert_eq!(legacy_request, canonical_request);
}

async fn read_words_exchange(legacy: bool) -> (Vec<u16>, Vec<u8>) {
    let server = SingleExchangeServer::start(vec![0x34, 0x12, 0xCD, 0xAB])
        .await
        .unwrap();
    let client = connect(server.port).await;
    let device = SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR);
    let value = if legacy {
        client.read_words_raw(device, 2).await.unwrap()
    } else {
        client.read_words(device, 2).await.unwrap()
    };
    (value, server.first_request().await)
}

async fn read_dwords_exchange(legacy: bool) -> (Vec<u32>, Vec<u8>) {
    let server = SingleExchangeServer::start(vec![0x34, 0x12, 0xAB, 0x89])
        .await
        .unwrap();
    let client = connect(server.port).await;
    let device = SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR);
    let value = if legacy {
        client.read_dwords_raw(device, 1).await.unwrap()
    } else {
        client.read_dwords(device, 1).await.unwrap()
    };
    (value, server.first_request().await)
}

async fn extended_read_exchange(legacy: bool) -> (SlmpRandomReadResult, Vec<u8>) {
    let server = SingleExchangeServer::start(vec![0x34, 0x12]).await.unwrap();
    let client = connect(server.port).await;
    let device = qualified_word();
    let value = if legacy {
        client.read_random_ext(&[device], &[]).await.unwrap()
    } else {
        client.read_random_extended(&[device], &[]).await.unwrap()
    };
    (value, server.first_request().await)
}

#[derive(Clone, Copy, Debug)]
enum ExtendedWriteOperation {
    RegisterMonitor,
    RandomWords,
    RandomBits,
}

async fn extended_write_exchange(operation: ExtendedWriteOperation, legacy: bool) -> Vec<u8> {
    let server = SingleExchangeServer::start(vec![]).await.unwrap();
    let client = connect(server.port).await;
    match operation {
        ExtendedWriteOperation::RegisterMonitor => {
            let device = qualified_word();
            if legacy {
                client
                    .register_monitor_devices_ext(&[device], &[])
                    .await
                    .unwrap();
            } else {
                client
                    .register_monitor_devices_extended(&[device], &[])
                    .await
                    .unwrap();
            }
        }
        ExtendedWriteOperation::RandomWords => {
            let device = qualified_word();
            if legacy {
                client
                    .write_random_words_ext(&[(device, 0x1234)], &[])
                    .await
                    .unwrap();
            } else {
                client
                    .write_random_words_extended(&[(device, 0x1234)], &[])
                    .await
                    .unwrap();
            }
        }
        ExtendedWriteOperation::RandomBits => {
            let device = parse_qualified_device(r"J1\B0", SlmpPlcProfile::IqR).unwrap();
            if legacy {
                client
                    .write_random_bits_ext(&[(device, true)])
                    .await
                    .unwrap();
            } else {
                client
                    .write_random_bits_extended(&[(device, true)])
                    .await
                    .unwrap();
            }
        }
    }
    server.first_request().await
}

async fn poll_exchange(legacy: bool) -> (plc_comm_slmp::NamedAddress, Vec<u8>) {
    let server = SingleExchangeServer::start(vec![0x34, 0x12]).await.unwrap();
    let client = connect(server.port).await;
    let addresses = vec!["D100:U".to_owned()];
    let value = if legacy {
        let mut stream = Box::pin(poll_named(&client, &addresses, Duration::ZERO));
        stream.next().await.unwrap().unwrap()
    } else {
        let mut stream = Box::pin(poll(&client, &addresses, Duration::ZERO));
        stream.next().await.unwrap().unwrap()
    };
    (value, server.first_request().await)
}

fn qualified_word() -> SlmpQualifiedDeviceAddress {
    parse_qualified_device(r"J1\W0", SlmpPlcProfile::IqR).unwrap()
}

async fn connect(port: u16) -> SlmpClient {
    let options = SlmpConnectionOptions::new(
        "127.0.0.1",
        port,
        SlmpTransportMode::Tcp,
        SlmpTargetAddress::default(),
        SlmpPlcProfile::IqR,
    )
    .unwrap();
    SlmpClient::connect(options).await.unwrap()
}

struct SingleExchangeServer {
    port: u16,
    requests: Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl SingleExchangeServer {
    async fn start(response_data: Vec<u8>) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let requests = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let request_sink = requests.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut header = [0u8; 13];
                if stream.read_exact(&mut header).await.is_err() {
                    return;
                }
                let body_length = u16::from_le_bytes([header[11], header[12]]) as usize;
                let mut body = vec![0u8; body_length];
                if stream.read_exact(&mut body).await.is_err() {
                    return;
                }
                let mut request = header.to_vec();
                request.extend_from_slice(&body);
                request_sink.lock().await.push(request.clone());
                let response = build_4e_response(&request, &response_data);
                let _ = stream.write_all(&response).await;
            }
        });
        Ok(Self { port, requests })
    }

    async fn first_request(&self) -> Vec<u8> {
        self.requests
            .lock()
            .await
            .first()
            .cloned()
            .expect("one request")
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
