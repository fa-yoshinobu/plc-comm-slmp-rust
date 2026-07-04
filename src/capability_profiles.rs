use crate::model::{SlmpDeviceCode, SlmpPlcProfile};

pub(crate) const CANONICAL_SOURCE: &str =
    "plc-comm-slmp-profiles v1.0.0 capability/slmp_builtin_ethernet_profiles.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SlmpProfileFeatureState {
    Supported,
    Blocked,
    ConfigDependent,
    #[allow(dead_code)]
    Unverified,
    Delegated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SlmpProfileFeature {
    TypeName,
    Direct,
    Random,
    Block,
    Monitor,
    ExtModuleAccess,
    ExtLinkDirect,
    HgCpuBuffer,
    LongDevicePath,
    Lz32BitPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SlmpProfileLimit {
    DirectWordRead,
    DirectWordWrite,
    DirectBitRead,
    DirectBitWrite,
    RandomReadWord,
    RandomWriteWord,
    RandomWriteBit,
    MonitorRegisterWord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SlmpCapabilityFeature {
    pub feature: SlmpProfileFeature,
    pub state: SlmpProfileFeatureState,
    pub source: &'static str,
    pub note: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SlmpCapabilityLimit {
    pub key: SlmpProfileLimit,
    pub max: usize,
    pub over_end_code: Option<&'static str>,
    pub source: &'static str,
    pub weighted_max: Option<usize>,
    pub note: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SlmpWritePolicy {
    pub device: SlmpDeviceCode,
    pub policy: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SlmpCapabilityProfile {
    pub plc_profile: SlmpPlcProfile,
    pub profile_id: &'static str,
    pub frame: &'static str,
    pub compat: &'static str,
    pub features: &'static [SlmpCapabilityFeature],
    pub limits: &'static [SlmpCapabilityLimit],
    pub write_policy: &'static [SlmpWritePolicy],
}

const IQR_FEATURES: &[SlmpCapabilityFeature] = &[
    feature(
        SlmpProfileFeature::TypeName,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Direct,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Random,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Block,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Monitor,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::ExtModuleAccess,
        SlmpProfileFeatureState::ConfigDependent,
        "live",
        Some("Module access depends on the installed special module."),
    ),
    feature(
        SlmpProfileFeature::ExtLinkDirect,
        SlmpProfileFeatureState::ConfigDependent,
        "policy",
        Some("Link-direct access depends on the network/module configuration."),
    ),
    feature(
        SlmpProfileFeature::HgCpuBuffer,
        SlmpProfileFeatureState::Supported,
        "live",
        Some("U3E0\\HG direct/random/monitor succeeded."),
    ),
    feature(
        SlmpProfileFeature::LongDevicePath,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Lz32BitPath,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
];

const IQL_FEATURES: &[SlmpCapabilityFeature] = &[
    feature(
        SlmpProfileFeature::TypeName,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Direct,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Random,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Block,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Monitor,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::ExtModuleAccess,
        SlmpProfileFeatureState::ConfigDependent,
        "live",
        Some("Module access depends on the installed special module."),
    ),
    feature(
        SlmpProfileFeature::ExtLinkDirect,
        SlmpProfileFeatureState::ConfigDependent,
        "policy",
        Some("Link-direct access depends on the network/module configuration."),
    ),
    feature(
        SlmpProfileFeature::HgCpuBuffer,
        SlmpProfileFeatureState::Blocked,
        "manual",
        Some("CPU-buffer HG is an iQ-R-only path."),
    ),
    feature(
        SlmpProfileFeature::LongDevicePath,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Lz32BitPath,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
];

const MX_FEATURES: &[SlmpCapabilityFeature] = &[
    feature(
        SlmpProfileFeature::TypeName,
        SlmpProfileFeatureState::Supported,
        "policy",
        None,
    ),
    feature(
        SlmpProfileFeature::Direct,
        SlmpProfileFeatureState::Supported,
        "policy",
        None,
    ),
    feature(
        SlmpProfileFeature::Random,
        SlmpProfileFeatureState::Supported,
        "policy",
        None,
    ),
    feature(
        SlmpProfileFeature::Block,
        SlmpProfileFeatureState::Supported,
        "policy",
        None,
    ),
    feature(
        SlmpProfileFeature::Monitor,
        SlmpProfileFeatureState::Supported,
        "policy",
        None,
    ),
    feature(
        SlmpProfileFeature::ExtModuleAccess,
        SlmpProfileFeatureState::ConfigDependent,
        "policy",
        Some("Module access depends on the installed special module."),
    ),
    feature(
        SlmpProfileFeature::ExtLinkDirect,
        SlmpProfileFeatureState::ConfigDependent,
        "policy",
        Some("Link-direct access depends on the network/module configuration."),
    ),
    feature(
        SlmpProfileFeature::HgCpuBuffer,
        SlmpProfileFeatureState::Blocked,
        "spec",
        Some("CPU-buffer HG is an iQ-R-only path."),
    ),
    feature(
        SlmpProfileFeature::LongDevicePath,
        SlmpProfileFeatureState::Supported,
        "policy",
        None,
    ),
    feature(
        SlmpProfileFeature::Lz32BitPath,
        SlmpProfileFeatureState::Supported,
        "policy",
        None,
    ),
];

const IQF_FEATURES: &[SlmpCapabilityFeature] = &[
    feature(
        SlmpProfileFeature::TypeName,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Direct,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Random,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Block,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Monitor,
        SlmpProfileFeatureState::Blocked,
        "live",
        Some("0x0801/0x0802 returned C059 on FX5U."),
    ),
    feature(
        SlmpProfileFeature::ExtModuleAccess,
        SlmpProfileFeatureState::ConfigDependent,
        "live",
        Some("U1\\G0 depends on the installed special module."),
    ),
    feature(
        SlmpProfileFeature::ExtLinkDirect,
        SlmpProfileFeatureState::Blocked,
        "live",
        Some("J1 link-direct access returned a PLC error."),
    ),
    feature(
        SlmpProfileFeature::HgCpuBuffer,
        SlmpProfileFeatureState::Blocked,
        "manual",
        Some("CPU-buffer HG is an iQ-R-only path."),
    ),
    feature(
        SlmpProfileFeature::LongDevicePath,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Lz32BitPath,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
];

const QL_MEASURED_FEATURES: &[SlmpCapabilityFeature] = &[
    feature(
        SlmpProfileFeature::TypeName,
        SlmpProfileFeatureState::Blocked,
        "live",
        Some("Read Type Name returned C059."),
    ),
    feature(
        SlmpProfileFeature::Direct,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Random,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::Block,
        SlmpProfileFeatureState::Blocked,
        "live",
        Some("Read/Write Block returned C059."),
    ),
    feature(
        SlmpProfileFeature::Monitor,
        SlmpProfileFeatureState::Supported,
        "live",
        None,
    ),
    feature(
        SlmpProfileFeature::ExtModuleAccess,
        SlmpProfileFeatureState::Blocked,
        "live",
        Some("U\\G access is not available on the tested built-in CPU port."),
    ),
    feature(
        SlmpProfileFeature::ExtLinkDirect,
        SlmpProfileFeatureState::Blocked,
        "live",
        Some("Link-direct access is not available on the tested built-in CPU port."),
    ),
    feature(
        SlmpProfileFeature::HgCpuBuffer,
        SlmpProfileFeatureState::Blocked,
        "manual",
        Some("CPU-buffer HG is an iQ-R-only path."),
    ),
    feature(
        SlmpProfileFeature::LongDevicePath,
        SlmpProfileFeatureState::Delegated,
        "policy",
        Some("Existing long-device route rules decide this feature."),
    ),
    feature(
        SlmpProfileFeature::Lz32BitPath,
        SlmpProfileFeatureState::Delegated,
        "policy",
        Some("Existing 32-bit route rules decide this feature."),
    ),
];

const IQR_LIMITS: &[SlmpCapabilityLimit] = &[
    limit(
        SlmpProfileLimit::DirectWordRead,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectWordWrite,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitRead,
        7168,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitWrite,
        7168,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomReadWord,
        96,
        Some("C054"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteWord,
        80,
        Some("C054"),
        "live",
        Some(960),
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteBit,
        94,
        Some("C053"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::MonitorRegisterWord,
        96,
        Some("C054"),
        "live",
        None,
        None,
    ),
];

const IQL_LIMITS: &[SlmpCapabilityLimit] = &[
    limit(
        SlmpProfileLimit::DirectWordRead,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectWordWrite,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitRead,
        7168,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitWrite,
        7168,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomReadWord,
        96,
        Some("C054"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteWord,
        80,
        Some("C054"),
        "live",
        Some(960),
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteBit,
        94,
        Some("C053"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::MonitorRegisterWord,
        96,
        Some("C054"),
        "live",
        None,
        None,
    ),
];

const IQF_LIMITS: &[SlmpCapabilityLimit] = &[
    limit(
        SlmpProfileLimit::DirectWordRead,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectWordWrite,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitRead,
        3584,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitWrite,
        3584,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomReadWord,
        192,
        Some("C054"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteWord,
        160,
        Some("C054"),
        "live",
        Some(1920),
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteBit,
        188,
        Some("C053"),
        "live",
        None,
        None,
    ),
];

const QL_LIMITS: &[SlmpCapabilityLimit] = &[
    limit(
        SlmpProfileLimit::DirectWordRead,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectWordWrite,
        960,
        Some("C051"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitRead,
        7168,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::DirectBitWrite,
        7168,
        Some("C052"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomReadWord,
        192,
        Some("C054"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteWord,
        160,
        Some("C054"),
        "live",
        Some(1920),
        None,
    ),
    limit(
        SlmpProfileLimit::RandomWriteBit,
        188,
        Some("C053"),
        "live",
        None,
        None,
    ),
    limit(
        SlmpProfileLimit::MonitorRegisterWord,
        192,
        Some("C054"),
        "live",
        None,
        None,
    ),
];

const IQR_WRITE_POLICY: &[SlmpWritePolicy] = &[write_policy(SlmpDeviceCode::S)];
const IQF_WRITE_POLICY: &[SlmpWritePolicy] = &[write_policy_value(SlmpDeviceCode::S, "read-write")];
const QL_WRITE_POLICY: &[SlmpWritePolicy] = &[write_policy(SlmpDeviceCode::S)];

pub(crate) const BUILTIN_CAPABILITY_PROFILES: &[SlmpCapabilityProfile] = &[
    profile(
        SlmpPlcProfile::IqR,
        "melsec:iq-r",
        "4E",
        "iQ-R",
        IQR_FEATURES,
        IQR_LIMITS,
        IQR_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::IqL,
        "melsec:iq-l",
        "4E",
        "iQ-R",
        IQL_FEATURES,
        IQL_LIMITS,
        IQR_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::MxR,
        "melsec:mx-r",
        "4E",
        "iQ-R",
        MX_FEATURES,
        IQR_LIMITS,
        IQR_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::MxF,
        "melsec:mx-f",
        "4E",
        "iQ-R",
        MX_FEATURES,
        IQR_LIMITS,
        IQR_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::IqF,
        "melsec:iq-f",
        "3E",
        "Q/L",
        IQF_FEATURES,
        IQF_LIMITS,
        IQF_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::QCpu,
        "melsec:qcpu",
        "3E",
        "Q/L",
        QL_MEASURED_FEATURES,
        QL_LIMITS,
        QL_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::LCpu,
        "melsec:lcpu",
        "3E",
        "Q/L",
        QL_MEASURED_FEATURES,
        QL_LIMITS,
        QL_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::QnU,
        "melsec:qnu",
        "3E",
        "Q/L",
        QL_MEASURED_FEATURES,
        QL_LIMITS,
        QL_WRITE_POLICY,
    ),
    profile(
        SlmpPlcProfile::QnUDV,
        "melsec:qnudv",
        "3E",
        "Q/L",
        QL_MEASURED_FEATURES,
        QL_LIMITS,
        QL_WRITE_POLICY,
    ),
];

pub(crate) fn capability_profile(
    plc_profile: SlmpPlcProfile,
) -> Option<&'static SlmpCapabilityProfile> {
    BUILTIN_CAPABILITY_PROFILES
        .iter()
        .find(|profile| profile.plc_profile == plc_profile)
}

pub(crate) fn profile_feature(
    plc_profile: SlmpPlcProfile,
    feature: SlmpProfileFeature,
) -> Option<&'static SlmpCapabilityFeature> {
    capability_profile(plc_profile)?
        .features
        .iter()
        .find(|entry| entry.feature == feature)
}

pub(crate) fn profile_limit(
    plc_profile: SlmpPlcProfile,
    key: SlmpProfileLimit,
) -> Option<&'static SlmpCapabilityLimit> {
    capability_profile(plc_profile)?
        .limits
        .iter()
        .find(|entry| entry.key == key)
}

pub(crate) fn is_profile_read_only_device(
    plc_profile: SlmpPlcProfile,
    device: SlmpDeviceCode,
) -> bool {
    capability_profile(plc_profile).is_some_and(|profile| {
        profile
            .write_policy
            .iter()
            .any(|entry| entry.device == device && entry.policy == "read-only")
    })
}

pub(crate) fn feature_key(feature: SlmpProfileFeature) -> &'static str {
    match feature {
        SlmpProfileFeature::TypeName => "type_name",
        SlmpProfileFeature::Direct => "direct",
        SlmpProfileFeature::Random => "random",
        SlmpProfileFeature::Block => "block",
        SlmpProfileFeature::Monitor => "monitor",
        SlmpProfileFeature::ExtModuleAccess => "ext_module_access",
        SlmpProfileFeature::ExtLinkDirect => "ext_link_direct",
        SlmpProfileFeature::HgCpuBuffer => "hg_cpu_buffer",
        SlmpProfileFeature::LongDevicePath => "long_device_path",
        SlmpProfileFeature::Lz32BitPath => "lz_32bit_path",
    }
}

#[cfg(test)]
pub(crate) fn limit_key(key: SlmpProfileLimit) -> &'static str {
    match key {
        SlmpProfileLimit::DirectWordRead => "direct_word_read",
        SlmpProfileLimit::DirectWordWrite => "direct_word_write",
        SlmpProfileLimit::DirectBitRead => "direct_bit_read",
        SlmpProfileLimit::DirectBitWrite => "direct_bit_write",
        SlmpProfileLimit::RandomReadWord => "random_read_word",
        SlmpProfileLimit::RandomWriteWord => "random_write_word",
        SlmpProfileLimit::RandomWriteBit => "random_write_bit",
        SlmpProfileLimit::MonitorRegisterWord => "monitor_register_word",
    }
}

pub(crate) fn state_name(state: SlmpProfileFeatureState) -> &'static str {
    match state {
        SlmpProfileFeatureState::Supported => "supported",
        SlmpProfileFeatureState::Blocked => "blocked",
        SlmpProfileFeatureState::ConfigDependent => "config-dependent",
        SlmpProfileFeatureState::Unverified => "unverified",
        SlmpProfileFeatureState::Delegated => "delegated",
    }
}

pub(crate) fn feature_evidence(feature: SlmpCapabilityFeature) -> String {
    match feature.note {
        Some(note) => format!("{}: {}; {}", feature.source, note, CANONICAL_SOURCE),
        None => format!("{}; {}", feature.source, CANONICAL_SOURCE),
    }
}

const fn feature(
    feature: SlmpProfileFeature,
    state: SlmpProfileFeatureState,
    source: &'static str,
    note: Option<&'static str>,
) -> SlmpCapabilityFeature {
    SlmpCapabilityFeature {
        feature,
        state,
        source,
        note,
    }
}

const fn limit(
    key: SlmpProfileLimit,
    max: usize,
    over_end_code: Option<&'static str>,
    source: &'static str,
    weighted_max: Option<usize>,
    note: Option<&'static str>,
) -> SlmpCapabilityLimit {
    SlmpCapabilityLimit {
        key,
        max,
        over_end_code,
        source,
        weighted_max,
        note,
    }
}

const fn write_policy(device: SlmpDeviceCode) -> SlmpWritePolicy {
    write_policy_value(device, "read-only")
}

const fn write_policy_value(device: SlmpDeviceCode, policy: &'static str) -> SlmpWritePolicy {
    SlmpWritePolicy { device, policy }
}

const fn profile(
    plc_profile: SlmpPlcProfile,
    profile_id: &'static str,
    frame: &'static str,
    compat: &'static str,
    features: &'static [SlmpCapabilityFeature],
    limits: &'static [SlmpCapabilityLimit],
    write_policy: &'static [SlmpWritePolicy],
) -> SlmpCapabilityProfile {
    SlmpCapabilityProfile {
        plc_profile,
        profile_id,
        frame,
        compat,
        features,
        limits,
        write_policy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::BTreeMap;

    #[test]
    fn builtin_capability_profiles_match_canonical_fixture() {
        let fixture = include_str!("../tests/fixtures/slmp_builtin_ethernet_profiles.json");
        let expected: Value = serde_json::from_str(fixture).unwrap();
        let expected_profiles = expected["profiles"].as_object().unwrap();
        let actual: BTreeMap<_, _> = BUILTIN_CAPABILITY_PROFILES
            .iter()
            .map(|profile| (profile.profile_id, *profile))
            .collect();

        assert_eq!(
            expected_profiles.keys().cloned().collect::<Vec<_>>(),
            actual
                .keys()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
        );

        for (profile_id, expected_profile) in expected_profiles {
            let actual_profile = actual[profile_id.as_str()];
            assert_eq!(
                actual_profile.frame,
                expected_profile["frame"].as_str().unwrap()
            );
            assert_eq!(
                actual_profile.compat,
                expected_profile["compat"].as_str().unwrap()
            );

            let expected_features = expected_profile["features"].as_object().unwrap();
            let actual_features: BTreeMap<_, _> = actual_profile
                .features
                .iter()
                .map(|entry| (feature_key(entry.feature), state_name(entry.state)))
                .collect();
            assert_eq!(
                expected_features.keys().cloned().collect::<Vec<_>>(),
                actual_features
                    .keys()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            );
            for (key, expected_feature) in expected_features {
                assert_eq!(
                    actual_features[key.as_str()],
                    expected_feature["state"].as_str().unwrap()
                );
            }

            let expected_limits = expected_profile["limits"].as_object().unwrap();
            let actual_limits: BTreeMap<_, _> = actual_profile
                .limits
                .iter()
                .map(|entry| (limit_key(entry.key), *entry))
                .collect();
            assert_eq!(
                expected_limits.keys().cloned().collect::<Vec<_>>(),
                actual_limits
                    .keys()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            );
            for (key, expected_limit) in expected_limits {
                let actual_limit = actual_limits[key.as_str()];
                assert_eq!(
                    actual_limit.max as u64,
                    expected_limit["max"].as_u64().unwrap()
                );
                assert_eq!(
                    actual_limit.weighted_max.map(|value| value as u64),
                    expected_limit.get("weighted_max").and_then(Value::as_u64)
                );
            }

            let expected_policy = expected_profile["write_policy"].as_object().unwrap();
            let actual_policy: BTreeMap<_, _> = actual_profile
                .write_policy
                .iter()
                .map(|entry| (entry.device.prefix(), entry.policy))
                .collect();
            assert_eq!(
                expected_policy.keys().cloned().collect::<Vec<_>>(),
                actual_policy
                    .keys()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            );
            for (device, expected_value) in expected_policy {
                assert_eq!(
                    actual_policy[device.as_str()],
                    expected_value.as_str().unwrap()
                );
            }
        }
    }
}
