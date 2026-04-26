//! Async Rust client for Mitsubishi SLMP Binary 3E and 4E.
//!
//! This crate follows the same operation semantics as the sibling
//! `plc-comm-slmp-python`, `.NET`, `C++`, `Node-RED`, and `cross-verify`
//! projects in the same family. The intended flow is:
//!
//! 1. connect with [`SlmpConnectionOptions`] and [`SlmpClient`]
//! 2. use raw device APIs for low-level control
//! 3. use helper APIs such as [`read_named`] and [`write_named`] for
//!    application-facing snapshots and typed values
//! 4. validate behavior through `plc-comm-slmp-cross-verify`
//!
//! # Quick Start
//!
//! ```no_run
//! use plc_comm_slmp::{
//!     SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcFamily,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime = tokio::runtime::Builder::new_current_thread()
//!     .enable_all()
//!     .build()?;
//! runtime.block_on(async {
//!     let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcFamily::IqR);
//!     options.port = 1025;
//!
//!     let client = SlmpClient::connect(options).await?;
//!     let words = client.read_words_raw(SlmpAddress::parse("D100")?, 2).await?;
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
//!     NamedAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcFamily, SlmpValue, read_named,
//!     write_named,
//! };
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let runtime = tokio::runtime::Builder::new_current_thread()
//!     .enable_all()
//!     .build()?;
//! runtime.block_on(async {
//!     let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcFamily::IqF);
//!     options.port = 1025;
//!     let client = SlmpClient::connect(options).await?;
//!
//!     let snapshot = read_named(
//!         &client,
//!         &["D100".into(), "X100".into(), "D50.3".into(), "LTN10:D".into()],
//!     )
//!     .await?;
//!     println!("{snapshot:?}");
//!
//!     let mut updates = NamedAddress::new();
//!     updates.insert("D300".into(), SlmpValue::U16(42));
//!     updates.insert("D400:F".into(), SlmpValue::F32(3.14));
//!     write_named(&client, &updates).await?;
//!     Ok(())
//! })
//! # }
//! ```
//!
//! # Address Notes
//!
//! - Plain word devices: `D100`, `R50`, `ZR0`
//! - Plain bit devices: `M100`, `X20`, `Y20`, `B10`
//! - Typed suffixes: `D200:F`, `D300:D`, `D400:L`
//! - Bit-in-word form: `D50.3`
//! - Long current values: `LTN10:D`, `LSTN20:D`, `LCN30:D`
//! - Extended devices: `J1\\W10`, `U3\\G100`, `U1\\HG0`
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
//! # Verification
//!
//! This crate is meant to participate in `plc-comm-slmp-cross-verify`.
//! The canonical wrapper binary is `slmp_verify_client`.
//!
mod address;
mod client;
mod connection_profile_probe;
mod device_ranges;
mod error;
mod helpers;
mod model;

pub use address::{
    SlmpAddress, normalize_named_address, parse_named_address, parse_named_target,
    parse_qualified_device, parse_target_auto_number,
};
pub use client::{SlmpClient, encode_device_spec};
pub use connection_profile_probe::{
    SlmpConnectionProfileProbeResult, SlmpConnectionProfileProbeStatus,
    SlmpResolvedDeviceRangeCatalog, probe_connection_profiles,
    read_device_range_catalog_with_three_e_legacy_fallback,
};
pub use device_ranges::{
    SlmpDeviceRangeCatalog, SlmpDeviceRangeCategory, SlmpDeviceRangeEntry, SlmpDeviceRangeFamily,
    SlmpDeviceRangeNotation,
};
pub use error::SlmpError;
pub use helpers::{
    NamedAddress, SlmpValue, parse_scalar_for_named, parse_scalar_for_named_with_family,
    poll_named, read_dwords_chunked, read_dwords_single_request, read_named, read_typed,
    read_words_chunked, read_words_single_request, write_bit_in_word, write_dwords_chunked,
    write_dwords_single_request, write_named, write_typed, write_words_chunked,
    write_words_single_request,
};
pub use model::{
    SlmpBlockRead, SlmpBlockReadResult, SlmpBlockWrite, SlmpBlockWriteOptions, SlmpCommand,
    SlmpCompatibilityMode, SlmpConnectionOptions, SlmpCpuOperationState, SlmpCpuOperationStatus,
    SlmpDeviceAddress, SlmpDeviceCode, SlmpExtensionSpec, SlmpFrameType, SlmpLongTimerResult,
    SlmpNamedTarget, SlmpPlcFamily, SlmpQualifiedDeviceAddress, SlmpRandomReadResult,
    SlmpTargetAddress, SlmpTraceDirection, SlmpTrafficStats, SlmpTransportMode, SlmpTypeNameInfo,
};
