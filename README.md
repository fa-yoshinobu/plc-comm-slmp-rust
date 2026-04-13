# plc-comm-slmp-rust

Rust implementation of the SLMP library, aligned with `plc-comm-slmp-dotnet`
and intended to join `plc-comm-slmp-cross-verify`.

## Current Scope

- Binary 3E / 4E over TCP and UDP
- Raw device read/write
- Random read/write
- Block read/write
- Extended device read/write
- Memory and extend-unit access
- Remote operations and self test
- High-level typed read/write and named read/write helpers
- Cross-verify wrapper binary: `slmp_verify_client`

## Development

```bash
cargo test
cargo run --bin slmp_verify_client -- 127.0.0.1 9000 read-type
```
