// SlmpError is the public concrete error type; boxing it would be a breaking API change.
#![allow(clippy::result_large_err)]

//! Async Rust client for MELSEC SLMP Binary 3E and 4E.
//!
//! This crate follows the same operation semantics as the sibling
//! `plc-comm-slmp-python`, `.NET`, `C++`, and `Node-RED` libraries. The
//! intended flow is:
//!
//! 1. connect with [`SlmpConnectionOptions`] and [`SlmpClient`]
//! 2. use raw device APIs for low-level control
//! 3. use helper APIs such as [`read_named`] and [`write_named`] for
//!    application-facing snapshots and typed values
//! 4. validate changes with this repository's Rust tests and examples
//!
//! # Quick Start
//!
//! ```no_run
//! use plc_comm_slmp::{
//!     SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpTargetAddress,
//!     SlmpTransportMode,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime = tokio::runtime::Builder::new_current_thread()
//!     .enable_all()
//!     .build()?;
//! runtime.block_on(async {
//!     let options = SlmpConnectionOptions::new("192.168.250.100", 1025, SlmpTransportMode::Tcp, SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;
//!
//!     let client = SlmpClient::connect(options).await?;
//!     let words = client.read_words_raw(SlmpAddress::parse("D100", plc_comm_slmp::SlmpPlcProfile::IqR)?, 2).await?;
//!     println!("{words:?}");
//!     Ok(())
//! })
//! # }
//! ```
//!
//! # Recommended High-Level Helpers
//!
//! ```no_run
//! use plc_comm_slmp::{
//!     NamedAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpValue, read_named,
//!     SlmpTargetAddress, SlmpTransportMode, write_named,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime = tokio::runtime::Builder::new_current_thread()
//!     .enable_all()
//!     .build()?;
//! runtime.block_on(async {
//!     let options = SlmpConnectionOptions::new("192.168.250.100", 1025, SlmpTransportMode::Tcp, SlmpTargetAddress::default(), SlmpPlcProfile::IqF)?;
//!     let client = SlmpClient::connect(options).await?;
//!
//!     let snapshot = read_named(
//!         &client,
//!         &["D100:U".into(), "X100:BIT".into(), "D50.3".into(), "LTN10:D".into()],
//!     )
//!     .await?;
//!     println!("{snapshot:?}");
//!
//!     let mut updates = NamedAddress::new();
//!     updates.insert("D300:U".into(), SlmpValue::U16(42));
//!     updates.insert("D400:F".into(), SlmpValue::F32(3.14));
//!     write_named(&client, &updates).await?;
//!     Ok(())
//! })
//! # }
//! ```
//!
//! # Address Notes
//!
//! - Word values in named helpers: `D100:U`, `R50:U`, `ZR0:U`
//! - Bit values in named helpers: `M100:BIT`, `X20:BIT`, `Y20:BIT`, `B10:BIT`
//! - Typed suffixes: `D200:F`, `D300:D`, `D400:L`
//! - Bit-in-word form: `D50.3`
//! - Long current values: `LTN10:D`, `LSTN20:D`, `LCN30:D`
//! - Extended devices: `J1\\W10`, `U3\\G100`, `U3E0\\HG0`
//!
//! `.bit` notation is only valid for word devices. Long timer state reads
//! (`LTS`, `LTC`, `LSTS`, `LSTC`) are decoded through the corresponding
//! current-value blocks. Long counter state reads (`LCS`, `LCC`) use direct bit
//! read. `LCN` current values use random dword access in the high-level helpers,
//! and high-level state writes use random bit write (`0x1402`).
//!
//! # Examples
//!
//! The repository includes runnable examples under `examples/`:
//!
//! - `raw_read_write`
//! - `named_helpers`
//! - `advanced_operations`
//!
//! Run them with `cargo run --features cli --example <name>`.
//!
mod address;
mod capability_profiles;
mod client;
mod client_rules;
mod device_range_sample;
mod device_ranges;
mod error;
mod error_codes;
mod helpers;
mod model;
mod route_validation;

pub use address::{
    SlmpAddress, normalize_named_address, parse_device, parse_named_address, parse_named_target,
    parse_qualified_device, parse_target_auto_number,
};
pub use client::{SlmpClient, encode_raw_device_spec};
pub use device_range_sample::{
    SlmpDeviceRangeSampleDeviceReport, SlmpDeviceRangeSampleFailure, SlmpDeviceRangeSampleOptions,
    SlmpDeviceRangeSampleReport, SlmpDeviceRangeSampleSummary, SlmpDeviceRangeSampleValueKind,
    run_device_range_sample_compare,
};
pub use device_ranges::{
    SlmpDeviceRangeCatalog, SlmpDeviceRangeCategory, SlmpDeviceRangeEntry, SlmpDeviceRangeNotation,
};
pub use error::{SlmpError, SlmpErrorInfo, SlmpErrorKind, SlmpProfileFeatureErrorInfo};
pub use error_codes::{end_code_key, end_code_name, is_remote_password_end_code};
pub use helpers::{
    NamedAddress, SlmpValue, parse_scalar_for_named, poll_named, read_dwords_single_request,
    read_named, read_typed, read_words_single_request, write_bit_in_word,
    write_dwords_single_request, write_named, write_typed, write_words_single_request,
};
pub use model::{
    RawSlmpDeviceAddress, SlmpBlockRead, SlmpBlockReadResult, SlmpBlockWrite, SlmpCommand,
    SlmpCompatibilityMode, SlmpConnectionOptions, SlmpCpuOperationState, SlmpCpuOperationStatus,
    SlmpDeviceAddress, SlmpDeviceCode, SlmpDeviceModification, SlmpFrameType,
    SlmpLabelArrayReadPoint, SlmpLabelArrayReadResult, SlmpLabelArrayWritePoint,
    SlmpLabelRandomReadResult, SlmpLabelRandomWritePoint, SlmpLongTimerResult, SlmpModuleIo,
    SlmpNamedTarget, SlmpPlcProfile, SlmpPlcProfileDescriptor, SlmpQualifiedDeviceAddress,
    SlmpRandomReadResult, SlmpRemoteClearMode, SlmpRemoteMode, SlmpTargetAddress, SlmpTrafficStats,
    SlmpTransportMode, SlmpTypeNameInfo, plc_profile_descriptors,
};
pub use route_validation::{
    SlmpRouteValidationCase, SlmpRouteValidationOptions, SlmpRouteValidationReport,
    SlmpRouteValidationStatus, SlmpRouteValidationSummary, run_route_validation_compare,
};
