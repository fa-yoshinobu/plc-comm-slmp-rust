# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.1.6] - 2026-04-27

### Changed
- Tightened long-device route guards so `LTN/LSTN/LCN/LZ` avoid unsupported direct/raw word and dword paths, while supported random/named dword paths remain available.
- Aligned `LCS/LCC` write validation with the random/named bit route policy.
- Added UDP transport selection to `slmp_verify_client` for cross-library real-device validation.

## [0.1.5] - 2026-04-20

### Fixed
- `slmp_verify_client` now parses named write values with the active `SlmpPlcFamily`, so `X`/`Y` addresses work in cross-verify named-write flows.

### Changed
- CI now builds `slmp_verify_client` with the required `cli` feature enabled.

## [0.1.4] - 2026-04-14

### Changed
- The standard connection route now requires explicit `SlmpPlcFamily`, and the normal client path derives frame/profile defaults from that family instead of exposing profile selection as the application-facing route.
- `read_device_range_catalog()` now follows the configured family directly, while profile probing remains a separate diagnostic helper.

## [0.1.3] - 2026-04-14

### Added
- `SlmpPlcFamily` defaults and the family-driven high-level helper surface.

### Changed
- High-level string device parsing and device-range catalog reads now use explicit PLC-family rules. `X/Y` strings require explicit family, `iQ-F` uses octal, and other supported families use hexadecimal.

## [0.1.2] - 2026-04-14

### Added
- Public device-range catalog support, connection-profile probing, and runnable examples for device-range lookup and CPU operation-state inspection.

### Changed
- Refreshed the README and docs so the new device-range and profile-probe helpers are documented alongside the existing client surface.

## [0.1.1] - 2026-04-13

### Added
- Expanded README, address guide, recipe guide, and runnable examples for raw reads/writes, named helpers, advanced operations, and live device-matrix comparison.

### Changed
- Long timer and long retentive timer device handling now blocks unsupported direct-read paths and uses only the supported helper and block-read routes.
- `LCS/LCC` now reject unsupported random, block, and monitor-registration commands so the Rust client matches the shared cross-library consistency rules.
