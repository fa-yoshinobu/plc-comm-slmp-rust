# Supported registers

This page lists the device families exposed by the public helper-layer address parser and typed helper APIs.

## Bit device families

| Family | Examples | Notes |
| --- | --- | --- |
| Special relays | `SM0` | System bit devices. |
| Inputs and outputs | `X10`, `Y10` | Hex-addressed except iQ-F string addresses, which use octal. |
| Internal relays | `M100`, `L100`, `F100`, `V100` | Plain form maps to `SlmpValue::Bool`. |
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
| no suffix on a word device | `SlmpValue::U16` | Default word read. |
| no suffix on a bit device | `SlmpValue::Bool` | Default bit read. |
| `:U` | `SlmpValue::U16` | Unsigned 16-bit word. |
| `:S` | `SlmpValue::I16` | Signed 16-bit word. |
| `:D` | `SlmpValue::U32` | Unsigned 32-bit value. |
| `:L` | `SlmpValue::I32` | Signed 32-bit value. |
| `:F` | `SlmpValue::F32` | 32-bit floating-point value. |
| `.n` | `SlmpValue::Bool` | Bit `n` inside a word, where `n` is `0` through `F`. |

## Addressing notes

| Note | Detail |
| --- | --- |
| Long 32-bit families | `LTN`, `LSTN`, `LCN`, and `LZ` require `:D` or `:L`; plain low-level word access is rejected. |
| iQ-F direct I/O | `DX` and `DY` are not valid for `SlmpPlcProfile::IqF`; use `X` and `Y`. |
| Extended devices | `G` and `HG` are not in the public high-level surface. Use low-level extended-device APIs with qualified addresses such as `U3\G100` or `U3E0\HG0`. |
| PLC range limits | Use `SlmpClient::read_device_range_catalog()` to read live bounds for your selected profile. |

For profile selection, see [PROFILES.md](PROFILES.md).
