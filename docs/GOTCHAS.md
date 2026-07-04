# Gotchas

Use this page only for library-specific caveats.

Shared SLMP setup, profile, point-limit, and end-code symptoms live in the shared
[SLMP Troubleshooting & End Codes](https://fa-yoshinobu.github.io/plc-comm-docs-site/plc-setup/slmp/troubleshooting-end-codes/)
page. For profile limits and device availability, use the shared
[SLMP Profile Parameters](https://fa-yoshinobu.github.io/plc-comm-docs-site/slmp/profile-reference/parameters/)
page.

## Current library-specific caveats

| Area | Symptom | Guidance |
| --- | --- | --- |
| Request ordering | Two async tasks sharing one PLC connection appear to run one request at a time. | A single SLMP connection is one ordered frame stream. Share `SlmpClient` clones for safety, or open separate connections only when your PLC and network design allow it. |

```rust
let (left, right) = tokio::join!(
    read_typed(&client, SlmpAddress::parse("D100")?, "U"),
    read_typed(&client, SlmpAddress::parse("D101")?, "U"),
);
```
