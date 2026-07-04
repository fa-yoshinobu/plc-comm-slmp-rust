# Getting started

## Start here

Use this page when you want the shortest path from a Rust program to a MELSEC PLC. The examples use TCP at `192.168.250.100:1025`.

## Prerequisites

| Requirement | Value |
| --- | --- |
| Rust | Stable Rust |
| Async runtime | `tokio` with `macros` and a runtime feature |
| PLC endpoint | `192.168.250.100:1025` |
| Profile used below | `SlmpPlcProfile::IqR` |

## Add dependency

```bash
cargo add plc-comm-slmp-rust
cargo add tokio --features macros,rt-multi-thread
```

## Choose your PLC profile

`SlmpPlcProfile` is the only required selector. The library derives the SLMP frame and compatibility mode from it.

```rust
use plc_comm_slmp::{SlmpConnectionOptions, SlmpPlcProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;
    println!("{:?}", options.plc_profile());

    Ok(())
}
```

## First read

This reads `D100` as an unsigned 16-bit word.

```rust
use plc_comm_slmp::{
    read_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let value = read_typed(&client, SlmpAddress::parse("D100")?, "U").await?;
    println!("{:?}", value);
    client.close().await?;

    Ok(())
}
```

Expected output shape:

```text
U16(123)
```

## First write

Only write to a test address that your PLC program allows you to change.

```rust
use plc_comm_slmp::{
    read_typed, write_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
    SlmpValue,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let address = SlmpAddress::parse("D600")?;

    let original = read_typed(&client, address, "U").await?;
    write_typed(&client, address, "U", &SlmpValue::U16(42)).await?;
    let value = read_typed(&client, address, "U").await?;
    println!("{:?}", value);
    write_typed(&client, address, "U", &original).await?;
    client.close().await?;

    Ok(())
}
```

## Confirm success

1. The PLC IP address is reachable from your computer.
2. TCP port `1025` is open on your PLC Ethernet module or built-in Ethernet port.
3. The PLC-side communication data code is Binary and the port/open setting matches your transport; see the [MELSEC SLMP PLC Setup Guide](https://fa-yoshinobu.github.io/plc-comm-docs-site/plc-setup/slmp/).
4. PLC-side RUN-time write permission is enabled before you run a write example where the PLC exposes that setting.
5. `SlmpPlcProfile::IqR` matches your real hardware, or you selected the correct variant from [profiles](PROFILES.md).
6. A read from `D100` returns a `SlmpValue` without an SLMP end code error.
7. Any write test uses a register reserved for testing and restores the original value.

## If it does not work

| Symptom | Check |
| --- | --- |
| You get SLMP end code errors | `SlmpPlcProfile` must match the actual hardware. |
| Connection opens but all requests fail | Confirm Binary communication data code in the PLC setup guide. |
| Reads work but writes fail | Confirm RUN-time write permission in the PLC setup guide and the selected profile write policy. |
| You are tempted to set the frame manually | Do not override frame type manually; it is derived from the profile. |
| A special device family fails | Start with `D` reads, not `G`, `HG`, `LTN`, or `LCN`. |
| `X` or `Y` addresses look different on iQ-F | `SlmpPlcProfile::IqF` parses `X` and `Y` string addresses as octal. |

## Next pages

| Page | Link |
| --- | --- |
| Usage guide | [USAGE_GUIDE.md](USAGE_GUIDE.md) |
| SLMP profile reference | [https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/) |
