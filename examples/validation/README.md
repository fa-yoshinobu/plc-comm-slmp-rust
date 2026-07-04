# SLMP Rust Validation Examples

These examples are maintainer lab validation tools, not ordinary user samples.
They can write many PLC devices, sweep routes, or run stress loops.

Run them from the repository root against a prepared test PLC only.

```bash
cargo run --features cli --example device_matrix_compare
cargo run --features cli --example extended_device_coverage
cargo run --features cli --example device_range_sample_compare
cargo run --features cli --example route_validation_compare
cargo run --features cli --example iql_live_stress
```

The source files live under `examples/validation/`, but the command names are
kept stable by explicit `[[example]]` entries in `Cargo.toml`.
