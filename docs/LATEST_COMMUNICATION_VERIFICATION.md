# Latest communication verification

This page summarizes current live-device validation for public use.

| Date | PLC / CPU | Canonical profile | Transport | Verified scope | Public notes |
| --- | --- | --- | --- | --- | --- |
| 2026-05-03 | MELSEC iQ-F / FX5UC-32MT/D | `melsec:iq-f` | TCP `1025`, UDP `1027` | Device-range sample validation and profile-specific address checks. | iQ-F `X`/`Y` text notation is octal; `DX` and `DY` are not valid for this profile. |
| 2026-05-03 | MELSEC iQ-R / R08CPU | `melsec:iq-r` | TCP `1025`, UDP `1027` | Device-range sample validation, all-device checker coverage, and bit-block checks. | `S` is read-only for this profile. |
| 2026-05-03 | MELSEC iQ-L / L16HCPU | `melsec:iq-l` | TCP `1025`, UDP `1027` | Device-range sample validation and live stress validation. | iQ-L has its own explicit profile. |
| 2026-05-03 | MELSEC LCPU | `melsec:lcpu` | TCP `1025`, UDP `1027` | Device-range sample validation. | Legacy 3E profile. |
| 2026-05-15 | MELSEC Q12HCPU | `melsec:qcpu` | TCP `1025` | Runtime range validation for legacy 3E QCPU routing. | Legacy 3E profile. |
| 2026-05-15 | MELSEC Q26UDEHCPU | `melsec:qnu` | TCP `1025` | Runtime range validation for QnU routing. | Legacy 3E profile. |
| 2026-05-03 / 2026-05-15 | MELSEC Q06UDVCPU | `melsec:qnudv` | TCP `1025` | Device-range sample validation and runtime range validation. | Legacy 3E profile. |

Update this page when a new live-device validation result becomes the public summary.
