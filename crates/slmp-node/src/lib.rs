use napi_derive::napi;
use plc_comm_slmp::{SlmpAddress, SlmpPlcProfile};

#[napi]
pub fn normalize_address(address: String, plc_profile: String) -> napi::Result<String> {
    let plc_profile = SlmpPlcProfile::parse_label(&plc_profile).ok_or_else(|| {
        napi::Error::from_reason(
            "plc_profile must be an exact canonical profile label such as 'melsec:iq-r'",
        )
    })?;
    plc_profile
        .validate_connection_selectable()
        .map_err(|error| napi::Error::from_reason(error.message))?;
    SlmpAddress::normalize(&address, plc_profile)
        .map_err(|error| napi::Error::from_reason(error.message))
}

#[cfg(test)]
mod tests {
    use super::normalize_address;

    #[test]
    fn normalization_requires_an_exact_canonical_profile() {
        assert_eq!(
            normalize_address("d100".to_owned(), "melsec:iq-r".to_owned()).unwrap(),
            "D100"
        );
        assert!(normalize_address("D100".to_owned(), "iqr".to_owned()).is_err());
        assert!(normalize_address("D100".to_owned(), "melsec:qcpu".to_owned()).is_err());
    }

    #[test]
    fn normalization_applies_profile_device_rules() {
        assert!(normalize_address("DX10".to_owned(), "melsec:iq-f".to_owned()).is_err());
        assert_eq!(
            normalize_address("DX10".to_owned(), "melsec:iq-r".to_owned()).unwrap(),
            "DX10"
        );
    }
}
