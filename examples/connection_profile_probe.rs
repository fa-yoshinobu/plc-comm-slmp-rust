mod common;

use common::{options_from_env, print_connection_banner};
use plc_comm_slmp::probe_connection_profiles;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("connection_profile_probe");
    let options = options_from_env()?;
    let results = probe_connection_profiles(&options).await;

    for result in results {
        let type_name_text = result
            .type_name_info
            .as_ref()
            .map(|info| format!("model={} model_code=0x{:04X}", info.model, info.model_code))
            .unwrap_or_else(|| "type_name=-".to_string());
        let family_text = result
            .family
            .map(|family| format!("{family:?}"))
            .unwrap_or_else(|| "-".to_string());
        let sd_text = match (result.sd_register_start, result.sd_register_count) {
            (Some(start), Some(count)) if count > 0 => {
                format!("sd_block=SD{start}-SD{} count={count}", start + count - 1)
            }
            (Some(start), Some(count)) => format!("sd_block=SD{start} count={count}"),
            _ => "sd_block=-".to_string(),
        };
        let error_text = result
            .error_message
            .as_ref()
            .map(|error| format!(" error={error}"))
            .unwrap_or_default();

        println!(
            "{:?}/{:?} status={:?} family={} sd_read={} {} {}{}",
            result.frame_type,
            result.compatibility_mode,
            result.status,
            family_text,
            result.sd_read_succeeded,
            sd_text,
            type_name_text,
            error_text
        );
    }

    Ok(())
}
