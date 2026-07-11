# Examples

These examples are runnable entry points for the public API. The destination
port, transport, complete target route, and PLC profile are required. For an
own-station TCP example, set `SLMP_PORT=1025`, `SLMP_TRANSPORT=tcp`,
`SLMP_TARGET=SELF`, and `SLMP_PLC_PROFILE=melsec:iq-r` explicitly. A host value
shown by an example is illustrative, not a discovered PLC endpoint.

Use only test addresses that are safe for your PLC program before you run any write example.

| Example | Command | Purpose |
| --- | --- | --- |
| `01_minimal` | `cargo run --example 01_minimal` | Connects to `192.168.250.100:1025`, reads `D100`, prints the value, and disconnects. |
| `raw_read_write` | `cargo run --features cli --example raw_read_write` | Reads words with the raw client and optionally writes test words. |
| `named_helpers` | `cargo run --features cli --example named_helpers` | Uses `read_named`, `write_named`, and `poll_named`. |
| `polling_reconnect` | `cargo run --features cli --example polling_reconnect -- D100 U 1` | Read-only polling loop with automatic reconnect and backoff after transport loss. |
| `multi_plc_monitor` | `cargo run --features cli --example multi_plc_monitor -- --plc line-a=192.168.250.101,melsec:iq-r,1035,udp,SELF --tag d100=D100:U --cycles 1 --dry-run` | Read-only multi-PLC polling with `connected`/`lost`/`reconnecting`/`recovered` states. |
| `config_polling` | `cargo run --features cli --example config_polling -- --config examples/config_polling.example.json --dry-run` | Read-only polling from JSON config, with long-form `timestamp,plc,tag,value` CSV output. |
| `advanced_operations` | `cargo run --features cli --example advanced_operations` | Demonstrates type-name, random, block, extended, and self-test operations. |
| `device_range_catalog` | `cargo run --features cli --example device_range_catalog` | Reads the live device-range catalog for the selected profile. |

Maintainer validation examples live in `examples/validation/`. They remain runnable with their existing `cargo run --example ...` names because they are declared in `Cargo.toml`.

Common profile values:

| PLC profile | Environment value |
| --- | --- |
| `SlmpPlcProfile::IqF` | `SLMP_PLC_PROFILE=melsec:iq-f` |
| `SlmpPlcProfile::IqR` | `SLMP_PLC_PROFILE=melsec:iq-r` |
| `SlmpPlcProfile::IqRRj71En71` | `SLMP_PLC_PROFILE=melsec:iq-r:rj71en71` |
| `SlmpPlcProfile::IqL` | `SLMP_PLC_PROFILE=melsec:iq-l` |
| `SlmpPlcProfile::LCpu` | `SLMP_PLC_PROFILE=melsec:lcpu` |
| `SlmpPlcProfile::LCpuLj71E71100` | `SLMP_PLC_PROFILE=melsec:lcpu:lj71e71-100` |
| `SlmpPlcProfile::QCpuQj71E71100` | `SLMP_PLC_PROFILE=melsec:qcpu:qj71e71-100` |
| `SlmpPlcProfile::QnU` | `SLMP_PLC_PROFILE=melsec:qnu` |
| `SlmpPlcProfile::QnUQj71E71100` | `SLMP_PLC_PROFILE=melsec:qnu:qj71e71-100` |
| `SlmpPlcProfile::QnUDV` | `SLMP_PLC_PROFILE=melsec:qnudv` |
| `SlmpPlcProfile::QnUDVQj71E71100` | `SLMP_PLC_PROFILE=melsec:qnudv:qj71e71-100` |

`melsec:qcpu` is base-only; use `melsec:qcpu:qj71e71-100` for QCPU Ethernet unit connections.
