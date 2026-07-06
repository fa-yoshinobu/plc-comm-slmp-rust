# v2.0.0

## BREAKING

crates.io package renamed and Rust import path standardized.

| Old crate/use | New crate/use |
| --- | --- |
| `plc-comm-slmp-rust` | `plc-comm-slmp` |
| `use plc_comm_slmp_rust::...` | `use plc_comm_slmp::...` |

Short `SlmpModuleIo` aliases were removed in favor of canonical module I/O names.

| Old name | New name |
| --- | --- |
| `CONTROL_CPU`, `CONNECTED_CPU`, `DEFAULT` | `OWN_STATION` |
| `ACTIVE_CPU` | `CONTROL_SYSTEM_CPU` |
| `STANDBY_CPU` | `STANDBY_SYSTEM_CPU` |
| `TYPE_A_CPU` | `SYSTEM_A_CPU` |
| `TYPE_B_CPU` | `SYSTEM_B_CPU` |
| `CPU1` / `CPU_1` to `CPU4` / `CPU_4` | `MULTIPLE_CPU_1` to `MULTIPLE_CPU_4` |

## Highlights

- Version metadata bumped to 2.0.0.
- Release duplicate checks now target `plc-comm-slmp`.
- SLMP profile fixture synced to `plc-comm-slmp-profiles` v1.2.2.
- README and Getting Started use the new crate name.

Package matrix: https://fa-yoshinobu.github.io/plc-comm-docs-site/package-matrix/
