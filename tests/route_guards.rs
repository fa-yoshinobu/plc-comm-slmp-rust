use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpExtensionSpec,
    SlmpPlcFamily, SlmpQualifiedDeviceAddress, SlmpTransportMode,
};

async fn udp_client() -> SlmpClient {
    let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqR);
    options.transport_mode = SlmpTransportMode::Udp;
    options.port = 9;
    SlmpClient::connect(options).await.unwrap()
}

#[tokio::test]
async fn direct_bit_read_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .read_bits(SlmpDeviceAddress::new(SlmpDeviceCode::LTS, 10), 1)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Direct bit read is not supported"));
}

#[tokio::test]
async fn direct_bit_write_rejects_long_counter_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits(SlmpDeviceAddress::new(SlmpDeviceCode::LCC, 10), &[true])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct bit write is not supported"));
}

#[tokio::test]
async fn direct_bit_write_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits(SlmpDeviceAddress::new(SlmpDeviceCode::LTC, 10), &[true])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct bit write is not supported"));
}

#[tokio::test]
async fn direct_word_write_rejects_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10), 4)
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct word read is not supported"));

    let err = client
        .write_words(SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10), &[1])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct word write is not supported"));

    let err = client
        .write_words(SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1), &[1])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct word write is not supported"));
}

#[tokio::test]
async fn direct_dword_routes_reject_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .read_dwords_raw(SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10), 1)
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct dword read is not supported"));

    let err = client
        .write_dwords(SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1), &[1])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct dword write is not supported"));
}

#[tokio::test]
async fn direct_extended_bit_write_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LSTS, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[true],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct bit write is not supported"));
}

#[tokio::test]
async fn direct_extended_bit_write_rejects_long_counter_state_devices() {
    let client = udp_client().await;
    let err = client
        .write_bits_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LCS, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[true],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct bit write is not supported"));
}

#[tokio::test]
async fn direct_extended_word_read_rejects_long_counter_current_devices() {
    let client = udp_client().await;
    let err = client
        .read_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            4,
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct word read is not supported"));
}

#[tokio::test]
async fn direct_extended_word_write_rejects_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .write_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LTN, 10),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[1],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct word write is not supported"));

    let err = client
        .write_words_extended(
            SlmpQualifiedDeviceAddress {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1),
                extension_specification: None,
                direct_memory_specification: None,
            },
            &[1],
            SlmpExtensionSpec::default(),
        )
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Direct word write is not supported"));
}

#[tokio::test]
async fn random_read_rejects_long_timer_state_devices() {
    let client = udp_client().await;
    let err = client
        .read_random(&[SlmpDeviceAddress::new(SlmpDeviceCode::LTC, 10)], &[])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("Read Random (0x0403) does not support LTS/LTC/LSTS/LSTC"));
}

#[tokio::test]
async fn random_word_routes_reject_long_current_and_lz_devices() {
    let client = udp_client().await;
    let err = client
        .read_random(&[SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10)], &[])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("does not support LTN/LSTN/LCN/LZ as word entries"));

    let err = client
        .write_random_words(&[(SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 1), 1)], &[])
        .await
        .unwrap_err();
    assert!(err
        .to_string()
        .contains("does not support LTN/LSTN/LCN/LZ as word entries"));
}

#[tokio::test]
async fn block_routes_reject_lcn_lz_and_long_current_write_blocks() {
    let client = udp_client().await;
    let err = client
        .read_block(
            &[plc_comm_slmp::SlmpBlockRead {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LCN, 10),
                points: 4,
            }],
            &[],
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("does not support LCN/LZ"));

    let err = client
        .write_block(
            &[plc_comm_slmp::SlmpBlockWrite {
                device: SlmpDeviceAddress::new(SlmpDeviceCode::LZ, 0),
                values: vec![1, 0],
            }],
            &[],
            None,
        )
        .await
        .unwrap_err();
    assert!(err.to_string().contains("does not support LTN/LSTN/LCN/LZ"));
}
