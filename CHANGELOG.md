# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Changed
- `SlmpConnectionOptions::new(...)` now requires `SlmpPlcFamily`, and the standard connection path derives frame/profile defaults from that explicit family.
- `read_device_range_catalog()` now uses the configured PLC family directly instead of model-name auto resolution.

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
