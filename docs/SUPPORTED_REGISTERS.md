# Supported registers

This page lists the device families exposed by the public helper-layer address parser and typed helper APIs.

## Bit device families

| Family | Examples | Notes |
| --- | --- | --- |
| Special relays | `SM0` | System bit devices. |
| Inputs and outputs | `X10`, `Y10` | Hex-addressed except iQ-F string addresses, which use octal. |
| Internal relays | `M100:BIT`, `L100:BIT`, `F100:BIT`, `V100:BIT` | Use `:BIT` in named helper addresses. |
| Step relays | `S10:BIT` | Read-only bit device; write routes are rejected before transport. |
| Link relays | `B10`, `SB10` | Hex-addressed bit families. |
| Timer states | `TS10`, `TC10`, `STS10`, `STC10` | Contact and coil state bits. |
| Counter states | `CS10`, `CC10` | Contact and coil state bits. |
| Long timer states | `LTS10`, `LTC10`, `LSTS10`, `LSTC10` | Read through helper routes; writes use random bit write. |
| Long counter states | `LCS10`, `LCC10` | Reads use direct bit read; writes use random bit write. |
| Direct access I/O | `DX10`, `DY10` | Not valid for `SlmpPlcProfile::IqF`. |

## Word device families

| Family | Examples | Notes |
| --- | --- | --- |
| Special registers | `SD0` | System word devices. |
| Data registers | `D100` | Common first read target. |
| Link registers | `W10`, `SW10` | Hex-addressed word families. |
| Timer current values | `TN10`, `STN10` | 16-bit current values. |
| Counter current values | `CN10` | 16-bit current value. |
| Long current values | `LTN10:D`, `LSTN10:D`, `LCN10:D` | 32-bit families; use `:D` or `:L`. |
| Index registers | `Z0`, `LZ0:D` | `LZ` is a 32-bit family; use `:D` or `:L`. |
| File and refresh registers | `R0`, `ZR0`, `RD0` | Profile support depends on your PLC catalog. |

## Type suffixes

| Suffix | Value | Use |
| --- | --- | --- |
| `:U` | `SlmpValue::U16` | Unsigned 16-bit word. |
| `:S` | `SlmpValue::I16` | Signed 16-bit word. |
| `:D` | `SlmpValue::U32` | Unsigned 32-bit value. |
| `:L` | `SlmpValue::I32` | Signed 32-bit value. |
| `:F` | `SlmpValue::F32` | 32-bit floating-point value. |
| `:BIT` | `SlmpValue::Bool` | Direct bit-device value. |
| `.n` | `SlmpValue::Bool` | Bit `n` inside a word, where `n` is `0` through `F`. |

## Addressing notes

| Note | Detail |
| --- | --- |
| Explicit named types | `read_named`, `write_named`, and `poll_named` require suffixes such as `D100:U` or `M100:BIT`; plain named addresses are rejected. |
| Long 32-bit families | `LTN`, `LSTN`, `LCN`, and `LZ` require `:D` or `:L`; plain low-level word access is rejected. |
| Step relay | `S` writes follow the selected profile's write policy; iQ-F allows writes, while iQ-R/iQ-L/MX/Q/L profiles mark `S` read-only. |
| iQ-F direct I/O | `DX` and `DY` are not valid for `SlmpPlcProfile::IqF`; use `X` and `Y`. |
| Extended devices | `G` and `HG` are not in the public high-level surface. Use low-level extended-device APIs with qualified addresses such as `U3\G100` or `U3E0\HG0`. |
| PLC range limits | Use `SlmpClient::read_device_range_catalog()` to read live bounds for your selected profile. |

## Long timer and long counter behavior

`LTN`, `LSTN`, `LCN`, and `LZ` are 32-bit logical families. Use helper APIs,
random dword high-level access, or 4-word block reads where supported. They are
not exposed as plain direct dword reads in the Rust client API.

`LCS` and `LCC` use direct bit read through `read_typed` / `read_named`; use `LCS0:BIT` or `LCC0:BIT` in named helper addresses.
`LTS`, `LTC`, `LSTS`, and `LSTC` are not exposed through direct bit
read/write or Read Random in the Rust client API; use helper APIs for reads and
random bit write where applicable.

## Extended device forms

Extended addresses are supported for both `J` and `U` forms through
`parse_qualified_device` and the raw extended read/write APIs.

| Form | Meaning |
| --- | --- |
| `J1\W10` | Network direct memory. |
| `J1\X10` | Network direct bit device. |
| `U3\G100` | Extension unit direct memory. |
| `U3E0\HG0` | iQ-R multi-CPU high-speed CPU-buffer memory. |

`HG` is valid only for `U3E0\HG` through `U3E3\HG`. Lower-unit forms such as
`U1\HG` are rejected before transport.

For profile selection, see [PROFILES.md](PROFILES.md).
