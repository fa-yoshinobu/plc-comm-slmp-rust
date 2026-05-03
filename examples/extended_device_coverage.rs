mod common;

use common::{env_bool, env_csv, env_string, options_from_env, print_connection_banner};
use plc_comm_slmp::{
    SlmpClient, SlmpDeviceCode, SlmpExtensionSpec, SlmpQualifiedDeviceAddress,
    parse_qualified_device,
};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct CoverageRow {
    device: String,
    points: u16,
    direct_memory: u8,
    unit: &'static str,
    status: &'static str,
    detail: String,
}

fn parse_auto_number(text: &str) -> Result<u32, Box<dyn Error>> {
    let trimmed = text.trim();
    if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        Ok(u32::from_str_radix(hex, 16)?)
    } else {
        Ok(trimmed.parse()?)
    }
}

fn effective_extension(
    qualified: SlmpQualifiedDeviceAddress,
    extension: SlmpExtensionSpec,
) -> SlmpExtensionSpec {
    let mut result = extension;
    if let Some(value) = qualified.extension_specification {
        result.extension_specification = value;
    }
    if let Some(value) = qualified.direct_memory_specification {
        result.direct_memory_specification = value;
    }
    result
}

fn is_bit_extended_device(code: SlmpDeviceCode) -> bool {
    code.is_bit_device()
}

fn format_words(values: &[u16]) -> String {
    values
        .iter()
        .map(|value| format!("0x{value:04X}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_bits(values: &[bool]) -> String {
    values
        .iter()
        .map(|value| if *value { "1" } else { "0" })
        .collect::<Vec<_>>()
        .join(", ")
}

fn markdown_cell(value: &str) -> String {
    value.replace('|', "/")
}

async fn check_bit_device(
    client: &SlmpClient,
    device_text: &str,
    qualified: SlmpQualifiedDeviceAddress,
    points: u16,
    extension: SlmpExtensionSpec,
    write_check: bool,
) -> Result<String, Box<dyn Error>> {
    let before = client
        .read_bits_extended(qualified, points, extension)
        .await?;
    if !write_check {
        return Ok(format!(
            "device={device_text}, points={points}, before=[{}], mode=read_only",
            format_bits(&before)
        ));
    }

    let write = (0..points).map(|index| index % 2 == 0).collect::<Vec<_>>();
    client
        .write_bits_extended(qualified, &write, extension)
        .await?;
    let readback = client
        .read_bits_extended(qualified, points, extension)
        .await?;
    let restore = match client
        .write_bits_extended(qualified, &before, extension)
        .await
    {
        Ok(()) => "ok",
        Err(_) => "failed",
    };
    let mismatch = readback != write;
    let mismatch_text = if mismatch {
        ", readback_mismatch=yes"
    } else {
        ""
    };
    Ok(format!(
        "device={device_text}, points={points}, before=[{}], write=[{}], readback=[{}]{mismatch_text}, restore={restore}",
        format_bits(&before),
        format_bits(&write),
        format_bits(&readback)
    ))
}

async fn check_word_device(
    client: &SlmpClient,
    device_text: &str,
    qualified: SlmpQualifiedDeviceAddress,
    points: u16,
    extension: SlmpExtensionSpec,
    write_check: bool,
) -> Result<String, Box<dyn Error>> {
    let before = client
        .read_words_extended(qualified, points, extension)
        .await?;
    if !write_check {
        return Ok(format!(
            "device={device_text}, points={points}, before=[{}], mode=read_only",
            format_words(&before)
        ));
    }

    let write = (0..points)
        .map(|index| 0x001E_u16.wrapping_add(index))
        .collect::<Vec<_>>();
    client
        .write_words_extended(qualified, &write, extension)
        .await?;
    let readback = client
        .read_words_extended(qualified, points, extension)
        .await?;
    let restore = match client
        .write_words_extended(qualified, &before, extension)
        .await
    {
        Ok(()) => "ok",
        Err(_) => "failed",
    };
    let mismatch = readback != write;
    let mismatch_text = if mismatch {
        ", readback_mismatch=yes"
    } else {
        ""
    };
    Ok(format!(
        "device={device_text}, points={points}, before=[{}], write=[{}], readback=[{}]{mismatch_text}, restore={restore}",
        format_words(&before),
        format_words(&write),
        format_words(&readback)
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_connection_banner("extended_device_coverage");
    let options = options_from_env()?;
    let client = SlmpClient::connect(options).await?;
    let write_check = env_bool("SLMP_EXT_WRITE_CHECK");
    let devices = env_csv("SLMP_EXT_DEVICES", r"U3E0\G10");
    let points = env_csv("SLMP_EXT_POINTS", "1")
        .into_iter()
        .map(|value| Ok(parse_auto_number(&value)? as u16))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let direct_memories = env_csv("SLMP_EXT_DIRECT_MEMORIES", "0xF8")
        .into_iter()
        .map(|value| Ok(parse_auto_number(&value)? as u8))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    println!("=== Extended Device Coverage Sweep ===");
    println!(
        "write_check={} devices=[{}] points=[{}] direct_memories=[{}]",
        write_check,
        devices.join(", "),
        points
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        direct_memories
            .iter()
            .map(|value| format!("0x{value:02X}"))
            .collect::<Vec<_>>()
            .join(", ")
    );

    let mut rows = Vec::new();
    for device_text in &devices {
        let qualified = parse_qualified_device(device_text)?;
        let unit = if is_bit_extended_device(qualified.device.code) {
            "bit"
        } else {
            "word"
        };
        for direct_memory in &direct_memories {
            let extension = effective_extension(
                qualified,
                SlmpExtensionSpec {
                    direct_memory_specification: *direct_memory,
                    ..SlmpExtensionSpec::default()
                },
            );
            for point_count in &points {
                let result = if unit == "bit" {
                    check_bit_device(
                        &client,
                        device_text,
                        qualified,
                        *point_count,
                        extension,
                        write_check,
                    )
                    .await
                } else {
                    check_word_device(
                        &client,
                        device_text,
                        qualified,
                        *point_count,
                        extension,
                        write_check,
                    )
                    .await
                };

                match result {
                    Ok(detail) => {
                        let status = if detail.contains("readback_mismatch=yes") {
                            "NG"
                        } else {
                            "OK"
                        };
                        println!(
                            "[{status}] {device_text} points={point_count} unit={unit} direct=0x{:02X}: {detail}",
                            extension.direct_memory_specification
                        );
                        rows.push(CoverageRow {
                            device: device_text.clone(),
                            points: *point_count,
                            direct_memory: extension.direct_memory_specification,
                            unit,
                            status,
                            detail,
                        });
                    }
                    Err(error) => {
                        println!(
                            "[NG] {device_text} points={point_count} unit={unit} direct=0x{:02X}: {error}",
                            extension.direct_memory_specification
                        );
                        rows.push(CoverageRow {
                            device: device_text.clone(),
                            points: *point_count,
                            direct_memory: extension.direct_memory_specification,
                            unit,
                            status: "NG",
                            detail: error.to_string(),
                        });
                    }
                }
            }
        }
    }

    let report_dir = PathBuf::from(env_string("SLMP_EXT_REPORT_DIR", "docs"));
    fs::create_dir_all(&report_dir)?;
    let report_path = report_dir.join("EXTENDED_DEVICE_COVERAGE_LATEST.md");
    let mut report = String::new();
    report.push_str("# Extended Device Coverage Latest\n\n");
    report.push_str(&format!(
        "- Host: {}\n",
        env_string("SLMP_HOST", "127.0.0.1")
    ));
    report.push_str(&format!("- Port: {}\n", env_string("SLMP_PORT", "1025")));
    report.push_str(&format!(
        "- PLC family: {}\n",
        env_string("SLMP_PLC_FAMILY", "iq-r")
    ));
    report.push_str(&format!(
        "- Transport: {}\n",
        env_string("SLMP_TRANSPORT", "tcp")
    ));
    report.push_str(&format!("- Write check: {write_check}\n"));
    report.push_str("\n| Device | Points | Unit | Direct | Status | Detail |\n");
    report.push_str("|---|---:|---|---:|---|---|\n");
    for row in &rows {
        report.push_str(&format!(
            "| {} | {} | {} | 0x{:02X} | {} | {} |\n",
            row.device,
            row.points,
            row.unit,
            row.direct_memory,
            row.status,
            markdown_cell(&row.detail)
        ));
    }
    fs::write(&report_path, report)?;
    println!("[DONE] report={}", report_path.display());

    let failures = rows.iter().filter(|row| row.status == "NG").count();
    if failures == 0 {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("{failures} extended device checks failed")).into())
    }
}
