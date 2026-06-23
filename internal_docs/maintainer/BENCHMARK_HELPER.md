# Benchmark Helper

This repository includes an optional `slmp_bench_client` binary for integration
with `plc-comm-bench-dotnet`.

```bash
cargo build --release --features cli --bin slmp_bench_client
```

The helper keeps one Rust SLMP connection open while it runs warmup and measured
iterations, then prints one JSON result. It is not part of the library API.
