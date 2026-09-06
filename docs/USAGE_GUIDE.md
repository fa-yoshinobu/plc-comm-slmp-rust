# Usage guide

## Recommended entry points

| Entry point | Use it for |
| --- | --- |
| `SlmpConnectionOptions::new(host, port, transport, target, plc_profile)` | Creating a connection configuration with an explicit endpoint, route, and profile; only timeout, monitoring timer, and keepalive use approved defaults. |
| `SlmpClient::connect(options)` | Opening a TCP or UDP SLMP client. |
| `SlmpAddress::parse("D100", plc_profile)` | Parsing a profile-bound direct `DeviceAddress`; dtype, bit selector, and qualified route syntax are rejected. |
| `parse_named_address("D100:U")` / `normalize_named_address("D50.A", plc_profile)` | Parsing or normalizing a typed `AddressSpec`; the public result type is `NamedAddressParts`. |
| `read_latest_self_diagnosis_error_code` | Reading the latest PLC self-diagnosis error code from `SD0`. |
| `read_typed` and `write_typed` | Reading or writing one scalar value. |
| `read_named` and `write_named` | Reading or writing a small typed collection by address text. |
| `read_words_single_request` | Reading one contiguous word range in one request. |
| `read_bits_single_request` / `write_bits_single_request` | Reading or writing one contiguous direct-bit range in one request. |
| `read_dwords_single_request` | Reading one contiguous 32-bit range in one request. |
| `write_bit_in_word` | Updating one bit inside a word register. |
| `poll` | Repeating a named typed collection read on an interval. |
| `parse_qualified_device` | Parsing extended device text such as `U3\G100`, `U3E0\HG0`, and `J2\SW10`. |
| `read_words_extended` / `write_words_extended` | Reading or writing routed `U...` / `J...` word devices. |
| `read_bits_extended` / `write_bits_extended` | Reading or writing routed `U...` / `J...` bit devices. |

Keep the three address concepts separate: `SlmpAddress` and `parse_device`
handle only direct `DeviceAddress` values such as `D100` and `X10`;
`NamedAddressParts`, `parse_named_address`, and `normalize_named_address`
handle typed `AddressSpec` values such as `D100:U` and `D50.A`; and
`parse_qualified_device` handles routes such as `J1\X10` and `U0\G100`.

## Connection

`SlmpConnectionOptions` exposes user-settable connection fields. The PLC profile, frame type, and compatibility mode are selected together by `SlmpConnectionOptions::new`.

| Field | Default | Meaning |
| --- | --- | --- |
| `host` | value passed to `new` | PLC host name or IP address. |
| `port` | required argument | TCP or UDP destination port. |
| `timeout` | 3 seconds | Absolute deadline used separately for complete connection establishment and for each complete request. |
| `tcp_keepalive` | 30 seconds | TCP keepalive idle time, or `None`. |
| `target` | required argument | SLMP target address fields; pass `SlmpTargetAddress::default()` explicitly for the own station. |
| `transport_mode` | required argument | TCP or UDP. |
| `monitoring_timer` | `0x0010` | SLMP monitoring timer. |

`SlmpClient::connect` creates one monotonic connection deadline before IPv4
hostname resolution or socket work. TCP resolution, every selected IPv4
candidate, no-delay and keepalive configuration, and final adoption share that
single deadline. UDP resolution, bind, connect, and adoption use the same rule.
An IPv4 literal bypasses DNS. The library never tries IPv6 or gives each phase
or candidate a fresh timeout.

Connection-deadline expiry returns `SlmpErrorKind::Timeout` and exposes no
client. If all selected candidates fail before expiry, the error is
`SlmpErrorKind::Transport` with the connection cause. Dropping the connect
future cancels the public operation; a platform DNS call that cannot be stopped
may finish in its background task, but its result is discarded and cannot
create or mutate a client.

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
Always attempt re-lock after a confirmed unlock, even when the intervening
operation fails. If unlock or re-lock has an outcome-unknown result, do not
retry automatically; reopen, inspect the PLC lock state, and reconcile it
explicitly because access may remain unlocked.

```rust
let password_address =
    SlmpAddress::parse("D100", client.plc_profile().await)?;
client.remote_password_unlock("secret").await?;
let read_result = read_typed(&client, password_address, "U").await;
let lock_result = client.remote_password_lock("secret").await;
lock_result?;
let value = read_result?;
```

For `C200`-series password end codes, see the shared
[SLMP Troubleshooting & Codes](https://plc-comm-docs-site.fa-labo.com/plc-setup/slmp/troubleshooting-codes/)
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

Responses must return the same network, station, module I/O, and multidrop route.
The client discards complete foreign-route responses while waiting within the
original request deadline. A 4E response must also match the request serial.

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
the route is accepted. The write below is a controlled-test example: reserve
the exact configured module-buffer range and do not use a production control
buffer.

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
let module_readback_result = client.read_words_extended(module, 4).await;
let module_restore_result = client.write_words_extended(module, &module_words).await;
module_restore_result?;
let module_readback = module_readback_result?;

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

Restoration is attempted before a readback error is propagated. If the module
write itself returns an outcome-unknown error, do not automatically restore or
retry; reopen the session, inspect the configured buffer, and choose recovery
explicitly. If restoration fails, inspect the configured buffer and reconcile
it manually before continuing. `module_readback`, `cpu_buffer_words`, `link_words`, `bits`, and
`indexed_words` are the values available to the surrounding application.

For iQ-R multi-CPU `U3En\HG...` access, the qualified device never changes the
SLMP request target automatically. Select the target CPU in
`SlmpConnectionOptions` when a write must be reflected there; use a separate
client for a different target. A write can return a normal end code without
changing the intended CPU buffer when the selected request target identifies a
different CPU or Own Station. Cross-CPU reads remain valid. See the shared
[iQ-R target guidance](https://plc-comm-docs-site.fa-labo.com/plc-setup/slmp/iq-r/#multi-cpu-cpu-buffer-target).

## Monitor, self-test, and Clear Error

Registration and every monitor cycle are separate one-request operations.
`run_monitor_cycle` requires the registered Word and DWord counts every time;
it does not auto-register, retry, or infer them. Calling it before monitor
registration still sends one cycle request, so the PLC determines the returned
error. The combined expected count must be nonzero and cannot exceed the
selected profile's monitor-registration limit.

Monitor registration changes the PLC's active monitor set. Use only a known
device list on a controlled PLC. If registration has an outcome-unknown result,
do not register again automatically; reconnect and reconcile the monitor state
explicitly.

```rust
use plc_comm_slmp::parse_device;

let word = parse_device("D120", SlmpPlcProfile::IqR)?;
let dword = parse_device("D200", SlmpPlcProfile::IqR)?;
client.register_monitor_devices(&[word], &[dword]).await?;
let cycle = client.run_monitor_cycle(1, 1).await?;

let echo = client.self_test_loopback(b"A1B2C3D4").await?;
```

Self-test accepts only 1–960 ASCII `0-9/A-F` bytes and requires exact declared
length, actual length, and echo equality.

Clear Error is a separate state-changing maintenance action. It clears the
PLC's current error state and cannot recreate the diagnostics that existed
before the command. Capture the required diagnostics first and invoke it only
on a controlled PLC when clearing the error is explicitly intended:

```rust
client.clear_error().await?;
```

`clear_error` has no wire-level arguments and always sends the fixed
empty-payload command.

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
Read `end_code` for the PLC response code and `error_info` when the PLC returned the structured error-information block. The library accepts that PLC error as definitive only when the embedded route, command, and subcommand match the active request. A mismatch is malformed, retires the transport, and makes a transmitted state change outcome-unknown. `error_info.raw` contains the required nine-byte prefix and `error_info.extra` retains every following byte.
Each request gets one monotonic deadline immediately before its first transport send. Queue
waiting does not consume that deadline. The same deadline covers the complete send, TCP/UDP
response assembly, response correlation, protocol parsing, and command-specific payload decode.
Request-exchange deadline expiry on a read returns `SlmpErrorKind::Timeout`; callers may also use
`SlmpError::is_timeout()` instead of matching error-message text. Timeout invalidates the
in-flight transport, so explicitly connect a new client before another request.

`SlmpErrorKind` distinguishes validation (`General`), `Timeout`, `Cancelled`, `Closed`,
`NotConnected`, `Transport`, `MalformedResponse`, `PlcEndCode`, `ProfileFeature`, and
`OutcomeUnknown`. If transmission of a state-changing command may have started, timeout,
cancellation reported by an adapter, close, transport failure, or malformed acknowledgement returns `OutcomeUnknown`;
inspect `outcome_unknown_reason` for the structured cause. Do not automatically resend that
command. A nonzero PLC end code remains the definitive `PlcEndCode` result and does not by itself
retire a fully correlated connection.

Once the complete response has been correlated, protocol-checked, and decoded, that definitive
success or PLC end code wins over a concurrent `close` or a deadline observed only after decode.
The transport is still retired when either local condition occurred. A close that arrives before
definitive decode still returns `Closed` for a read or `OutcomeUnknown`/`Closed` for a transmitted
state-changing command; an unfinished decode may likewise retain timeout classification.

Dropping a Rust future (including an outer `tokio::time::timeout`) cannot deliver a library
`Result`; it retires an in-flight transport. If the dropped command was state-changing, the caller
must treat its outcome as unknown and must not resend it automatically.

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
            println!("additional error bytes={:02X?}", info.extra);
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
    let readback_result = read_typed(&client, device, "U").await;
    let restore_result = write_typed(&client, device, "U", &original).await;
    restore_result?;
    let value = readback_result?;
    println!("{:?}", value);
    client.close().await?;

    Ok(())
}
```

Use only a register reserved for controlled testing. The example attempts
restoration before propagating a readback error. An outcome-unknown test write
requires explicit state inspection rather than automatic restore or retry. If
restoration fails, inspect the register and reconcile its value manually before
continuing.

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

`write_bit_in_word` performs one read followed by one write while holding one FIFO client turn, so
another operation on the same client cannot interleave between those two frames. It is still a
non-atomic PLC read-modify-write: another PLC client or PLC logic can change the word, and a
post-send interruption of its write reports `OutcomeUnknown`. FIFO wait is outside the timeout;
one absolute deadline covers both requests after admission, and a successful read always proceeds
to the write even when the selected bit is unchanged. Use `write_bit_in_word_extended` with a
`SlmpQualifiedDeviceAddress` for an immutable U-qualified module-buffer or J-qualified link-direct
route. Unsupported profile/route combinations fail before the read. Do not retry automatically.

## Shared-client ordering and close

Clones of one `SlmpClient` share one FIFO admission queue and allow at most one wire transaction at
a time. Arguments are fixed for the queued call; cancelling a waiting Rust future removes that
waiter without sending and without delaying later waiters. Calling `close` invalidates the exact
client connection generation immediately. An active incomplete read returns `Closed`, an active
incomplete transmitted state-changing command returns `OutcomeUnknown` with reason `Closed`, and
queued calls return `Closed` without sending. A fully correlated and decoded response remains the
definitive result even if `close` wins the later transport-state race. Separate `SlmpClient`
instances have independent queues and can progress concurrently.

## Profile request limits

Batch planners can read the selected profile's canonical request limit without
opening a connection or communicating with a PLC:

```rust
use plc_comm_slmp::{SlmpPlcProfile, SlmpProfileLimitKey};

if let Some(limit) =
    SlmpPlcProfile::QnUDV.profile_limit(SlmpProfileLimitKey::RandomReadWord)
{
    println!("{}", limit.max_points); // 192
}
```

`weighted_max_points` is `Some` only for commands, such as random Word/DWord
writes, that enforce an encoded-entry weight in addition to the point count.
The lookup returns `None` when no canonical profile/key value exists and does
not invent a family fallback.

## Single-request range reads

The library rejects ranges above the protocol/profile limit. It does not split
one logical read into multiple requests because the first and last values would
then be sampled at different times. If multiple requests are intentional, build
that sequence in the application and handle the temporal split explicitly.

Every contiguous request must also fit the address field selected by the wire
format: Q/L-compatible and link-direct layouts use 24 bits, while iQ-R layouts
use 32 bits. A J-qualified link-direct request always uses the 24-bit Q/L layout,
even when the client profile is iQ-R; other iQ-R Extended Device layouts use
32 bits. Admission uses the complete consumed span, not the configured PLC
device-range catalog. Word-unit access to a word device consumes one device per
word; word-unit packed access to a bit device consumes 16 bit devices per word;
ordinary DWord/Float32 access consumes two word devices per value (32 bit-device
numbers per value when packed through a bit family); and one bit-block point
consumes 16 bit devices. The long-timer Direct status block is the explicit
exception: four returned words consume one `LTN`/`LSTN` device. Random and
monitor DWord entries use the same route-specific logical widths. A span that
crosses the wire maximum is rejected before request counters or transport; the
library does not substitute profile usable-range policy for this
wire-representability check.

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
The helper requires a word device and preflights writability before sending its
read request, so an invalid or non-writable target sends neither half of the
read-modify-write sequence.

```rust
use plc_comm_slmp::{
    read_named, read_typed, write_bit_in_word, write_bit_in_word_extended, write_typed, SlmpAddress, SlmpClient,
    SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let word = SlmpAddress::parse("D50", SlmpPlcProfile::IqR)?;
    let original = read_typed(&client, word, "U").await?;
    write_bit_in_word(&client, word, 3, true).await?;

    // A qualified route uses the explicit sibling helper:
    // let qualified = plc_comm_slmp::parse_qualified_device(r"J1\W10", SlmpPlcProfile::IqR)?;
    // write_bit_in_word_extended(&client, qualified, 3, true).await?;

    let addresses = vec!["D50.3".to_string()];
    let snapshot_result = read_named(&client, &addresses).await;
    let restore_result = write_typed(&client, word, "U", &original).await;
    restore_result?;
    let snapshot = snapshot_result?;
    println!("{:?}", snapshot);
    client.close().await?;

    Ok(())
}
```

Use only a word reserved for controlled testing. Restoration is attempted
before a snapshot-read error is propagated. If the bit write has an unknown
outcome, inspect and reconcile the whole word instead of retrying automatically.
If restoration fails, inspect the whole word and reconcile it manually before
continuing.

## Polling

```rust
use std::time::Duration;

use futures_util::StreamExt;
use plc_comm_slmp::{poll, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let addresses = vec!["D100:U".to_string(), "M100:BIT".to_string(), "D50.3".to_string()];
    let mut stream = Box::pin(poll(&client, &addresses, Duration::from_millis(500)));

    if let Some(snapshot) = stream.next().await.transpose()? {
        println!("{:?}", snapshot);
    }
    client.close().await?;

    Ok(())
}
```

The address plan, compact result indexes, and immutable Random Read payload are
validated and prepared once when the polling stream is created. Each cycle
reuses that prepared request while retaining the ordinary client FIFO,
deadline, cancellation, close, and error contracts.

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
| `SLMP_TIMEOUT_MS` | optional; `3000` | Absolute request-exchange deadline in milliseconds. |
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
SLMP_NAMED_ADDRESSES='D100:U,D200:F,D50.3,M100:BIT' \
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

Every semantic `SlmpDeviceAddress` or qualified address is bound to the exact canonical profile used to create it. Passing it to a client configured for any other profile is rejected before request construction or transport activity, including when a unit-specific profile shares a base family with the client. Parse the address again with the destination client's profile instead of reusing it across profiles.

`read_device_range_catalog` reads the canonical profile's required SD-register window after you connect. It requires an explicit profile through `SlmpConnectionOptions` and does not auto-discover your intended profile. For QCPU, LCPU, QnU, and QnUDV address profiles, it then performs the canonical runtime probes: QCPU checks `Z15`, and all four families find the readable `ZR` boundary before deriving `R`. Any nonzero PLC end code returned by a probe means only that candidate address is unreadable; timeout, transport, protocol, lifecycle, and local-validation failures are returned to the caller without a partial catalog. Rust future cancellation drops the operation instead of producing a library error.
The source rules for this catalog are maintained in the shared [SLMP device ranges](https://plc-comm-docs-site.fa-labo.com/slmp/profile-reference/device-ranges/) reference.

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

`LTN`, `LSTN`, `LCN`, and `LZ` are 32-bit families. Use `read_typed` with `D` or `L` for one scalar
value. `LTN` and `LSTN` use the Direct long-timer route and therefore cannot be included in
`read_named` or `poll`, whose contract is exactly one Random Read request.

```rust
use plc_comm_slmp::{
    read_typed, SlmpAddress, SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = SlmpConnectionOptions::new("192.168.250.100", 1025, plc_comm_slmp::SlmpTransportMode::Tcp, plc_comm_slmp::SlmpTargetAddress::default(), SlmpPlcProfile::IqR)?;

    let client = SlmpClient::connect(options).await?;
    let timer = read_typed(
        &client,
        SlmpAddress::parse("LTN10", SlmpPlcProfile::IqR)?,
        "D",
    ).await?;
    let counter = read_typed(
        &client,
        SlmpAddress::parse("LCN30", SlmpPlcProfile::IqR)?,
        "D",
    ).await?;
    println!("timer={timer:?}, counter={counter:?}");
    client.close().await?;

    Ok(())
}
```

> **Caution:** Plain word access to LTN/LSTN/LCN/LZ is rejected by guarded semantic routes. Use
> `read_typed`/`write_typed` with `D` or `L`, or an explicit long-device helper. Low-level explicit
> packed-word APIs remain separate wire-level operations.

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

Named addresses used with `read_named`, `write_named`, and `poll` must include the intended
type, for example `D100:U` or `M100:BIT`. `BIT` is valid only for bit devices; numeric/string dtypes
are valid only for word devices. A word-device bit is written explicitly with `.n` or
`write_bit_in_word`; no helper performs an implicit mask, read-modify-write, or fallback.
## Request payload limits

One SLMP request can carry at most 65,529 command-payload bytes over TCP. UDP must also fit one
complete datagram, so the command-payload maximum is 65,492 bytes for 3E and 65,488 bytes for 4E.
Array and random label requests use even-sized payloads and therefore have a largest
protocol-representable payload of 65,528 bytes before the lower UDP limit is applied.

Oversized requests return `SlmpError` before send, traffic counters, request-frame publication, or
4E serial allocation. They are never truncated or split automatically; applications that issue
several requests must define ordering, partial-success, and write-atomicity behavior.

## Traffic statistics

`client.traffic_stats().await` returns the client-lifetime `request_count`, `tx_bytes`, and
`rx_bytes` snapshot. A request counts only after its complete frame is sent; a complete received
frame counts before protocol or PLC end-code validation. Close and reconnect do not reset it.
