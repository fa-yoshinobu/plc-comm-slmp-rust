# Gotchas

Use this page only for library-specific caveats.

Shared SLMP setup, profile, point-limit, and end-code symptoms live in the shared
[SLMP Troubleshooting & Codes](https://fa-yoshinobu.github.io/plc-comm-docs-site/plc-setup/slmp/troubleshooting-codes/)
page. For profile limits and device availability, use the shared
[SLMP Profile Parameters](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/parameters/)
page.

## Current library-specific caveats

| Area | Symptom | Guidance |
| --- | --- | --- |
| IPv6 endpoint | Connection options reject an IPv6 literal, or a hostname has no usable address. | TCP and UDP are IPv4-only. Use an IPv4 literal or a hostname with an IPv4 result; only IPv4 resolver results are attempted. |
| Request ordering | Two async tasks sharing one PLC connection appear to run one request at a time. | Clones share one FIFO queue and allow one wire transaction. Cancelling a waiting future sends nothing and removes its queue position. Separate clients are independent. |
| Cancelled request | A request future is dropped or an outer `tokio::time::timeout` expires, and the next call reports that the transport is closed. | This is intentional. A partial TCP frame or delayed UDP response cannot safely be reused, especially with 3E frames. Create and explicitly connect a new client; a state-changing command may already have reached the PLC, so do not automatically retry it. |
| Closed during a command | `close` completes while active and queued calls fail. | `close` immediately invalidates that client's connection generation. Active transmitted writes report `OutcomeUnknown`/`Closed`; queued calls report `Closed` without sending. |
| Bit-in-word RMW | A bit update can still race PLC logic or another connection. | `write_bit_in_word` reserves one FIFO turn on this client, but the PLC read and write are two non-atomic commands. Treat a post-send failure as an unknown write outcome. |
| Named collection | A required address cannot fit the named batch. | `read_named` and each `poll_named` cycle emit one random-read request or fail before transport. Use explicit operations—and an application snapshot/version design—when multiple commands are required. |

```rust
let (left, right) = tokio::join!(
    read_typed(&client, SlmpAddress::parse("D100", SlmpPlcProfile::IqR)?, "U"),
    read_typed(&client, SlmpAddress::parse("D101", SlmpPlcProfile::IqR)?, "U"),
);
```
