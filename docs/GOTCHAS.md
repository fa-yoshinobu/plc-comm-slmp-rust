# Gotchas

Each entry starts with the symptom, then gives the root cause and a complete Rust fix. The examples use TCP `192.168.250.100:1025` and `SlmpPlcProfile::IqR` unless the profile itself is the point.

## LTN/LSTN/LCN/LZ reads return wrong values

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `LTN0`, `LSTN0`, `LCN0`, or `LZ0` looks truncated or is rejected. | These current-value families are 32-bit values, not normal 16-bit word values. | Use named addresses with `:D` or `:L`, or call `read_typed` with a 32-bit dtype. |

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
        "LTN0:D".to_string(),
        "LSTN0:D".to_string(),
        "LCN0:L".to_string(),
        "LZ0:D".to_string(),
    ];
    let values = read_named(&client, &addresses).await?;
    println!("{values:?}");
    client.close().await?;

    Ok(())
}
```

## LCS/LCC reads look incorrect

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `LCS0` or `LCC0` does not behave like a normal word value. | Long counter state devices are state bits. Reads use direct bit access, and writes route through random bit write (`0x1402`). | Use `read_named` or `write_typed` with `SlmpValue::Bool`. |

```rust
use plc_comm_slmp::{
    read_named, write_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
    SlmpValue,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let addresses = vec!["LCS0:BIT".to_string(), "LCC0:BIT".to_string()];
    let state = read_named(&client, &addresses).await?;
    write_typed(&client, SlmpAddress::parse("LCC0")?, "BIT", &SlmpValue::Bool(true)).await?;
    println!("{state:?}");
    client.close().await?;

    Ok(())
}
```

## LTS/LTC/LSTS/LSTC write rejected

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Direct bit writes to long timer state devices are rejected. | These families need the helper route that selects supported random bit write behavior. | Use `write_named` or `write_typed` for the state device. |

```rust
use plc_comm_slmp::{
    write_named, NamedAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpValue,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let mut updates = NamedAddress::new();
    updates.insert("LTS0:BIT".to_string(), SlmpValue::Bool(true));
    updates.insert("LTC0:BIT".to_string(), SlmpValue::Bool(false));
    write_named(&client, &updates).await?;
    client.close().await?;

    Ok(())
}
```

## G/HG fails

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `G100` or `HG1000` fails through the high-level helpers. | Module buffer access is outside the public typed helper surface. | Use extended-device methods with qualified addresses such as `U3\G100` or `U3E0\HG0`. |

```rust
use plc_comm_slmp::{
    parse_qualified_device, SlmpClient, SlmpConnectionOptions, SlmpExtensionSpec, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let device = parse_qualified_device("U3\\G100")?;
    let values = client.read_words_extended(device, 4, SlmpExtensionSpec::default()).await?;
    println!("{values:?}");
    client.close().await?;

    Ok(())
}
```

## Mixed word and bit write fails

| Symptom | Root cause | Fix |
| --- | --- | --- |
| A block write that combines word blocks and bit blocks returns a PLC-side error. | Some PLC paths reject command `0x1406` for mixed word and bit payloads. | Split word and bit writes, or use `SlmpBlockWriteOptions { split_mixed_blocks: true }` intentionally. |

```rust
use plc_comm_slmp::{
    SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile, SlmpValue, write_typed,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    write_typed(&client, SlmpAddress::parse("D100")?, "U", &SlmpValue::U16(42)).await?;
    write_typed(&client, SlmpAddress::parse("M100")?, "BIT", &SlmpValue::Bool(true)).await?;
    client.close().await?;

    Ok(())
}
```

## DX/DY fails on SlmpPlcProfile::IqF

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `DX` or `DY` is rejected with `SlmpPlcProfile::IqF`. | iQ-F does not support `DX` and `DY` in this profile. | Use `X` and `Y`; iQ-F string notation is octal. |

```rust
use plc_comm_slmp::{
    parse_device_for_plc_profile, read_typed, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqF);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let x100 = parse_device_for_plc_profile("X100", SlmpPlcProfile::IqF)?;
    let value = read_typed(&client, x100, "BIT").await?;
    println!("{value:?}");
    client.close().await?;

    Ok(())
}
```

## All reads return an end code

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Simple reads such as `D100` connect but return an SLMP end code. | `SlmpPlcProfile` selects the frame type, compatibility mode, and address parsing rules. The wrong profile can make every request invalid for your PLC. | Choose the concrete profile from [PROFILES.md](PROFILES.md) in your configuration or UI. |

The crate reports the raw numeric `end_code` and a deterministic resource key
such as `slmp_end_code_c810`. Localized end-code message text is not embedded in
the public communication crate; resolve the key in an application-owned catalog
when user-facing text is required.

```rust
use plc_comm_slmp::{
    read_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    match read_typed(&client, SlmpAddress::parse("D100")?, "U").await {
        Ok(value) => println!("{value:?}"),
        Err(error) if error.end_code.is_some() => {
            println!("PLC rejected the request: {:?}", error.end_code);
        }
        Err(error) => return Err(error.into()),
    }
    client.close().await?;

    Ok(())
}
```

## Missing or non-canonical profile is rejected

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Text configuration such as `iq-r` or `MELSEC:IQ-R` cannot be converted to a profile. | `SlmpPlcProfile::parse_label` accepts only exact canonical profile values, and `SlmpConnectionOptions::new` requires a concrete Rust selector. There is no `Unspecified` fallback in this crate. | Store only canonical profiles such as `melsec:iq-r`, or let your UI store the selector directly. |

```rust
use plc_comm_slmp::{SlmpConnectionOptions, SlmpPlcProfile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profile = SlmpPlcProfile::parse_label("melsec:iq-r").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "unsupported PLC profile")
    })?;
    let mut options = SlmpConnectionOptions::new("192.168.250.100", profile);
    options.port = 1025;
    println!("{}", options.plc_profile().canonical_name());

    Ok(())
}
```

## Concurrent callers are serialized on one connection

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Two async tasks sharing one PLC connection appear to run one request at a time. | A single SLMP connection is one ordered frame stream. The public `SlmpClient` is cloneable and internally serialized to keep responses matched to requests. | Share `SlmpClient` clones for safety, or open separate connections only when your PLC and network design allow it. |

```rust
use plc_comm_slmp::{
    read_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqR);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let left_address = SlmpAddress::parse("D100")?;
    let right_address = SlmpAddress::parse("D101")?;
    let (left, right) = tokio::join!(
        read_typed(&client, left_address, "U"),
        read_typed(&client, right_address, "U"),
    );
    println!("{:?} {:?}", left?, right?);
    client.close().await?;

    Ok(())
}
```

## X/Y addresses look shifted on iQ-F

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `X100` or `Y100` points to a different I/O address than expected after a profile change. | iQ-F uses octal `X`/`Y` text; non-iQ-F profiles use hexadecimal text. | Parse string addresses with the same `SlmpPlcProfile` you use for the connection. |

```rust
use plc_comm_slmp::{parse_device_for_plc_profile, SlmpPlcProfile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iqf_x = parse_device_for_plc_profile("X100", SlmpPlcProfile::IqF)?;
    let iqr_x = parse_device_for_plc_profile("X20", SlmpPlcProfile::IqR)?;
    println!("{iqf_x:?} {iqr_x:?}");

    Ok(())
}
```

## S appears in a catalog but parsing fails

| Symptom | Root cause | Fix |
| --- | --- | --- |
| The live range catalog reports `S`, but `S` cannot be parsed as a public device address. | The crate intentionally does not expose the MELSEC step-relay device in the parser/client surface. | Keep `S` out of application address lists until support is added deliberately. |

```rust
use plc_comm_slmp::SlmpAddress;

fn main() {
    assert!(SlmpAddress::try_parse("S0").is_none());
}
```
