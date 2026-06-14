# Latest communication verification

This page is the public index for retained live-device validation records.

| Date | PLC / CPU | Canonical profile | Transport | Verified scope | Limitations | Record |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-05-03 | MELSEC iQ-F / FX5UC-32MT/D | `melsec:iq-f` | TCP `1025`, UDP `1027` | Device-range sample validation and profile-specific address checks. | iQ-F `X`/`Y` text notation is octal; `DX`/`DY` are not valid for this profile. | [iQ-F range validation](IQF_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md) |
| 2026-05-03 | MELSEC iQ-R / R08CPU | `melsec:iq-r` | TCP `1025`, UDP `1027` | Device-range sample validation, all-device checker coverage, and bit-block checks. | Unsupported `S` entries are recorded in the validation note. | [iQ-R range validation](IQR_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md) |
| 2026-05-03 | MELSEC iQ-L / L16HCPU | `melsec:iq-l` | TCP `1025`, UDP `1027` | Device-range sample validation and live stress validation. | iQ-L keeps its own profile while using iQ-R-equivalent address grammar. | [iQ-L range validation](IQL_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md), [iQ-L live stress](IQL_LIVE_STRESS_VALIDATION_2026-05-03.md) |
| 2026-05-03 | MELSEC LCPU | `melsec:lcpu` | TCP `1025`, UDP `1027` | Device-range sample validation. | Profile-limited legacy 3E behavior is recorded in the validation note. | [LCPU range validation](LCPU_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md) |
| 2026-05-15 | MELSEC Q12HCPU | `melsec:qcpu` | TCP `1025` | Runtime range validation for legacy 3E QCPU routing. | `Z`, `ZR`, and `R` ranges are target-resolved and documented in the validation note. | [QCPU runtime range validation](QCPU_RUNTIME_RANGE_VALIDATION_2026-05-15.md) |
| 2026-05-15 | MELSEC Q26UDEHCPU | `melsec:qnu` | TCP `1025` | Runtime range validation for QnU routing. | Target-specific runtime limits are documented in the validation note. | [QnU runtime range validation](QNU_RUNTIME_RANGE_VALIDATION_2026-05-15.md) |
| 2026-05-03 / 2026-05-15 | MELSEC Q06UDVCPU | `melsec:qnudv` | TCP `1025` | Device-range sample validation and runtime range validation. | Target-specific runtime limits are documented in the validation notes. | [QnUDV range validation](QNUDV_DEVICE_RANGE_SAMPLE_VALIDATION_2026-05-03.md), [QnUDV runtime range validation](QNUDV_RUNTIME_RANGE_VALIDATION_2026-05-15.md) |

Update this page when a new live-device validation result becomes the public summary. Keep the raw validation notes in dated files and link them from the `Record` column.
