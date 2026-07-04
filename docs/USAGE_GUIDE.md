# Usage guide

## Recommended entry points

| Entry point | Use it for |
| --- | --- |
| `SlmpConnectionOptions::new(host, plc_profile)` | Creating a connection configuration with profile-derived defaults. |
| `SlmpClient::connect(options)` | Opening a TCP or UDP SLMP client. |
| `SlmpAddress::parse("D100")` | Parsing a device address into `SlmpDeviceAddress`. |
| `read_latest_self_diagnosis_error_code` | Reading the latest PLC self-diagnosis error code from `SD0`. |
| `read_typed` and `write_typed` | Reading or writing one scalar value. |
| `read_named` and `write_named` | Reading or writing a small mixed snapshot by address text. |
| `read_words_single_request` and `read_words_chunked` | Reading contiguous word ranges. |
| `read_dwords_single_request` and `read_dwords_chunked` | Reading contiguous 32-bit ranges. |
| `write_bit_in_word` | Updating one bit inside a word register. |
| `poll_named` | Repeating a named snapshot on an interval. |

## Connection

`SlmpConnectionOptions` exposes user-settable connection fields. The PLC profile, frame type, and compatibility mode are selected together by `SlmpConnectionOptions::new`.

| Field | Default | Meaning |
| --- | --- | --- |
| `host` | value passed to `new` | PLC host name or IP address. |
| `port` | `1025` | TCP or UDP port. |
| `timeout` | 3 seconds | Socket read/write timeout. |
| `tcp_keepalive` | 30 seconds | TCP keepalive idle time, or `None`. |
| `target` | self CPU target | SLMP target address fields. |
| `transport_mode` | `SlmpTransportMode::Tcp` | TCP or UDP. |
| `monitoring_timer` | `0x0010` | SLMP monitoring timer. |

```rust
use std::time::Duration;

use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpTransportMode,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;
    options.timeout = Duration::from_secs(3);
    options.transport_mode = SlmpTransportMode::Tcp;

    let client = SlmpClient::connect(options).await?;
    println!("{:?}", client.plc_profile().await);
    client.close().await?;

    Ok(())
}
```

## Remote password

Remote password lock/unlock commands are available on `SlmpClient`.
The Rust high-level connection does not automatically unlock or lock a remote password.
If your PLC route uses remote password protection, unlock after connecting and lock before closing.

```rust
client.remote_password_unlock("secret").await?;
let value = read_typed(&client, SlmpAddress::parse("D100")?, "U").await?;
client.remote_password_lock("secret").await?;
```

For `C200`-series password end codes, see the shared
[SLMP Troubleshooting & End Codes](https://fa-yoshinobu.github.io/plc-comm-docs-site/plc-setup/slmp/troubleshooting-end-codes/)
page.

## Routing / target station

Most applications keep the default target, which means the directly connected
own station/control CPU. Change the target only when your PLC network is
configured for another station, multi-CPU module I/O, or multidrop access.

`SlmpTargetAddress` controls the SLMP destination header. It is not a device
family selector; routed devices such as `Un\Gn` and `Jn\...` still need their
own address syntax.

```rust
use plc_comm_slmp::{SlmpConnectionOptions, SlmpPlcProfile, SlmpTargetAddress};

let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
options.port = 1025;
options.target = SlmpTargetAddress {
    network: 0x01,
    station: 0x02,
    module_io: 0x03FF,
    multidrop: 0x00,
};
```

Use the default target unless the PLC routing setup gives you specific values.

## PLC diagnostics

`SlmpClient::read_latest_self_diagnosis_error_code` reads `SD0`, the latest PLC self-diagnosis error code, and returns the raw 16-bit value. Format it as hexadecimal when displaying it.

This value is separate from `SlmpError.end_code`. `SlmpError.end_code` is the SLMP response end code for a communication request, while `SD0` is the PLC CPU's self-diagnosis error register.

```rust
use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let error_code = client.read_latest_self_diagnosis_error_code().await?;
    println!("latest self-diagnosis error code: 0x{error_code:04X}");
    client.close().await?;

    Ok(())
}
```

## SLMP response end codes

When the PLC returns a non-zero SLMP end code, high-level calls return `SlmpError`.
Read `end_code` for the PLC response code and `error_info` when the PLC returned the structured error-information block.

```rust
match read_typed(&client, "D100", "U").await {
    Ok(value) => println!("D100 = {value:?}"),
    Err(error) => {
        if let Some(end_code) = error.end_code {
            println!("SLMP end_code=0x{end_code:04X}");
        }
        if let Some(info) = error.error_info.as_ref() {
            println!("command=0x{:04X}", info.command);
            println!("subcommand=0x{:04X}", info.subcommand);
        }
    }
}
```

## Read a single value

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

| Suffix | Rust value | Meaning |
| --- | --- | --- |
| `U` | `SlmpValue::U16` | Unsigned 16-bit word. |
| `S` | `SlmpValue::I16` | Signed 16-bit word. |
| `D` | `SlmpValue::U32` | Unsigned 32-bit value. |
| `L` | `SlmpValue::I32` | Signed 32-bit value. |
| `F` | `SlmpValue::F32` | 32-bit float. |
| `BIT` | `SlmpValue::Bool` | Bit device value. |

## Write a single value

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
    let device = SlmpAddress::parse("D600")?;

    let original = read_typed(&client, device, "U").await?;
    write_typed(&client, device, "U", &SlmpValue::U16(42)).await?;
    let value = read_typed(&client, device, "U").await?;
    println!("{:?}", value);
    write_typed(&client, device, "U", &original).await?;
    client.close().await?;

    Ok(())
}
```

## Named snapshot

```rust
use plc_comm_slmp::{
    read_named, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let addresses = vec![
        "D100:U".to_string(),
        "D200:F".to_string(),
        "D50.3".to_string(),
        "M100:BIT".to_string(),
        "LTN10:D".to_string(),
    ];
    let snapshot = read_named(&client, &addresses).await?;
    println!("{:?}", snapshot);
    client.close().await?;

    Ok(())
}
```

## Block reads

```rust
use plc_comm_slmp::{
    read_words_chunked, read_words_single_request, SlmpAddress, SlmpClient,
    SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let start = SlmpAddress::parse("D100")?;

    let single = read_words_single_request(&client, start, 8).await?;
    let chunked = read_words_chunked(&client, start, 128, 32).await?;
    println!("{:?}", single);
    println!("{:?}", chunked);
    client.close().await?;

    Ok(())
}
```

## Bit in word

Use `.n` notation when reading through `read_named`, and use `write_bit_in_word` when you need to update one bit inside a word.

```rust
use plc_comm_slmp::{
    read_named, read_typed, write_bit_in_word, write_typed, SlmpAddress, SlmpClient,
    SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let word = SlmpAddress::parse("D50")?;
    let original = read_typed(&client, word, "U").await?;
    write_bit_in_word(&client, word, 3, true).await?;

    let addresses = vec!["D50.3".to_string()];
    let snapshot = read_named(&client, &addresses).await?;
    println!("{:?}", snapshot);
    write_typed(&client, word, "U", &original).await?;
    client.close().await?;

    Ok(())
}
```

## Polling

```rust
use std::time::Duration;

use futures_util::StreamExt;
use plc_comm_slmp::{
    poll_named, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let addresses = vec!["D100:U".to_string(), "M100:BIT".to_string(), "D50.3".to_string()];
    let mut stream = Box::pin(poll_named(&client, &addresses, Duration::from_millis(500)));

    if let Some(snapshot) = stream.next().await.transpose()? {
        println!("{:?}", snapshot);
    }
    client.close().await?;

    Ok(())
}
```

## Runnable examples

The repository examples are designed to run from environment variables.

| Variable | Default | Meaning |
| --- | --- | --- |
| `SLMP_HOST` | `127.0.0.1` | PLC or mock server host. |
| `SLMP_PORT` | `1025` | TCP/UDP port. |
| `SLMP_PLC_PROFILE` | required | Canonical profile such as `melsec:iq-r` or `melsec:iq-f`. |
| `SLMP_TRANSPORT` | `tcp` | `tcp` or `udp`. |
| `SLMP_TARGET` | unset | `SELF`, `SELF-CPU1`, or `NAME,NET,ST,IO,MD`. |
| `SLMP_NETWORK` / `SLMP_STATION` | unset | Other-station target, for example `SLMP_NETWORK=1 SLMP_STATION=2`. |
| `SLMP_MODULE_IO` / `SLMP_MULTIDROP` | `0x03FF` / `0x00` | Optional fields used with `SLMP_NETWORK` / `SLMP_STATION`. |
| `SLMP_TIMEOUT_MS` | `3000` | Socket timeout. |
| `SLMP_MONITORING_TIMER` | `16` | SLMP monitoring timer. |
| `SLMP_ENABLE_WRITES` | `0` | Set `1` to enable write examples. |

Raw read/write:

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_PROFILE=melsec:iq-r \
cargo run --example raw_read_write
```

Named helpers:

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PLC_PROFILE=melsec:iq-f \
SLMP_NAMED_ADDRESSES='D100:U,D200:F,D50.3,LTN10:D,LTS10:BIT' \
cargo run --example named_helpers
```

Advanced operations:

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PLC_PROFILE=melsec:iq-r \
SLMP_RANDOM_WORDS='D100,R10' \
SLMP_RANDOM_DWORDS='D200,LTN10' \
SLMP_EXT_DEVICE='J1\W10' \
cargo run --example advanced_operations
```

Device matrix compare:

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_PLC_PROFILE=melsec:iq-r \
cargo run --example device_matrix_compare
```

Cross-library verification lives in the separate `plc-comm-slmp-cross-verify`
repository:

```bash
cd ../plc-comm-slmp-cross-verify
python verify.py --clients rust
```

## Device range catalog

`read_device_range_catalog` reads live device range bounds after you connect. It requires an explicit profile through `SlmpConnectionOptions`; it does not auto-discover your intended profile.
The source rules for this catalog are maintained in the shared [SLMP device ranges](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/device-ranges/) reference.

```rust
use plc_comm_slmp::{SlmpClient, SlmpConnectionOptions, SlmpPlcProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let catalog = client.read_device_range_catalog().await?;

    if let Some(entry) = catalog.entries.iter().find(|entry| entry.supported) {
        println!("{:?}", entry);
    }
    client.close().await?;

    Ok(())
}
```

## Long device families

`LTN`, `LSTN`, `LCN`, and `LZ` are 32-bit families. Always use `:D` or `:L` suffixes in named addresses.

```rust
use plc_comm_slmp::{
    read_named, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let addresses = vec![
        "LTN10:D".to_string(),
        "LSTN20:L".to_string(),
        "LCN30:D".to_string(),
        "LZ0:D".to_string(),
    ];
    let snapshot = read_named(&client, &addresses).await?;
    println!("{:?}", snapshot);
    client.close().await?;

    Ok(())
}
```

> **Caution:** Plain word access to LTN/LSTN/LCN/LZ is rejected by the guarded low-level routes. Use helper APIs with `:D` or `:L`.

## Address reference table

| Form | Example | Meaning |
| --- | --- | --- |
| `:U` | `D100:U` | Unsigned 16-bit word. |
| `:S` | `D100:S` | Signed 16-bit word. |
| `:D` | `D100:D` | Unsigned 32-bit value. |
| `:L` | `D100:L` | Signed 32-bit value. |
| `:F` | `D100:F` | 32-bit float. |
| `:BIT` | `M100:BIT` | Boolean bit device value. |
| `.n` | `D50.3` | Bit `n` inside a word, where `n` is `0` through `F`. |

Named addresses used with `read_named`, `write_named`, and `poll_named` must include the intended type, for example `D100:U` or `M100:BIT`.
