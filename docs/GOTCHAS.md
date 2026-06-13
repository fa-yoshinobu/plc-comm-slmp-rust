# Gotchas

## LTN/LSTN/LCN/LZ reads return wrong values

These are 32-bit families. Use `:D` or `:L` in named addresses and use an explicit 32-bit dtype with typed reads.

Fix:

```rust
use plc_comm_slmp::{read_typed, SlmpAddress};

# async fn demo(client: &plc_comm_slmp::SlmpClient) -> Result<(), plc_comm_slmp::SlmpError> {
let value = read_typed(&client, SlmpAddress::parse("LTN0")?, "D").await?;
println!("{:?}", value);
# Ok(())
# }
```

## LCS/LCC reads look incorrect

`LCS` and `LCC` state bits use direct bit read. Writes use random bit write (`0x1402`).

Fix: use `write_bit_in_word` for bit-in-word targets, or use `write_typed`/`write_named` for `LCS` and `LCC` state writes so the library routes correctly.

## LTS/LTC/LSTS/LSTC write rejected

Direct bit write is rejected for long timer state devices.

Fix: use `write_typed` or `write_named`; the helper route selects random bit write.

## G or HG address raises an error

`G` and `HG` are not in the public high-level API.

Fix: use low-level extended-device methods with qualified addresses such as `U3\G100` or `U3E0\HG0`.

## Mixed write fails with PLC error

Some PLC paths reject `0x1406` block writes for mixed word and bit combinations, even though the Rust client now encodes block write payloads in manual order.

Fix: separate word writes and bit writes, or use `SlmpBlockWriteOptions { split_mixed_blocks: true }` when you intentionally want two block write requests.

## DX or DY fails on `SlmpPlcProfile::IqF`

`DX` and `DY` are not valid for iQ-F profile parsing.

Fix: use `X` and `Y`.

## Profile not matching causes end code errors

`SlmpPlcProfile` determines the frame type and access mode.

Fix: verify that the selected profile matches your actual hardware.

## X/Y addresses look shifted on iQ-F

iQ-F parses `X` and `Y` string addresses as octal. Non-iQ-F profiles parse them as hexadecimal.

Fix: pass the correct `SlmpPlcProfile` and avoid sharing raw `X`/`Y` strings across profiles without checking notation.

## S appears in a catalog but parsing fails

The live device-range catalog can report `S`, but the public parser/client does not expose the MELSEC step-relay device in this crate.

Fix: keep `S` out of your application address list until parser/client support is intentionally added.
