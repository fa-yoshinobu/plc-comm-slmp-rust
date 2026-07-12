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
| Request ordering | Two async tasks sharing one PLC connection appear to run one request at a time. | A single SLMP connection is one ordered frame stream. Share `SlmpClient` clones for safety, or open separate connections only when your PLC and network design allow it. |
| Cancelled request | A request future is dropped or an outer `tokio::time::timeout` expires, and the next call reports that the transport is closed. | This is intentional. A partial TCP frame or delayed UDP response cannot safely be reused, especially with 3E frames. Create and explicitly connect a new client; the cancelled command may already have reached the PLC, so do not automatically retry writes. |
| Named collection | A required address cannot fit the named batch. | `read_named` and each `poll_named` cycle emit one random-read request or fail before transport. Use explicit operations—and an application snapshot/version design—when multiple commands are required. |

```rust
let (left, right) = tokio::join!(
    read_typed(&client, SlmpAddress::parse("D100", SlmpPlcProfile::IqR)?, "U"),
    read_typed(&client, SlmpAddress::parse("D101", SlmpPlcProfile::IqR)?, "U"),
);
```
