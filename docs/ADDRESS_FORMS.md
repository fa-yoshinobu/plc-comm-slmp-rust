# Address Forms

This crate accepts the same practical address forms used across the other
SLMP libraries in this family.

## Plain Device Forms

Use plain device names when you want the default type for that device.

| Form | Meaning |
| --- | --- |
| `D100` | word device |
| `R50` | file register |
| `ZR0` | extended file register |
| `M100` | bit device |
| `X20` | hex-addressed input |
| `Y20` | hex-addressed output |
| `TN10` | timer current value |
| `TS10` | timer contact |
| `TC10` | timer coil |

## Typed Suffixes

Use `:suffix` when the same word address should be interpreted as a specific
scalar type.

| Form | Meaning |
| --- | --- |
| `D100:S` | signed 16-bit |
| `D200:D` | unsigned 32-bit |
| `D300:L` | signed 32-bit |
| `D400:F` | `f32` |

Rules:

- bit devices default to `Bool`
- plain word devices default to `U16`
- long current-value devices default to `U32`

## Bit-In-Word Form

Use `.bit` only on word devices.

| Form | Meaning |
| --- | --- |
| `D50.3` | bit 3 of `D50` |
| `W10.A` | bit `0xA` of `W10` |

Invalid examples:

- `M10.0`
- `X20.1`

Those should be addressed directly as bit devices.

## Long Timer and Long Counter Families

These families are normalized so that reads match the other libraries.

| Form | Meaning |
| --- | --- |
| `LTN10:D` | long timer current value |
| `LTS10` | long timer contact |
| `LTC10` | long timer coil |
| `LSTN10:D` | long retentive timer current value |
| `LSTS10` | long retentive timer contact |
| `LSTC10` | long retentive timer coil |
| `LCN10:D` | long counter current value |
| `LCS10` | long counter contact |
| `LCC10` | long counter coil |
| `LZ0:D` | long index register |

Notes:

- `LTN`, `LSTN`, `LCN`, and `LZ` are 32-bit values
- `LTS`, `LTC`, `LSTS`, `LSTC`, `LCS`, and `LCC` are state reads
- `LCS` and `LCC` are read through the `LCN` state block so behavior stays
  aligned across languages
- `LCS` and `LCC` are rejected for `Read Random (0x0403)`, `Read Block (0x0406)`,
  `Write Block (0x1406)`, and `Entry Monitor Device (0x0801)`
- `LTS`, `LTC`, `LSTS`, and `LSTC` are not exposed as direct bit reads in the
  Rust client API
- `LTN` and `LSTN` are not exposed as direct dword reads in the Rust client API;
  use helper APIs or 4-word block reads instead

## Extended Devices

Extended addresses are supported for both `J` and `U` forms.

| Form | Meaning |
| --- | --- |
| `J1\\W10` | network direct memory |
| `J1\\X10` | network direct bit device |
| `U3\\G100` | extension unit direct memory |
| `U1\\HG0` | extension unit high-speed direct memory |

Use [`parse_qualified_device`](../src/address.rs) or the raw client extended
read/write APIs for these forms.
