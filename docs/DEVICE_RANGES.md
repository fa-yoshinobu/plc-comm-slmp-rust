# Device Range Catalog

This document replaces `plc_device_range_registers - base.csv`.

The crate owns the device-range rules in source code and reads live upper
bounds from the PLC itself after the caller chooses the canonical PLC profile:

1. caller selects `SlmpPlcProfile`
2. the library uses that exact canonical PLC profile
3. read the profile-specific `SD` register window
4. build a `SlmpDeviceRangeCatalog` with point counts and 0-based address ranges

## Explicit Profile Policy

The library does not infer `SlmpPlcProfile` from `ReadTypeName`, model text, or
model code. `ReadTypeName` is diagnostic information only. Some PLCs or
communication paths cannot return a useful type name, and a wrong model-to-
profile conversion can select the wrong address grammar or range catalog. The
application, configuration UI, or operator must keep control of the PLC profile
selection.

`point_count` is the usable point count reported by the PLC or by a fixed
family rule. `upper_bound` is the inclusive last address, so for 0-based
devices it is `point_count - 1`. `address_range` is preformatted text such as
`X000-X1FF`.

`supported = false` means the family does not expose that device.
`supported = true` with `point_count = None` means the PLC family supports the
device but does not publish a finite bound register.

## API

`SlmpClient` exposes:

```rust
pub async fn read_device_range_catalog(&self) -> Result<SlmpDeviceRangeCatalog, SlmpError>
```

Example:

```rust
use plc_comm_slmp::{
    SlmpClient, SlmpConnectionOptions, SlmpPlcProfile,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = SlmpConnectionOptions::new("192.168.250.100", SlmpPlcProfile::IqF);
    options.port = 1025;

    let client = SlmpClient::connect(options).await?;
    let catalog = client.read_device_range_catalog().await?;

    println!("selected={} -> {:?}", catalog.model, catalog.plc_profile);
    for entry in catalog.entries.iter().filter(|x| x.supported) {
        println!(
            "{}: points={:?}, range={:?}, source={}",
            entry.device, entry.point_count, entry.address_range, entry.source
        );
    }

    Ok(())
}
```

Returned types:

- `SlmpDeviceRangeCatalog`
- `SlmpDeviceRangeEntry`
- `SlmpPlcProfile`
- `SlmpDeviceRangeCategory`
- `SlmpDeviceRangeNotation`

`SlmpDeviceRangeEntry.notation` uses `Decimal`, `Octal`, or `Hexadecimal` for the public
address text this crate expects.

## Internal Mapping

`read_device_range_catalog()` uses the `SlmpPlcProfile` configured on
`SlmpConnectionOptions`. `iq-l` uses independent iQ-L device-range rules.
`iQ-L` and legacy `LCPU` are treated as different PLC profiles in this crate;
they are not interchangeable.

Embedded profile rules cover:

- `IqR`
- `IqL`
- `MxF`
- `MxR`
- `IqF`
- `QCpu`
- `LCpu`
- `QnU`
- `QnUDV`

## Range Rules

`Addr` is the crate-facing address notation, not necessarily the notation used
in Mitsubishi manuals.

For iQ-R-series range rows, the `SD` register value is the PLC-configured
current point count. The catalog must cap that value at the iQ-R-series maximum
below, then derive `upper_bound = point_count - 1`. In other words, use
`point_count = min(SD point count, max_point_count)` for the rows listed here.
The iQ-R-series `0002/0003` device access format uses a 4-byte device number,
so these maximum addresses are representable by the protocol format.

| Item | Devices | Max address | max_point_count | Setting unit |
| --- | --- | --- | --- | --- |
| `X` | `X` | `X2FFF` | `12288` (`0x3000`) | n/a |
| `Y` | `Y` | `Y2FFF` | `12288` (`0x3000`) | n/a |
| `M` | `M` | `M94674943` | `94674944` (`0x5A4A000`) | 64 points |
| `B` | `B` | `B5A49FFF` | `94674944` (`0x5A4A000`) | 64 points |
| `F` | `F` | `F32767` | `32768` | 64 points |
| `SB` | `SB` | `SB5A49FFF` | `94674944` (`0x5A4A000`) | 64 points |
| `V` | `V` | `V32767` | `32768` | 64 points |
| `L` | `L` | `L32767` | `32768` | 64 points |
| `T` | `TS`, `TC`, `TN` | `T5259711` | `5259712` | 32 points |
| `ST` | `STS`, `STC`, `STN` | `ST5259711` | `5259712` | 32 points |
| `LT` | `LTS`, `LTC`, `LTN` | `LT1479295` | `1479296` | 1 point |
| `LST` | `LSTS`, `LSTC`, `LSTN` | `LST1479295` | `1479296` | 1 point |
| `C` | `CS`, `CC`, `CN` | `C5259711` | `5259712` | 32 points |
| `LC` | `LCS`, `LCC`, `LCN` | `LC2784543` | `2784544` | 32 points |
| `D` | `D` | `D5917183` | `5917184` (`0x5A4A00`) | 4 points |
| `W` | `W` | `W5A49FF` | `5917184` (`0x5A4A00`) | 4 points |
| `SW` | `SW` | `SW5A49FF` | `5917184` (`0x5A4A00`) | 4 points |

| Item | Devices | Addr | iQ-R | MX-F | MX-R | iQ-F | QCPU | LCPU / QnU / QnUDV |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `X` | `X` | base16 | `SD260-SD261` | `SD260-SD261` | `SD260-SD261` | `SD260-SD261` | `SD290` | `SD290` |
| `Y` | `Y` | base16 | `SD262-SD263` | `SD262-SD263` | `SD262-SD263` | `SD262-SD263` | `SD291` | `SD291` |
| `M` | `M` | base10 | `SD264-SD265` | `SD264-SD265` | `SD264-SD265` | `SD264-SD265` | `SD292`, clip `32768` | `SD286-SD287` |
| `B` | `B` | base16 | `SD266-SD267` | `SD266-SD267` | `SD266-SD267` | `SD266-SD267` | `SD294`, clip `32768` | `SD288-SD289` |
| `SB` | `SB` | base16 | `SD268-SD269` | `SD268-SD269` | `SD268-SD269` | `SD268-SD269` | `SD296` | `SD296` |
| `F` | `F` | base10 | `SD270-SD271` | `SD270-SD271` | `SD270-SD271` | `SD270-SD271` | `SD295` | `SD295` |
| `V` | `V` | base10 | `SD272-SD273` | `SD272-SD273` | `SD272-SD273` | `n/a` | `SD297` | `SD297` |
| `L` | `L` | base10 | `SD274-SD275` | `SD274-SD275` | `SD274-SD275` | `SD274-SD275` | `SD293` | `SD293` |
| `S` | `S` | base10 | `SD276-SD277` | `n/a` | `n/a` | `n/a` | `SD298` | `SD298` |
| `D` | `D` | base10 | `SD280-SD281` | `SD280-SD281` | `SD280-SD281` | `SD280-SD281` | `SD302`, clip `32768` | `SD308-SD309` |
| `W` | `W` | base16 | `SD282-SD283` | `SD282-SD283` | `SD282-SD283` | `SD282-SD283` | `SD303`, clip `32768` | `SD310-SD311` |
| `SW` | `SW` | base16 | `SD284-SD285` | `SD284-SD285` | `SD284-SD285` | `SD284-SD285` | `SD304` | `SD304` |
| `R` | `R` | base10 | `SD306-SD307`, clip `32768` | `SD306-SD307`, clip `32768` | `SD306-SD307`, clip `32768` | `SD304-SD305` | fixed `32768` | `SD306-SD307`, clip `32768` |
| `T` | `TS`, `TC`, `TN` | base10 | `SD288-SD289` | `SD288-SD289` | `SD288-SD289` | `SD288-SD289` | `SD299` | `SD299` |
| `ST` | `STS`, `STC`, `STN` | base10 | `SD290-SD291` | `SD290-SD291` | `SD290-SD291` | `SD290-SD291` | `SD300` | `SD300` |
| `C` | `CS`, `CC`, `CN` | base10 | `SD292-SD293` | `SD292-SD293` | `SD292-SD293` | `SD292-SD293` | `SD301` | `SD301` |
| `LT` | `LTS`, `LTC`, `LTN` | base10 | `SD294-SD295` | `SD294-SD295` | `SD294-SD295` | `n/a` | `n/a` | `n/a` |
| `LST` | `LSTS`, `LSTC`, `LSTN` | base10 | `SD296-SD297` | `SD296-SD297` | `SD296-SD297` | `n/a` | `n/a` | `n/a` |
| `LC` | `LCS`, `LCC`, `LCN` | base10 | `SD298-SD299` | `SD298-SD299` | `SD298-SD299` | `SD298-SD299` | `n/a` | `n/a` |
| `Z` | `Z` | base10 | `SD300` | `SD300` | `SD300` | `SD300` | fixed `10` | fixed `20` |
| `LZ` | `LZ` | base10 | `SD302` | `SD302` | `SD302` | `SD302` | `n/a` | `n/a` |
| `ZR` | `ZR` | base10 | `SD306-SD307` | `SD306-SD307` | `SD306-SD307` | `n/a` | open | `SD306-SD307` |
| `RD` | `RD` | base10 | `SD308-SD309` | `SD308-SD309` | `SD308-SD309` | `n/a` | `n/a` | `n/a` |
| `SM` | `SM` | base10 | fixed `4096` | fixed `10000` | fixed `4496` | fixed `10000` | fixed `1024` | fixed `2048` |
| `SD` | `SD` | base10 | fixed `4096` | fixed `10000` | fixed `4496` | fixed `12000` | fixed `1024` | fixed `2048` |

## Notes

- `QCPU` `D` and `W` exclude the extended area and are clipped to `32768`.
- `QCPU` `Z` is runtime-probed. A Mitsubishi `Q12HCPU` was live-checked on
  2026-05-15 and resolved to `Z0-Z15`. The current Q12HCPU record is
  `docs/QCPU_RUNTIME_RANGE_VALIDATION_2026-05-15.md`.
- `QCPU` `ZR` remains supported, but there is no finite upper-bound register, so
  the base catalog reports `supported = true`, `point_count = None`, and
  `address_range = None`. Runtime probing on the same `Q12HCPU` target resolved
  `ZR0-ZR131071`, with `R` capped at `R0-R32767`.
- `LCPU`, `QnU`, and `QnUDV` share the same range-register layout in this
  implementation.
- `Z` on `LCPU`, `QnU`, and `QnUDV` is treated as the fixed `Z0-Z19` family
  limit. `QnU` was live-checked on 2026-05-15 against `Q26UDEHCPU`: `Z19` read
  successfully and `Z20` returned `0x4031`. The current QnU record is
  `docs/QNU_RUNTIME_RANGE_VALIDATION_2026-05-15.md`. `QnUDV` was live-checked
  on 2026-05-15 against `Q06UDVCPU`: `Z19` read successfully and `Z20` returned
  `0x4031`. The current QnUDV record is
  `docs/QNUDV_RUNTIME_RANGE_VALIDATION_2026-05-15.md`.
- `ZR` and `R` on `LCPU`, `QnU`, and `QnUDV` are runtime-limited by probing
  readable `ZR` addresses. `QnU` was live-checked on 2026-05-15 against
  `Q26UDEHCPU` with `ZR655359` accepted, `ZR655360` returning `0x4031`, and
  `R32767` accepted with `R32768` blocked by the client guard or returned as out
  of range. `QnUDV` was live-checked on 2026-05-15 against `Q06UDVCPU` with
  `ZR393215` accepted, `ZR393216` returning `0x4031`, and `R32767` accepted
  with `R32768` blocked by the client guard or returned as out of range.
- `iQ-F` `X` and `Y` are documented in Mitsubishi manuals with octal addressing.
  This crate emits `Octal` and formats ranges such as `X0000-X1777`.
