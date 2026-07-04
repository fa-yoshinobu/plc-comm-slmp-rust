# Gotchas

Use this page as a short symptom index. For PLC response codes, use the shared
[SLMP Troubleshooting & End Codes](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/troubleshooting-end-codes/)
page. For profile limits and device availability, use the shared
[SLMP Profile Parameters](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/parameters/)
page.

## Connection fails or times out

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `SlmpClient::connect` cannot open the PLC connection. | Host, port, transport, PLC Ethernet setting, or network route is wrong. | Check the PLC setup first. Built-in Ethernet examples use TCP `192.168.250.100:1025`; use UDP only when the PLC port is configured for UDP. |

## Connection opens but every request returns an end code

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Simple reads such as `D100:U` connect but fail with an SLMP end code. | The selected `SlmpPlcProfile` does not match the PLC, or the PLC port data code does not match the library request format. | Select the canonical profile for the PLC and confirm the PLC Ethernet port is configured for binary SLMP. Use the shared end-code page for codes such as `C050`, `C059`, and `4031`. |

## Reads work but writes fail

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Reads work, but writes are rejected. | PLC-side write permission during RUN, remote password state, or profile write policy blocks the write. | Check the PLC setup guide and the selected profile's write policy. `S` is read-only except on iQ-F profiles. |

## Large requests fail with point-limit end codes

| Symptom | Root cause | Fix |
| --- | --- | --- |
| A large read, write, random request, or monitor request fails with `C051`, `C052`, `C053`, or `C054`. | The request exceeds the selected profile's per-request point limit. | Split the request or use the chunked helper. Check the shared profile parameter table for the limit. |

```rust
let words = read_words_chunked(&client, SlmpAddress::parse("D1000")?, 2000, 480).await?;
```

## Block commands are rejected on Q/L profiles

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `read_block()` or `write_block()` fails for `melsec:qcpu`, `melsec:qnu`, `melsec:qnudv`, or `melsec:lcpu`. | These profiles do not use block commands for normal high-level access. | Use normal direct/random read and write helpers. Disable strict profile only for deliberate compatibility investigation. |

## Mixed word and bit write fails

| Symptom | Root cause | Fix |
| --- | --- | --- |
| One write containing word values and bit values fails. | Some PLC paths reject mixed word and bit block writes. | Send word writes and bit writes as separate calls, or use an explicit split option where available. |

## iQ-F X/Y or DX/DY addresses fail

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `X`/`Y` points look shifted, or `DX`/`DY` is rejected on iQ-F. | iQ-F uses octal text for `X`/`Y`, and the iQ-F profile does not support `DX`/`DY`. | Parse string addresses with `SlmpPlcProfile::IqF`; use `X` and `Y` on iQ-F. |

```rust
let device = parse_device_for_plc_profile("X100", SlmpPlcProfile::IqF)?;
```

## Long timer/counter/index values look wrong

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `LTN`, `LSTN`, `LCN`, or `LZ` looks truncated or shifted. | These current-value families are 32-bit values. | Use `:D` or `:L` in named addresses, or a 32-bit dtype. |
| `LCS` or `LCC` behaves unlike a word value. | Long counter state devices are bits. | Read or write them as `BIT`. |

```rust
let addresses = vec!["LTN0:D".into(), "LSTN0:L".into(), "LCN0:D".into(), "LZ0:L".into(), "LCS0:BIT".into()];
let values = read_named(&client, &addresses).await?;
```

## G/HG fails as a normal address

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `G` or `HG` fails in high-level typed or named access. | Module buffer memory is not a standalone normal device route. | Use qualified routed forms such as `U3\G100` through the extended-device APIs. `HG` CPU-buffer access is profile-specific. |

## Missing or non-canonical profile is rejected

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Text configuration such as `iq-r` or an unknown value cannot be converted to a profile. | The crate requires exact canonical profiles and has no safe default profile. | Store canonical profile labels such as `melsec:iq-r`, or use the Rust enum selector directly. |

## Concurrent callers are serialized on one connection

| Symptom | Root cause | Fix |
| --- | --- | --- |
| Two async tasks sharing one PLC connection appear to run one request at a time. | A single SLMP connection is one ordered frame stream. | Share `SlmpClient` clones for safety, or open separate connections only when your PLC and network design allow it. |

```rust
let (left, right) = tokio::join!(
    read_typed(&client, SlmpAddress::parse("D100")?, "U"),
    read_typed(&client, SlmpAddress::parse("D101")?, "U"),
);
```
