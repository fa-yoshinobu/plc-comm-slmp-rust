# Usage guide

## Recommended entry points

| Entry point | Use it for |
| --- | --- |
| `SlmpConnectionOptions::new(host, port, transport, target, plc_profile)` | Creating a connection configuration with an explicit endpoint, route, and profile; only timeout, monitoring timer, and keepalive use approved defaults. |
| `SlmpClient::connect(options)` | Opening a TCP or UDP SLMP client. |
| `SlmpAddress::parse("D100", plc_profile)` | Parsing a profile-bound device address. |
| `read_latest_self_diagnosis_error_code` | Reading the latest PLC self-diagnosis error code from `SD0`. |
| `read_typed` and `write_typed` | Reading or writing one scalar value. |
| `read_named` and `write_named` | Reading or writing a small typed collection by address text. |
| `read_words_single_request` | Reading one contiguous word range in one request. |
| `read_dwords_single_request` | Reading one contiguous 32-bit range in one request. |
| `write_bit_in_word` | Updating one bit inside a word register. |
| `poll_named` | Repeating a named typed collection read on an interval. |
| `parse_qualified_device` | Parsing extended device text such as `U3\G100`, `U3E0\HG0`, and `J2\SW10`. |
| `read_words_extended` / `write_words_extended` | Reading or writing routed `U...` / `J...` word devices. |
| `read_bits_extended` / `write_bits_extended` | Reading or writing routed `U...` / `J...` bit devices. |

## Connection

`SlmpConnectionOptions` exposes user-settable connection fields. The PLC profile, frame type, and compatibility mode are selected together by `SlmpConnectionOptions::new`.

| Field | Default | Meaning |
| --- | --- | --- |
| `host` | value passed to `new` | PLC host name or IP address. |
| `port` | required argument | TCP or UDP destination port. |
| `timeout` | 3 seconds | Socket read/write timeout. |
| `tcp_keepalive` | 30 seconds | TCP keepalive idle time, or `None`. |
| `target` | required argument | SLMP target address fields; pass `SlmpTargetAddress::default()` explicitly for the own station. |
| `transport_mode` | required argument | TCP or UDP. |
| `monitoring_timer` | `0x0010` | SLMP monitoring timer. |

```rust
use std::time::Duration;

use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpTransportMode,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;
    options.timeout = Duration::from_secs(3);

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
let value = read_typed(&client, SlmpAddress::parse("D100", client.plc_profile().await)?, "U").await?;
client.remote_password_lock("secret").await?;
```

For `C200`-series password end codes, see the shared
[SLMP Troubleshooting & Codes](https://fa-yoshinobu.github.io/plc-comm-docs-site/plc-setup/slmp/troubleshooting-codes/)
page.

## Remote CPU control

`remote_run` and `remote_pause` require `SlmpRemoteMode::Normal` or
`SlmpRemoteMode::Force`. `remote_run` also requires one explicit
`SlmpRemoteClearMode`; no magic numeric fallback is used. `remote_reset()` sends
the fixed RESET request and returns after the send completes because a successful
RESET does not return the normal response. The client then closes its transport so
a delayed RESET response cannot be consumed by another 3E request. Create a new
client connection and confirm the PLC state when proof of the reset is required.

## Routing / target station

Every connection explicitly selects its complete target route. For a directly
connected own station, pass `SlmpTargetAddress::default()` deliberately. Use
other values when the PLC network is configured for another station, multi-CPU
module I/O, or multidrop access.

`SlmpTargetAddress` controls the SLMP destination header. It is not a device
family selector; routed devices such as `Un\Gn` and `Jn\...` still need their
own address syntax.

```rust
use plc_comm_slmp::{SlmpConnectionOptions, SlmpModuleIo, SlmpPlcProfile, SlmpTargetAddress};

let target = SlmpTargetAddress {
    network: 0x01,
    station: 0x02,
    module_io: SlmpModuleIo::OWN_STATION,
    multidrop: 0x00,
};
let options = SlmpConnectionOptions::new(
    "192.168.250.100",
    1025,
    plc_comm_slmp::SlmpTransportMode::Tcp,
    target,
    SlmpPlcProfile::IqR,
)?;
```

Do not omit the target. `SlmpTargetAddress::default()` is only a convenient way
to spell the explicit own-station route at the call site.

## Extended device access

`G`, `HG`, and `J` devices are not normal standalone addresses. Use the
extended device APIs with a qualified address:

| Address form | Meaning |
| --- | --- |
| `U3\G100` | Module access buffer memory `G100` on unit `U3`. |
| `U3E0\HG0` | CPU buffer memory `HG0` on `U3E0`, when the selected profile supports it. |
| `J2\SW10` | Link direct `SW10` on J network `2`. |
| `J1\X10` | Link direct `X10` on J network `1`. |

The selected PLC profile and the actual PLC configuration still decide whether
the route is accepted.

```rust
use plc_comm_slmp::{
    parse_qualified_device, SlmpClient, SlmpConnectionOptions, SlmpDeviceModification,
    SlmpPlcProfile,
};

let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

let client = SlmpClient::connect(options).await?;

let module = parse_qualified_device(r"U3\G100", SlmpPlcProfile::IqR)?;
let module_words = client.read_words_extended(module, 4).await?;
client.write_words_extended(module, &[1, 2, 3, 4]).await?;

let cpu_buffer = parse_qualified_device(r"U3E0\HG0", SlmpPlcProfile::IqR)?;
let cpu_buffer_words = client.read_words_extended(cpu_buffer, 2).await?;

let link_word = parse_qualified_device(r"J2\SW10", SlmpPlcProfile::IqR)?;
let link_words = client.read_words_extended(link_word, 1).await?;

let link_bits = parse_qualified_device(r"J1\X10", SlmpPlcProfile::IqR)?;
let bits = client.read_bits_extended(link_bits, 16).await?;

let indexed = parse_qualified_device(r"U3\D100", SlmpPlcProfile::IqR)?
    .with_modification(SlmpDeviceModification::IndexZ(4))?;
let indexed_words = client.read_words_extended(indexed, 1).await?;
client.close().await?;
```

## PLC diagnostics

`SlmpClient::read_latest_self_diagnosis_error_code` reads `SD0`, the latest PLC self-diagnosis error code, and returns the raw 16-bit value. Format it as hexadecimal when displaying it.

This value is separate from `SlmpError.end_code`. `SlmpError.end_code` is the SLMP response end code for a communication request, while `SD0` is the PLC CPU's self-diagnosis error register.

```rust
use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

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
match read_typed(
    &client,
    SlmpAddress::parse("D100", client.plc_profile().await)?,
    "U",
).await {
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
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let value = read_typed(&client, SlmpAddress::parse("D100", SlmpPlcProfile::IqR)?, "U").await?;
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

`write_typed` requires the exact matching `SlmpValue` variant shown above. It
does not coerce Boolean, floating-point, signed, unsigned, or differently sized
values. `F32` values must also be finite. Text accepted by CLI helpers is range
checked before it is converted into a `SlmpValue`.

## Write a single value

```rust
use plc_comm_slmp::{
    read_typed, write_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
    SlmpValue,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let device = SlmpAddress::parse("D600", SlmpPlcProfile::IqR)?;

    let original = read_typed(&client, device, "U").await?;
    write_typed(&client, device, "U", &SlmpValue::U16(42)).await?;
    let value = read_typed(&client, device, "U").await?;
    println!("{:?}", value);
    write_typed(&client, device, "U", &original).await?;
    client.close().await?;

    Ok(())
}
```

## Named typed collection

```rust
use plc_comm_slmp::{
    read_named, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let addresses = vec![
        "D100:U".to_string(),
        "D200:F".to_string(),
        "D50.3".to_string(),
        "M100:BIT".to_string(),
    ];
    let values = read_named(&client, &addresses).await?;
    println!("{:?}", values);
    client.close().await?;

    Ok(())
}
```

`read_named` sends exactly one random-read request. An oversized batch or an
entry that requires a direct, block, or long-timer helper route is rejected
before transport. Use the explicit typed or long-device API when another
command family is required.

`write_named` sends exactly one random-write request. Word and DWord entries may
share that request; bit entries form a separate request family. Mixing bit and
word/DWord entries is rejected before transport, so the application must perform
and account for the separate operations explicitly. Bit-in-word entries are also
rejected here because they require a read-modify-write sequence; call
`write_bit_in_word` explicitly when that non-atomic sequence is intended.

## Single-request range reads

The library rejects ranges above the protocol/profile limit. It does not split
one logical read into multiple requests because the first and last values would
then be sampled at different times. If multiple requests are intentional, build
that sequence in the application and handle the temporal split explicitly.

```rust
use plc_comm_slmp::{
    read_words_single_request, SlmpAddress, SlmpClient, SlmpConnectionOptions,
    SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let start = SlmpAddress::parse("D100", SlmpPlcProfile::IqR)?;

    let single = read_words_single_request(&client, start, 8).await?;
    println!("{:?}", single);
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
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let word = SlmpAddress::parse("D50", SlmpPlcProfile::IqR)?;
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
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

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

## Operational recipes

The `multi_plc_monitor` example is a read-only multi-PLC monitor. It polls each
PLC in an independent reconnect loop, logs `connected`, `lost`, `reconnecting`,
and `recovered` transitions, and uses the same 1 second to 30 second backoff as
the reconnect polling sample.

The `config_polling` example is read-only JSON-driven polling. Its CSV output
uses the shared long form `timestamp,plc,tag,value`. YAML config is only
available in the Python recipes; the Rust recipe intentionally accepts JSON.

## Runnable examples

The repository examples are designed to run from environment variables.

| Variable | Requirement | Meaning |
| --- | --- | --- |
| `SLMP_HOST` | optional; example host only | PLC or mock server host. |
| `SLMP_PORT` | required | TCP/UDP destination port; zero is rejected. |
| `SLMP_PLC_PROFILE` | required | Canonical profile such as `melsec:iq-r` or `melsec:iq-f`. |
| `SLMP_TRANSPORT` | required | Exactly `tcp` or `udp`. |
| `SLMP_TARGET` | required unless all four route fields are present | `SELF`, `SELF-MULTIPLE-CPU-1`, or `NAME,NET,ST,IO,MD`. |
| `SLMP_NETWORK` / `SLMP_STATION` | complete-set alternative | Both are required together with module I/O and multidrop. |
| `SLMP_MODULE_IO` / `SLMP_MULTIDROP` | complete-set alternative | Both are required together with network and station. |
| `SLMP_TIMEOUT_MS` | optional; `3000` | Socket timeout in milliseconds. |
| `SLMP_MONITORING_TIMER` | optional; `16` | SLMP monitoring timer in 250 ms units (4 seconds). |
| `SLMP_ENABLE_WRITES` | optional; `0` | Set `1` to enable write examples. |

Raw read/write:

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_TRANSPORT=tcp \
SLMP_TARGET=SELF \
SLMP_PLC_PROFILE=melsec:iq-r \
cargo run --example raw_read_write
```

Named helpers:

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_TRANSPORT=tcp \
SLMP_TARGET=SELF \
SLMP_PLC_PROFILE=melsec:iq-f \
SLMP_NAMED_ADDRESSES='D100:U,D200:F,D50.3,LTN10:D,LTS10:BIT' \
cargo run --example named_helpers
```

Advanced operations:

```bash
cd plc-comm-slmp-rust
SLMP_HOST=192.168.250.100 \
SLMP_PORT=1025 \
SLMP_TRANSPORT=tcp \
SLMP_TARGET=SELF \
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
SLMP_TRANSPORT=tcp \
SLMP_TARGET=SELF \
SLMP_PLC_PROFILE=melsec:iq-r \
cargo run --example device_matrix_compare
```

## Device range catalog

`read_device_range_catalog` reads live device range bounds after you connect. It requires an explicit profile through `SlmpConnectionOptions`; it does not auto-discover your intended profile.
The source rules for this catalog are maintained in the shared [SLMP device ranges](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/device-ranges/) reference.

```rust
use plc_comm_slmp::{SlmpClient, SlmpConnectionOptions, SlmpPlcProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

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
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let addresses = vec![
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
