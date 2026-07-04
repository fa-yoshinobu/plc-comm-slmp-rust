# Gotchas

Use this page as a short symptom index. For PLC response codes, use the shared
[SLMP Troubleshooting & End Codes](https://fa-yoshinobu.github.io/plc-comm-docs-site/plc-setup/slmp/troubleshooting-end-codes/)
page. For profile limits and device availability, use the shared
[SLMP Profile Parameters](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/parameters/)
page.
For PLC-side Ethernet settings, use the shared
[MELSEC SLMP PLC Setup Guide](https://fa-yoshinobu.github.io/plc-comm-docs-site/plc-setup/slmp/).
Check Binary communication data code, port/open settings, and RUN-time write permission there before debugging application code.

## Connection fails or times out

| Symptom | Root cause | Fix |
| --- | --- | --- |
| `SlmpClient::connect` cannot open the PLC connection. | Host, port, transport, PLC Ethernet setting, or network route is wrong. | Check the PLC setup guide first. Built-in Ethernet examples use TCP `192.168.250.100:1025`; use UDP only when the PLC port is configured for UDP. |

## Large requests fail with point-limit end codes

| Symptom | Root cause | Fix |
| --- | --- | --- |
| A large read, write, random request, or monitor request fails with `C051`, `C052`, `C053`, or `C054`. | The request exceeds the selected profile's per-request point limit. | Split the request or use the chunked helper. Check the shared profile parameter table for the limit. |

```rust
let words = read_words_chunked(&client, SlmpAddress::parse("D1000")?, 2000, 480).await?;
```

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
