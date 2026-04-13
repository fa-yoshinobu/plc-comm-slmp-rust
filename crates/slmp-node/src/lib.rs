use napi_derive::napi;
use plc_comm_slmp::SlmpAddress;

#[napi]
pub fn normalize_address(address: String) -> napi::Result<String> {
    SlmpAddress::normalize(&address).map_err(|error| napi::Error::from_reason(error.message))
}
