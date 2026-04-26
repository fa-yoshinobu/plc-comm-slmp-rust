use crate::client::SlmpClient;
use crate::device_ranges::{
    SlmpDeviceRangeCatalog, SlmpDeviceRangeFamily, build_catalog_for_family, read_registers,
    resolve_family, resolve_profile, resolve_profile_for_family,
};
use crate::model::{
    SlmpCompatibilityMode, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode, SlmpFrameType,
    SlmpPlcFamily, SlmpTransportMode, SlmpTypeNameInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SlmpConnectionProfileProbeStatus {
    Validated,
    TypeNameOnly,
    UnsupportedPlc,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpConnectionProfileProbeResult {
    pub transport_mode: SlmpTransportMode,
    pub frame_type: SlmpFrameType,
    pub compatibility_mode: SlmpCompatibilityMode,
    pub status: SlmpConnectionProfileProbeStatus,
    pub type_name_info: Option<SlmpTypeNameInfo>,
    pub family: Option<SlmpDeviceRangeFamily>,
    pub sd_register_start: Option<u16>,
    pub sd_register_count: Option<u16>,
    pub sd_read_succeeded: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SlmpResolvedDeviceRangeCatalog {
    pub transport_mode: SlmpTransportMode,
    pub frame_type: SlmpFrameType,
    pub compatibility_mode: SlmpCompatibilityMode,
    pub used_three_e_legacy_fallback: bool,
    pub catalog: SlmpDeviceRangeCatalog,
}

pub async fn probe_connection_profiles(
    options: &SlmpConnectionOptions,
) -> Vec<SlmpConnectionProfileProbeResult> {
    let candidates = [
        (SlmpFrameType::Frame4E, SlmpCompatibilityMode::Iqr),
        (SlmpFrameType::Frame3E, SlmpCompatibilityMode::Iqr),
        (SlmpFrameType::Frame4E, SlmpCompatibilityMode::Legacy),
        (SlmpFrameType::Frame3E, SlmpCompatibilityMode::Legacy),
    ];

    let mut results = Vec::with_capacity(candidates.len());
    for (frame_type, compatibility_mode) in candidates {
        results.push(probe_connection_profile(options, frame_type, compatibility_mode).await);
    }
    results
}

pub(crate) async fn probe_connection_profile(
    options: &SlmpConnectionOptions,
    frame_type: SlmpFrameType,
    compatibility_mode: SlmpCompatibilityMode,
) -> SlmpConnectionProfileProbeResult {
    let mut type_name_info = None;
    let mut family = None;
    let mut sd_register_start = None;
    let mut sd_register_count = None;
    let mut sd_read_succeeded = false;
    let mut error_message = None;

    let mut attempt = options.clone();
    attempt.frame_type = frame_type;
    attempt.compatibility_mode = compatibility_mode;

    match SlmpClient::connect(attempt.clone()).await {
        Ok(client) => {
            match client.read_type_name().await {
                Ok(info) => {
                    family = resolve_family(&info).ok();
                    type_name_info = Some(info);
                }
                Err(error) => {
                    error_message = Some(format!("read_type_name: {error}"));
                    return SlmpConnectionProfileProbeResult {
                        transport_mode: attempt.transport_mode,
                        frame_type,
                        compatibility_mode,
                        status: SlmpConnectionProfileProbeStatus::UnsupportedPlc,
                        type_name_info: None,
                        family: None,
                        sd_register_start: None,
                        sd_register_count: None,
                        sd_read_succeeded: false,
                        error_message,
                    };
                }
            }

            match resolve_profile(type_name_info.as_ref().expect("type name set above")) {
                Ok(profile) => {
                    family = Some(profile.family);
                    sd_register_start = Some(profile.register_start);
                    sd_register_count = Some(profile.register_count);
                    if profile.register_count == 0 {
                        sd_read_succeeded = true;
                    } else {
                        match client
                            .read_words_raw(
                                SlmpDeviceAddress::new(
                                    SlmpDeviceCode::SD,
                                    u32::from(profile.register_start),
                                ),
                                profile.register_count,
                            )
                            .await
                        {
                            Ok(_) => {
                                sd_read_succeeded = true;
                            }
                            Err(error) => {
                                error_message = Some(format!("read_sd_block: {error}"));
                            }
                        }
                    }
                }
                Err(error) => {
                    error_message = Some(format!("resolve_family: {error}"));
                }
            }
        }
        Err(error) => {
            error_message = Some(format!("open: {error}"));
        }
    }

    let status = if type_name_info.is_none() {
        SlmpConnectionProfileProbeStatus::Failed
    } else if sd_read_succeeded {
        SlmpConnectionProfileProbeStatus::Validated
    } else {
        SlmpConnectionProfileProbeStatus::TypeNameOnly
    };

    SlmpConnectionProfileProbeResult {
        transport_mode: attempt.transport_mode,
        frame_type,
        compatibility_mode,
        status,
        type_name_info,
        family,
        sd_register_start,
        sd_register_count,
        sd_read_succeeded,
        error_message,
    }
}

pub async fn read_device_range_catalog_with_three_e_legacy_fallback(
    options: &SlmpConnectionOptions,
) -> Result<SlmpResolvedDeviceRangeCatalog, crate::error::SlmpError> {
    let initial =
        try_read_device_range_catalog(options, options.frame_type, options.compatibility_mode)
            .await?;
    if let Some(result) = initial.result {
        return Ok(result);
    }

    if options.frame_type == SlmpFrameType::Frame3E
        && options.compatibility_mode == SlmpCompatibilityMode::Legacy
    {
        return Err(initial
            .error
            .unwrap_or_else(|| crate::error::SlmpError::new("device-range catalog read failed")));
    }

    let fallback = try_read_device_range_catalog(
        options,
        SlmpFrameType::Frame3E,
        SlmpCompatibilityMode::Legacy,
    )
    .await?;
    if let Some(mut result) = fallback.result {
        result.used_three_e_legacy_fallback = true;
        return Ok(result);
    }

    Err(fallback
        .error
        .or(initial.error)
        .unwrap_or_else(|| crate::error::SlmpError::new("device-range catalog read failed")))
}

struct TryReadDeviceRangeCatalogOutcome {
    result: Option<SlmpResolvedDeviceRangeCatalog>,
    error: Option<crate::error::SlmpError>,
}

async fn try_read_device_range_catalog(
    options: &SlmpConnectionOptions,
    frame_type: SlmpFrameType,
    compatibility_mode: SlmpCompatibilityMode,
) -> Result<TryReadDeviceRangeCatalogOutcome, crate::error::SlmpError> {
    let mut attempt = options.clone();
    attempt.frame_type = frame_type;
    attempt.compatibility_mode = compatibility_mode;

    let client = match SlmpClient::connect(attempt.clone()).await {
        Ok(client) => client,
        Err(error) => {
            return Ok(TryReadDeviceRangeCatalogOutcome {
                result: None,
                error: Some(error),
            });
        }
    };
    let family = map_plc_family_to_range_family(attempt.plc_family);
    let profile = resolve_profile_for_family(family);
    let registers = match read_registers(&client, &profile).await {
        Ok(registers) => registers,
        Err(error) => {
            return Ok(TryReadDeviceRangeCatalogOutcome {
                result: None,
                error: Some(error),
            });
        }
    };
    let catalog = match build_catalog_for_family(family, &registers) {
        Ok(catalog) => catalog,
        Err(error) => {
            return Ok(TryReadDeviceRangeCatalogOutcome {
                result: None,
                error: Some(error),
            });
        }
    };
    Ok(TryReadDeviceRangeCatalogOutcome {
        result: Some(SlmpResolvedDeviceRangeCatalog {
            transport_mode: attempt.transport_mode,
            frame_type,
            compatibility_mode,
            used_three_e_legacy_fallback: false,
            catalog,
        }),
        error: None,
    })
}

fn map_plc_family_to_range_family(family: SlmpPlcFamily) -> SlmpDeviceRangeFamily {
    match family {
        SlmpPlcFamily::IqF => SlmpDeviceRangeFamily::IqF,
        SlmpPlcFamily::IqR | SlmpPlcFamily::IqL => SlmpDeviceRangeFamily::IqR,
        SlmpPlcFamily::MxF => SlmpDeviceRangeFamily::MxF,
        SlmpPlcFamily::MxR => SlmpDeviceRangeFamily::MxR,
        SlmpPlcFamily::QCpu => SlmpDeviceRangeFamily::QCpu,
        SlmpPlcFamily::LCpu => SlmpDeviceRangeFamily::LCpu,
        SlmpPlcFamily::QnU => SlmpDeviceRangeFamily::QnU,
        SlmpPlcFamily::QnUDV => SlmpDeviceRangeFamily::QnUDV,
    }
}

#[cfg(test)]
mod tests {
    use super::{SlmpConnectionProfileProbeStatus, probe_connection_profile};
    use crate::{
        SlmpCompatibilityMode, SlmpConnectionOptions, SlmpDeviceRangeFamily, SlmpFrameType,
        SlmpPlcFamily,
    };
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn probe_connection_profile_returns_validated_when_type_name_and_sd_read_succeed() {
        let server = ProbeServer::start(vec![
            ProbeResponse::new(build_type_name_payload("R120PCPU", 0x4844)),
            ProbeResponse::new(build_word_payload(&vec![0u16; 50])),
        ])
        .await
        .unwrap();

        let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqR);
        options.port = server.port;
        options.frame_type = SlmpFrameType::Frame4E;
        options.compatibility_mode = SlmpCompatibilityMode::Iqr;

        let result =
            probe_connection_profile(&options, SlmpFrameType::Frame4E, SlmpCompatibilityMode::Iqr)
                .await;

        assert_eq!(result.status, SlmpConnectionProfileProbeStatus::Validated);
        assert!(result.sd_read_succeeded);
        assert_eq!(result.type_name_info.as_ref().unwrap().model, "R120PCPU");
        assert_eq!(result.type_name_info.as_ref().unwrap().model_code, 0x4844);
        assert_eq!(result.family, Some(SlmpDeviceRangeFamily::IqR));
        assert_eq!(result.sd_register_start, Some(260));
        assert_eq!(result.sd_register_count, Some(50));
        assert_eq!(result.error_message, None);
    }

    #[tokio::test]
    async fn probe_connection_profile_returns_type_name_only_when_sd_read_fails() {
        let server = ProbeServer::start(vec![
            ProbeResponse::new(build_type_name_payload("FX5UC-32MT/D", 0x4A91)),
            ProbeResponse::with_end_code(Vec::new(), 0xC059),
        ])
        .await
        .unwrap();

        let mut options = SlmpConnectionOptions::new("127.0.0.1", SlmpPlcFamily::IqF);
        options.port = server.port;
        options.frame_type = SlmpFrameType::Frame4E;
        options.compatibility_mode = SlmpCompatibilityMode::Iqr;

        let result =
            probe_connection_profile(&options, SlmpFrameType::Frame4E, SlmpCompatibilityMode::Iqr)
                .await;

        assert_eq!(
            result.status,
            SlmpConnectionProfileProbeStatus::TypeNameOnly
        );
        assert!(!result.sd_read_succeeded);
        assert_eq!(
            result.type_name_info.as_ref().unwrap().model,
            "FX5UC-32MT/D"
        );
        assert_eq!(result.type_name_info.as_ref().unwrap().model_code, 0x4A91);
        assert_eq!(result.family, Some(SlmpDeviceRangeFamily::IqF));
        assert_eq!(result.sd_register_start, Some(260));
        assert_eq!(result.sd_register_count, Some(46));
        assert!(
            result
                .error_message
                .as_deref()
                .unwrap()
                .contains("read_sd_block:")
        );
    }

    struct ProbeResponse {
        payload: Vec<u8>,
        end_code: u16,
    }

    impl ProbeResponse {
        fn new(payload: Vec<u8>) -> Self {
            Self {
                payload,
                end_code: 0,
            }
        }

        fn with_end_code(payload: Vec<u8>, end_code: u16) -> Self {
            Self { payload, end_code }
        }
    }

    struct ProbeServer {
        port: u16,
    }

    impl ProbeServer {
        async fn start(response_payloads: Vec<ProbeResponse>) -> std::io::Result<Self> {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let port = listener.local_addr()?.port();
            tokio::spawn(async move {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut pending = std::collections::VecDeque::from(response_payloads);
                    while let Some(response_info) = pending.pop_front() {
                        let mut header = [0u8; 19];
                        if stream.read_exact(&mut header).await.is_err() {
                            return;
                        }
                        let body_len = u16::from_le_bytes([header[11], header[12]]) as usize - 6;
                        let mut body = vec![0u8; body_len];
                        if stream.read_exact(&mut body).await.is_err() {
                            return;
                        }

                        let mut request = header.to_vec();
                        request.extend_from_slice(&body);
                        let response = build_4e_response(&request, response_info);
                        if stream.write_all(&response).await.is_err() {
                            return;
                        }
                    }
                }
            });
            Ok(Self { port })
        }
    }

    fn build_type_name_payload(model: &str, model_code: u16) -> Vec<u8> {
        let mut payload = vec![0u8; 18];
        payload[..model.len()].copy_from_slice(model.as_bytes());
        payload[16..18].copy_from_slice(&model_code.to_le_bytes());
        payload
    }

    fn build_word_payload(values: &[u16]) -> Vec<u8> {
        let mut payload = Vec::with_capacity(values.len() * 2);
        for value in values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        payload
    }

    fn build_4e_response(request: &[u8], response_info: ProbeResponse) -> Vec<u8> {
        let mut payload = vec![0u8; 2 + response_info.payload.len()];
        payload[0..2].copy_from_slice(&response_info.end_code.to_le_bytes());
        payload[2..].copy_from_slice(&response_info.payload);

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
}
