use plc_comm_slmp::{
    RawSlmpDeviceAddress, SlmpCompatibilityMode, SlmpDeviceCode, SlmpPlcProfile,
    encode_raw_device_spec, normalize_named_address,
};

#[test]
fn raw_device_spec_encoding_matches_repository_owned_vectors() {
    let cases = [
        (
            SlmpCompatibilityMode::Legacy,
            SlmpDeviceCode::D,
            100,
            vec![0x64, 0x00, 0x00, 0xA8],
        ),
        (
            SlmpCompatibilityMode::Legacy,
            SlmpDeviceCode::RD,
            524_287,
            vec![0xFF, 0xFF, 0x07, 0x2C],
        ),
        (
            SlmpCompatibilityMode::Iqr,
            SlmpDeviceCode::D,
            100,
            vec![0x64, 0x00, 0x00, 0x00, 0xA8, 0x00],
        ),
        (
            SlmpCompatibilityMode::Iqr,
            SlmpDeviceCode::RD,
            524_287,
            vec![0xFF, 0xFF, 0x07, 0x00, 0x2C, 0x00],
        ),
    ];

    for (mode, code, number, expected) in cases {
        assert_eq!(
            encode_raw_device_spec(mode, RawSlmpDeviceAddress::new(code, number)),
            expected
        );
    }
}

#[test]
fn named_address_normalization_pins_dtype_and_bit_index_forms() {
    assert_eq!(
        normalize_named_address("d001:u", SlmpPlcProfile::IqR).unwrap(),
        "D1:U"
    );
    assert_eq!(
        normalize_named_address("D50.a", SlmpPlcProfile::IqR).unwrap(),
        "D50.A"
    );
    assert_eq!(
        normalize_named_address("x01a:bit", SlmpPlcProfile::IqR).unwrap(),
        "X1A:BIT"
    );
}
