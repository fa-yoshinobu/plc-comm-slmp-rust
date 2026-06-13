# Examples

These examples are runnable entry points for the public API. Defaults are `SLMP_HOST=192.168.250.100`, TCP `SLMP_PORT=1025`, and UDP `SLMP_PORT=1035`. Use `SLMP_PLC_PROFILE` to select your PLC.

| Example | Command | Purpose |
| --- | --- | --- |
| `01_minimal` | `cargo run --example 01_minimal` | Connects to `192.168.250.100:1025`, reads `D100`, prints the value, and disconnects. |
| `raw_read_write` | `cargo run --features cli --example raw_read_write` | Reads words with the raw client and optionally writes test words. |
| `named_helpers` | `cargo run --features cli --example named_helpers` | Uses `read_named`, `write_named`, and `poll_named`. |
| `advanced_operations` | `cargo run --features cli --example advanced_operations` | Demonstrates type-name, random, block, extended, and self-test operations. |
| `device_range_catalog` | `cargo run --features cli --example device_range_catalog` | Reads the live device-range catalog for the selected profile. |
| `device_matrix_compare` | `cargo run --features cli --example device_matrix_compare` | Compares multiple read/write routes for the same device addresses. |
| `extended_device_coverage` | `cargo run --features cli --example extended_device_coverage` | Sweeps qualified extended-device addresses and writes a Markdown report. |
| `device_range_sample_compare` | `cargo run --features cli --example device_range_sample_compare` | Samples supported device ranges, writes test values, verifies, and restores. |
| `route_validation_compare` | `cargo run --features cli --example route_validation_compare` | Validates helper and raw route behavior on a live PLC. |
| `iql_live_stress` | `cargo run --features cli --example iql_live_stress` | Runs write/read-back stress checks for an iQ-L target. |

Common profile values:

| PLC profile | Environment value |
| --- | --- |
| `SlmpPlcProfile::IqF` | `SLMP_PLC_PROFILE=melsec:iq-f` |
| `SlmpPlcProfile::IqR` | `SLMP_PLC_PROFILE=melsec:iq-r` |
| `SlmpPlcProfile::IqL` | `SLMP_PLC_PROFILE=melsec:iq-l` |
| `SlmpPlcProfile::QCpu` | `SLMP_PLC_PROFILE=melsec:qcpu` |
| `SlmpPlcProfile::LCpu` | `SLMP_PLC_PROFILE=melsec:lcpu` |
| `SlmpPlcProfile::QnU` | `SLMP_PLC_PROFILE=melsec:qnu` |
| `SlmpPlcProfile::QnUDV` | `SLMP_PLC_PROFILE=melsec:qnudv` |
