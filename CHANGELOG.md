# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.1.1] - 2026-04-13

### Added
- Expanded README, address guide, recipe guide, and runnable examples for raw reads/writes, named helpers, advanced operations, and live device-matrix comparison.

### Changed
- Long timer and long retentive timer device handling now blocks unsupported direct-read paths and uses only the supported helper and block-read routes.
- `LCS/LCC` now reject unsupported random, block, and monitor-registration commands so the Rust client matches the shared cross-library consistency rules.
