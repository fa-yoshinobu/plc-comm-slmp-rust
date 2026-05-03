use crate::address::SlmpAddress;
use crate::client::SlmpClient;
use crate::device_ranges::{
    SlmpDeviceRangeCatalog, SlmpDeviceRangeCategory, SlmpDeviceRangeEntry, SlmpDeviceRangeFamily,
    SlmpDeviceRangeNotation,
};
use crate::error::SlmpError;
use crate::helpers::{SlmpValue, read_typed, write_typed};
use crate::model::{SlmpBlockRead, SlmpBlockWrite, SlmpBlockWriteOptions, SlmpDeviceAddress};
use std::future::Future;

const DEFAULT_RANGE_END_CODE: u16 = 0x4031;
const IQF_RANGE_END_CODE: u16 = 0xC056;

#[derive(Debug, Clone, Copy)]
struct RouteCapabilities {
    block: bool,
    random: bool,
    lz: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpRouteValidationOptions {
    #[serde(default = "default_word_device")]
    pub word_device: String,
    #[serde(default = "default_dword_device")]
    pub dword_device: String,
    #[serde(default = "default_float_device")]
    pub float_device: String,
    #[serde(default = "default_bit_device")]
    pub bit_device: String,
    #[serde(default = "default_lz_device")]
    pub lz_device: String,
    #[serde(default)]
    pub range_family: Option<SlmpDeviceRangeFamily>,
    #[serde(default = "default_range_error_devices")]
    pub range_error_devices: Vec<String>,
}

impl Default for SlmpRouteValidationOptions {
    fn default() -> Self {
        Self {
            word_device: default_word_device(),
            dword_device: default_dword_device(),
            float_device: default_float_device(),
            bit_device: default_bit_device(),
            lz_device: default_lz_device(),
            range_family: None,
            range_error_devices: default_range_error_devices(),
        }
    }
}

impl SlmpRouteValidationOptions {
    pub fn normalized(mut self) -> Self {
        if self.word_device.trim().is_empty() {
            self.word_device = default_word_device();
        }
        if self.dword_device.trim().is_empty() {
            self.dword_device = default_dword_device();
        }
        if self.float_device.trim().is_empty() {
            self.float_device = default_float_device();
        }
        if self.bit_device.trim().is_empty() {
            self.bit_device = default_bit_device();
        }
        if self.lz_device.trim().is_empty() {
            self.lz_device = default_lz_device();
        }
        if self.range_error_devices.is_empty() {
            self.range_error_devices = default_range_error_devices();
        }
        self.range_error_devices = self
            .range_error_devices
            .into_iter()
            .map(|device| device.trim().to_ascii_uppercase())
            .filter(|device| !device.is_empty())
            .collect();
        self
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpRouteValidationSummary {
    pub passed: usize,
    pub failed: usize,
    pub warned: usize,
    pub skipped: usize,
}

impl SlmpRouteValidationSummary {
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SlmpRouteValidationStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpRouteValidationCase {
    pub route: String,
    pub name: String,
    pub status: SlmpRouteValidationStatus,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlmpRouteValidationReport {
    pub model: String,
    pub family: SlmpDeviceRangeFamily,
    pub options: SlmpRouteValidationOptions,
    pub summary: SlmpRouteValidationSummary,
    pub cases: Vec<SlmpRouteValidationCase>,
}

impl SlmpRouteValidationReport {
    pub fn is_success(&self) -> bool {
        self.summary.is_success()
    }

    fn push(
        &mut self,
        route: &str,
        name: &str,
        status: SlmpRouteValidationStatus,
        detail: impl Into<String>,
    ) {
        match status {
            SlmpRouteValidationStatus::Passed => self.summary.passed += 1,
            SlmpRouteValidationStatus::Failed => self.summary.failed += 1,
            SlmpRouteValidationStatus::Warning => self.summary.warned += 1,
            SlmpRouteValidationStatus::Skipped => self.summary.skipped += 1,
        }
        self.cases.push(SlmpRouteValidationCase {
            route: route.to_string(),
            name: name.to_string(),
            status,
            detail: detail.into(),
        });
    }
}

pub async fn run_route_validation_compare(
    client: &SlmpClient,
    options: SlmpRouteValidationOptions,
) -> Result<SlmpRouteValidationReport, SlmpError> {
    let options = options.normalized();
    let model = client
        .read_type_name()
        .await
        .map(|info| info.model)
        .unwrap_or_else(|_| "unknown".to_string());
    let family = match options.range_family {
        Some(family) => family,
        None => client.configured_device_range_family().await,
    };
    let options = apply_family_default_devices(options, family);
    let capabilities = route_capabilities(family);
    let mut report = SlmpRouteValidationReport {
        model,
        family,
        options: options.clone(),
        summary: SlmpRouteValidationSummary::default(),
        cases: Vec::new(),
    };

    if capabilities.block {
        record_case(&mut report, "block", "read_block_matches_direct", || {
            validate_block_read(client, &options)
        })
        .await;
        record_case(&mut report, "block", "write_block_roundtrip", || {
            validate_block_write(client, &options)
        })
        .await;
    } else {
        report.push(
            "block",
            "read_block_matches_direct",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support block route 0x0406"),
        );
        report.push(
            "block",
            "write_block_roundtrip",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support block route 0x1406"),
        );
    }
    if capabilities.lz {
        record_case(&mut report, "block", "lz_blocks_rejected", || {
            validate_lz_block_guards(client, &options)
        })
        .await;
    } else {
        report.push(
            "block",
            "lz_blocks_rejected",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support LZ"),
        );
    }
    if capabilities.random {
        record_case(&mut report, "random", "read_random_matches_direct", || {
            validate_random_read(client, &options)
        })
        .await;
        record_case(&mut report, "random", "write_random_roundtrip", || {
            validate_random_write(client, &options)
        })
        .await;
    } else {
        report.push(
            "random",
            "read_random_matches_direct",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support random route 0x0403"),
        );
        report.push(
            "random",
            "write_random_roundtrip",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support random route 0x1402"),
        );
    }
    if capabilities.lz {
        record_case(&mut report, "random", "lz_word_entries_rejected", || {
            validate_lz_random_word_guards(client, &options)
        })
        .await;
    } else {
        report.push(
            "random",
            "lz_word_entries_rejected",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support LZ"),
        );
    }
    record_case(&mut report, "typed", "word_dword_float_roundtrip", || {
        validate_typed_roundtrip(client, &options)
    })
    .await;
    if capabilities.lz && capabilities.random {
        record_case(&mut report, "typed", "lz_random_dword_roundtrip", || {
            validate_lz_typed_roundtrip(client, &options)
        })
        .await;
        record_case(&mut report, "typed", "lz_invalid_dtypes_rejected", || {
            validate_lz_typed_guards(client, &options)
        })
        .await;
    } else {
        report.push(
            "typed",
            "lz_random_dword_roundtrip",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support LZ random dword route"),
        );
        report.push(
            "typed",
            "lz_invalid_dtypes_rejected",
            SlmpRouteValidationStatus::Skipped,
            format!("{family:?} does not support LZ random dword route"),
        );
    }

    validate_range_error_routes(client, &options, family, capabilities, &mut report).await?;
    Ok(report)
}

async fn record_case<F, Fut>(
    report: &mut SlmpRouteValidationReport,
    route: &str,
    name: &str,
    run: F,
) where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<String, SlmpError>>,
{
    match run().await {
        Ok(detail) => report.push(route, name, SlmpRouteValidationStatus::Passed, detail),
        Err(error) => report.push(
            route,
            name,
            SlmpRouteValidationStatus::Failed,
            error.to_string(),
        ),
    }
}

async fn parse_for_client(
    client: &SlmpClient,
    address: &str,
) -> Result<SlmpDeviceAddress, SlmpError> {
    SlmpAddress::parse_for_plc_family(address, client.plc_family().await)
}

async fn validate_block_read(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let word = parse_for_client(client, &options.word_device).await?;
    let bit = parse_for_client(client, &options.bit_device).await?;
    let direct_words = client.read_words_raw(word, 2).await?;
    let direct_bits = client.read_bits(bit, 16).await?;
    let block = client
        .read_block(
            &[SlmpBlockRead {
                device: word,
                points: 2,
            }],
            &[SlmpBlockRead {
                device: bit,
                points: 1,
            }],
        )
        .await?;

    if block.word_values != direct_words {
        return Err(SlmpError::new(format!(
            "word mismatch: direct={direct_words:?} block={:?}",
            block.word_values
        )));
    }
    let packed_bits = pack_bit_words(&direct_bits);
    if block.bit_values != packed_bits {
        return Err(SlmpError::new(format!(
            "bit mismatch: direct_packed={packed_bits:?} block={:?}",
            block.bit_values
        )));
    }
    Ok(format!(
        "word={} points=2 bit={} bit_words=1",
        options.word_device, options.bit_device
    ))
}

async fn validate_block_write(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let word = parse_for_client(client, &options.word_device).await?;
    let bit = parse_for_client(client, &options.bit_device).await?;
    let original_words = client.read_words_raw(word, 2).await?;
    let original_bits = client.read_bits(bit, 16).await?;
    let write_words = vec![
        alternate_u16(original_words[0], 0x1234),
        alternate_u16(original_words[1], 0x5678),
    ];
    let write_bit_words = vec![alternate_u16(pack_bit_words(&original_bits)[0], 0x00AA)];

    let test_result: Result<(), SlmpError> = async {
        client
            .write_block(
                &[SlmpBlockWrite {
                    device: word,
                    values: write_words.clone(),
                }],
                &[SlmpBlockWrite {
                    device: bit,
                    values: write_bit_words.clone(),
                }],
                Some(SlmpBlockWriteOptions {
                    split_mixed_blocks: false,
                    retry_mixed_on_error: true,
                }),
            )
            .await?;
        let observed_words = client.read_words_raw(word, 2).await?;
        if observed_words != write_words {
            return Err(SlmpError::new(format!(
                "word write mismatch: expected={write_words:?} observed={observed_words:?}"
            )));
        }
        let observed_bits = pack_bit_words(&client.read_bits(bit, 16).await?);
        if observed_bits != write_bit_words {
            return Err(SlmpError::new(format!(
                "bit write mismatch: expected={write_bit_words:?} observed={observed_bits:?}"
            )));
        }
        Ok(())
    }
    .await;

    let restore_words = client.write_words(word, &original_words).await;
    let restore_bits = client.write_bits(bit, &original_bits).await;
    finish_with_restore(
        test_result,
        &[
            ("restore word block", restore_words),
            ("restore bit block", restore_bits),
        ],
    )?;

    Ok(format!(
        "word={} values=2 bit={} bit_words=1",
        options.word_device, options.bit_device
    ))
}

async fn validate_lz_block_guards(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let lz = parse_for_client(client, &options.lz_device).await?;
    expect_client_error(
        client
            .read_block(
                &[SlmpBlockRead {
                    device: lz,
                    points: 1,
                }],
                &[],
            )
            .await,
        "read_block LZ",
    )?;
    expect_client_error(
        client
            .write_block(
                &[SlmpBlockWrite {
                    device: lz,
                    values: vec![1],
                }],
                &[],
                None,
            )
            .await,
        "write_block LZ",
    )?;
    Ok(format!(
        "{} rejected before unsupported block route",
        options.lz_device
    ))
}

async fn validate_random_read(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let word = parse_for_client(client, &options.word_device).await?;
    let dword = parse_for_client(client, &options.dword_device).await?;
    let lz = parse_for_client(client, &options.lz_device).await?;
    let random = client.read_random(&[word], &[dword, lz]).await?;
    let direct_word = client.read_words_raw(word, 1).await?[0];
    let direct_dword = client.read_dwords_raw(dword, 1).await?[0];
    let typed_lz = expect_u32(read_typed(client, lz, "D").await?)?;

    if random.word_values != vec![direct_word] {
        return Err(SlmpError::new(format!(
            "random word mismatch: direct={direct_word} random={:?}",
            random.word_values
        )));
    }
    if random.dword_values != vec![direct_dword, typed_lz] {
        return Err(SlmpError::new(format!(
            "random dword mismatch: direct=[{direct_dword}, {typed_lz}] random={:?}",
            random.dword_values
        )));
    }
    Ok(format!(
        "word={} dword={} lz={}",
        options.word_device, options.dword_device, options.lz_device
    ))
}

async fn validate_random_write(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let word = parse_for_client(client, &options.word_device).await?;
    let dword = parse_for_client(client, &options.dword_device).await?;
    let lz = parse_for_client(client, &options.lz_device).await?;
    let bit = parse_for_client(client, &options.bit_device).await?;

    let original_word = client.read_words_raw(word, 1).await?[0];
    let original_dword = client.read_dwords_raw(dword, 1).await?[0];
    let original_lz = expect_u32(read_typed(client, lz, "D").await?)?;
    let original_bit = client.read_bits(bit, 1).await?[0];
    let write_word = alternate_u16(original_word, 0x2468);
    let write_dword = alternate_u32(original_dword, 0x1357_2468);
    let write_lz = alternate_u32(original_lz, 0x2468_1357);
    let write_bit = !original_bit;

    let test_result: Result<(), SlmpError> = async {
        client
            .write_random_words(&[(word, write_word)], &[(dword, write_dword), (lz, write_lz)])
            .await?;
        client.write_random_bits(&[(bit, write_bit)]).await?;
        let observed_word = client.read_words_raw(word, 1).await?[0];
        let observed_dword = client.read_dwords_raw(dword, 1).await?[0];
        let observed_lz = expect_u32(read_typed(client, lz, "D").await?)?;
        let observed_bit = client.read_bits(bit, 1).await?[0];
        if observed_word != write_word
            || observed_dword != write_dword
            || observed_lz != write_lz
            || observed_bit != write_bit
        {
            return Err(SlmpError::new(format!(
                "random write mismatch: word {observed_word}/{write_word}, dword {observed_dword}/{write_dword}, lz {observed_lz}/{write_lz}, bit {observed_bit}/{write_bit}"
            )));
        }
        Ok(())
    }
    .await;

    let restore_word = client.write_words(word, &[original_word]).await;
    let restore_dword = client.write_dwords(dword, &[original_dword]).await;
    let restore_lz = client.write_random_words(&[], &[(lz, original_lz)]).await;
    let restore_bit = client.write_bits(bit, &[original_bit]).await;
    finish_with_restore(
        test_result,
        &[
            ("restore random word", restore_word),
            ("restore random dword", restore_dword),
            ("restore random lz", restore_lz),
            ("restore random bit", restore_bit),
        ],
    )?;

    Ok(format!(
        "word={} dword={} lz={} bit={}",
        options.word_device, options.dword_device, options.lz_device, options.bit_device
    ))
}

async fn validate_lz_random_word_guards(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let lz = parse_for_client(client, &options.lz_device).await?;
    expect_client_error(
        client.read_random(&[lz], &[]).await,
        "read_random LZ word entry",
    )?;
    expect_client_error(
        client.write_random_words(&[(lz, 1)], &[]).await,
        "write_random_words LZ word entry",
    )?;
    Ok(format!(
        "{} rejected as word entry; dword route is required",
        options.lz_device
    ))
}

async fn validate_typed_roundtrip(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let word = parse_for_client(client, &options.word_device).await?;
    let dword = parse_for_client(client, &options.dword_device).await?;
    let float = parse_for_client(client, &options.float_device).await?;
    let original_word = client.read_words_raw(word, 1).await?[0];
    let original_dword = client.read_dwords_raw(dword, 1).await?[0];
    let original_float = client.read_dwords_raw(float, 1).await?[0];
    let write_word = alternate_u16(original_word, 0x1357);
    let write_dword = alternate_u32(original_dword, 0x1234_ABCD);
    let write_float = if original_float == 12.5f32.to_bits() {
        -7.25f32
    } else {
        12.5f32
    };

    let test_result: Result<(), SlmpError> = async {
        write_typed(client, word, "U", &SlmpValue::U16(write_word)).await?;
        let observed_word = expect_u16(read_typed(client, word, "U").await?)?;
        if observed_word != write_word {
            return Err(SlmpError::new(format!(
                "typed word mismatch: expected={write_word} observed={observed_word}"
            )));
        }

        write_typed(client, dword, "D", &SlmpValue::U32(write_dword)).await?;
        let observed_dword = expect_u32(read_typed(client, dword, "D").await?)?;
        if observed_dword != write_dword {
            return Err(SlmpError::new(format!(
                "typed dword mismatch: expected={write_dword} observed={observed_dword}"
            )));
        }

        write_typed(client, float, "F", &SlmpValue::F32(write_float)).await?;
        let observed_float = expect_f32(read_typed(client, float, "F").await?)?;
        if observed_float.to_bits() != write_float.to_bits() {
            return Err(SlmpError::new(format!(
                "typed float mismatch: expected_bits=0x{:08X} observed_bits=0x{:08X}",
                write_float.to_bits(),
                observed_float.to_bits()
            )));
        }
        Ok(())
    }
    .await;

    let restore_word = client.write_words(word, &[original_word]).await;
    let restore_dword = client.write_dwords(dword, &[original_dword]).await;
    let restore_float = client.write_dwords(float, &[original_float]).await;
    finish_with_restore(
        test_result,
        &[
            ("restore typed word", restore_word),
            ("restore typed dword", restore_dword),
            ("restore typed float", restore_float),
        ],
    )?;

    Ok(format!(
        "word={} dword={} float={}",
        options.word_device, options.dword_device, options.float_device
    ))
}

async fn validate_lz_typed_roundtrip(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let lz = parse_for_client(client, &options.lz_device).await?;
    let original = expect_u32(read_typed(client, lz, "D").await?)?;
    let write = alternate_u32(original, 0x0BAD_F00D);
    let test_result: Result<(), SlmpError> = async {
        write_typed(client, lz, "D", &SlmpValue::U32(write)).await?;
        let observed = expect_u32(read_typed(client, lz, "D").await?)?;
        if observed != write {
            return Err(SlmpError::new(format!(
                "typed LZ mismatch: expected={write} observed={observed}"
            )));
        }
        Ok(())
    }
    .await;
    let restore = write_typed(client, lz, "D", &SlmpValue::U32(original)).await;
    finish_with_restore(test_result, &[("restore typed LZ", restore)])?;
    Ok(format!(
        "{} uses random dword typed route",
        options.lz_device
    ))
}

async fn validate_lz_typed_guards(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
) -> Result<String, SlmpError> {
    let lz = parse_for_client(client, &options.lz_device).await?;
    expect_client_error(read_typed(client, lz, "U").await, "read_typed LZ:U")?;
    for dtype in ["U", "S", "F", "BIT"] {
        expect_client_error(
            write_typed(client, lz, dtype, &SlmpValue::U16(1)).await,
            &format!("write_typed LZ:{dtype}"),
        )?;
    }
    Ok(format!(
        "{} rejects U/S/F/BIT and accepts D/L only",
        options.lz_device
    ))
}

async fn validate_range_error_routes(
    client: &SlmpClient,
    options: &SlmpRouteValidationOptions,
    family: SlmpDeviceRangeFamily,
    capabilities: RouteCapabilities,
    report: &mut SlmpRouteValidationReport,
) -> Result<(), SlmpError> {
    let expected_end_code = expected_range_end_code(family);
    let catalog = match options.range_family {
        Some(family) => client.read_device_range_catalog_for_family(family).await?,
        None => client.read_device_range_catalog().await?,
    };
    for device in &options.range_error_devices {
        let Some(entry) = catalog.entries.iter().find(|entry| &entry.device == device) else {
            report.push(
                "range",
                &format!("{device}_out_of_range"),
                SlmpRouteValidationStatus::Skipped,
                format!("{device} is not in the {family:?} range catalog"),
            );
            continue;
        };
        if !entry.supported {
            report.push(
                "range",
                &format!("{device}_out_of_range"),
                SlmpRouteValidationStatus::Skipped,
                format!("{device} is unsupported in the {family:?} range catalog"),
            );
            continue;
        }
        record_range_case(
            client,
            &catalog,
            entry,
            expected_end_code,
            capabilities,
            report,
        )
        .await;
    }
    Ok(())
}

async fn record_range_case(
    client: &SlmpClient,
    catalog: &SlmpDeviceRangeCatalog,
    entry: &SlmpDeviceRangeEntry,
    expected_end_code: u16,
    capabilities: RouteCapabilities,
    report: &mut SlmpRouteValidationReport,
) {
    let name = format!("{}_out_of_range", entry.device);
    match validate_one_range_error_device(client, catalog, entry, expected_end_code, capabilities)
        .await
    {
        Ok(detail) => report.push("range", &name, SlmpRouteValidationStatus::Passed, detail),
        Err(error) if error.message.starts_with("skip:") => report.push(
            "range",
            &name,
            SlmpRouteValidationStatus::Skipped,
            error.message.trim_start_matches("skip:").trim().to_string(),
        ),
        Err(error) if error.message.starts_with("warning:") => report.push(
            "range",
            &name,
            SlmpRouteValidationStatus::Warning,
            error
                .message
                .trim_start_matches("warning:")
                .trim()
                .to_string(),
        ),
        Err(error) => report.push(
            "range",
            &name,
            SlmpRouteValidationStatus::Failed,
            error.to_string(),
        ),
    }
}

async fn validate_one_range_error_device(
    client: &SlmpClient,
    _catalog: &SlmpDeviceRangeCatalog,
    entry: &SlmpDeviceRangeEntry,
    expected_end_code: u16,
    capabilities: RouteCapabilities,
) -> Result<String, SlmpError> {
    let Some(upper_bound) = entry.upper_bound else {
        return Err(SlmpError::new(format!(
            "skip: {} has no finite upper bound",
            entry.device
        )));
    };
    let Some(out_number) = upper_bound.checked_add(1) else {
        return Err(SlmpError::new(format!(
            "skip: {} upper bound cannot be incremented",
            entry.device
        )));
    };
    let address = format_entry_address(entry, out_number);
    let device = parse_for_client(client, &address).await?;
    let mut warnings = Vec::new();

    if entry.device == "LZ" {
        if !capabilities.lz || !capabilities.random {
            return Err(SlmpError::new(format!(
                "skip: {} LZ random dword route is unsupported for this family",
                entry.device
            )));
        }
        expect_range_read_or_warn(
            read_typed(client, device, "D").await,
            "read_typed range LZ:D",
            expected_end_code,
            &mut warnings,
        )?;
        expect_range_error(
            write_typed(client, device, "D", &SlmpValue::U32(1)).await,
            "write_typed range LZ:D",
            expected_end_code,
        )?;
        expect_range_read_or_warn(
            client.read_random(&[], &[device]).await,
            "read_random range LZ dword",
            expected_end_code,
            &mut warnings,
        )?;
        expect_range_error(
            client.write_random_words(&[], &[(device, 1)]).await,
            "write_random range LZ dword",
            expected_end_code,
        )?;
        return range_detail_or_warning(
            format!("{address} returned 0x{expected_end_code:04X} on typed/random dword routes"),
            warnings,
        );
    }

    if entry.is_bit_device {
        let mut checked_routes = vec!["bit"];
        expect_range_error(
            client.read_bits(device, 1).await,
            "read_bits range",
            expected_end_code,
        )?;
        expect_range_error(
            client.write_bits(device, &[false]).await,
            "write_bits range",
            expected_end_code,
        )?;
        if capabilities.random {
            checked_routes.push("random-bit");
            expect_range_error(
                client.write_random_bits(&[(device, false)]).await,
                "write_random_bits range",
                expected_end_code,
            )?;
        }
        if capabilities.block {
            checked_routes.push("block");
            expect_range_read_or_warn(
                client
                    .read_block(&[], &[SlmpBlockRead { device, points: 1 }])
                    .await,
                "read_block bit range",
                expected_end_code,
                &mut warnings,
            )?;
            expect_range_error(
                client
                    .write_block(
                        &[],
                        &[SlmpBlockWrite {
                            device,
                            values: vec![0],
                        }],
                        None,
                    )
                    .await,
                "write_block bit range",
                expected_end_code,
            )?;
        }
        return range_detail_or_warning(
            format!(
                "{address} returned 0x{expected_end_code:04X} on {} routes",
                checked_routes.join("/")
            ),
            warnings,
        );
    }

    let mut checked_routes = vec!["word", "typed"];
    expect_range_error(
        client.read_words_raw(device, 1).await,
        "read_words range",
        expected_end_code,
    )?;
    expect_range_error(
        client.write_words(device, &[0]).await,
        "write_words range",
        expected_end_code,
    )?;
    expect_range_error(
        read_typed(client, device, "U").await,
        "read_typed range U",
        expected_end_code,
    )?;
    expect_range_error(
        write_typed(client, device, "U", &SlmpValue::U16(0)).await,
        "write_typed range U",
        expected_end_code,
    )?;
    if capabilities.random {
        checked_routes.push("random");
        expect_range_read_or_warn(
            client.read_random(&[device], &[]).await,
            "read_random word range",
            expected_end_code,
            &mut warnings,
        )?;
        expect_range_error(
            client.write_random_words(&[(device, 0)], &[]).await,
            "write_random_words range",
            expected_end_code,
        )?;
    }
    if capabilities.block {
        checked_routes.push("block");
        expect_range_read_or_warn(
            client
                .read_block(&[SlmpBlockRead { device, points: 1 }], &[])
                .await,
            "read_block word range",
            expected_end_code,
            &mut warnings,
        )?;
        expect_range_error(
            client
                .write_block(
                    &[SlmpBlockWrite {
                        device,
                        values: vec![0],
                    }],
                    &[],
                    None,
                )
                .await,
            "write_block word range",
            expected_end_code,
        )?;
    }
    range_detail_or_warning(
        format!(
            "{address} returned 0x{expected_end_code:04X} on {} routes",
            checked_routes.join("/")
        ),
        warnings,
    )
}

fn expect_range_error<T>(
    result: Result<T, SlmpError>,
    operation: &str,
    expected_end_code: u16,
) -> Result<(), SlmpError> {
    match result {
        Ok(_) => Err(SlmpError::new(format!(
            "{operation} unexpectedly succeeded; expected end_code=0x{expected_end_code:04X}"
        ))),
        Err(error) if error.end_code == Some(expected_end_code) => Ok(()),
        Err(error) => Err(SlmpError::new(format!(
            "{operation} expected end_code=0x{expected_end_code:04X}, got {error}"
        ))),
    }
}

fn expect_range_read_or_warn<T>(
    result: Result<T, SlmpError>,
    operation: &str,
    expected_end_code: u16,
    warnings: &mut Vec<String>,
) -> Result<(), SlmpError> {
    match result {
        Ok(_) => {
            warnings.push(format!(
                "{operation} unexpectedly succeeded; expected end_code=0x{expected_end_code:04X}"
            ));
            Ok(())
        }
        Err(error) if error.end_code == Some(expected_end_code) => Ok(()),
        Err(error) => Err(SlmpError::new(format!(
            "{operation} expected end_code=0x{expected_end_code:04X}, got {error}"
        ))),
    }
}

fn range_detail_or_warning(detail: String, warnings: Vec<String>) -> Result<String, SlmpError> {
    if warnings.is_empty() {
        Ok(detail)
    } else {
        Err(SlmpError::new(format!(
            "warning: {detail}; {}",
            warnings.join("; ")
        )))
    }
}

fn expect_client_error<T>(result: Result<T, SlmpError>, operation: &str) -> Result<(), SlmpError> {
    match result {
        Ok(_) => Err(SlmpError::new(format!(
            "{operation} unexpectedly succeeded"
        ))),
        Err(error) if error.end_code.is_none() => Ok(()),
        Err(error) => Err(SlmpError::new(format!(
            "{operation} should be rejected before transport, got {error}"
        ))),
    }
}

fn finish_with_restore(
    test_result: Result<(), SlmpError>,
    restores: &[(&str, Result<(), SlmpError>)],
) -> Result<(), SlmpError> {
    let restore_errors = restores
        .iter()
        .filter_map(|(label, result)| {
            result
                .as_ref()
                .err()
                .map(|error| format!("{label}: {error}"))
        })
        .collect::<Vec<_>>();
    match (test_result, restore_errors.is_empty()) {
        (Ok(()), true) => Ok(()),
        (Ok(()), false) => Err(SlmpError::new(format!(
            "restore failed: {}",
            restore_errors.join("; ")
        ))),
        (Err(error), true) => Err(error),
        (Err(error), false) => Err(SlmpError::new(format!(
            "{error}; restore also failed: {}",
            restore_errors.join("; ")
        ))),
    }
}

fn pack_bit_words(values: &[bool]) -> Vec<u16> {
    values
        .chunks(16)
        .map(|chunk| {
            let mut word = 0u16;
            for (index, value) in chunk.iter().enumerate() {
                if *value {
                    word |= 1 << index;
                }
            }
            word
        })
        .collect()
}

fn format_entry_address(entry: &SlmpDeviceRangeEntry, number: u32) -> String {
    let text = match entry.notation {
        SlmpDeviceRangeNotation::Decimal => number.to_string(),
        SlmpDeviceRangeNotation::Octal => format!("{number:o}"),
        SlmpDeviceRangeNotation::Hexadecimal => format!("{number:X}"),
    };
    format!("{}{}", entry.device, text)
}

fn expect_u16(value: SlmpValue) -> Result<u16, SlmpError> {
    match value {
        SlmpValue::U16(value) => Ok(value),
        other => Err(SlmpError::new(format!("expected U16, got {other:?}"))),
    }
}

fn expect_u32(value: SlmpValue) -> Result<u32, SlmpError> {
    match value {
        SlmpValue::U32(value) => Ok(value),
        other => Err(SlmpError::new(format!("expected U32, got {other:?}"))),
    }
}

fn expect_f32(value: SlmpValue) -> Result<f32, SlmpError> {
    match value {
        SlmpValue::F32(value) => Ok(value),
        other => Err(SlmpError::new(format!("expected F32, got {other:?}"))),
    }
}

fn alternate_u16(original: u16, candidate: u16) -> u16 {
    if original == candidate {
        candidate ^ 0xFFFF
    } else {
        candidate
    }
}

fn alternate_u32(original: u32, candidate: u32) -> u32 {
    if original == candidate {
        candidate ^ 0xFFFF_FFFF
    } else {
        candidate
    }
}

fn apply_family_default_devices(
    mut options: SlmpRouteValidationOptions,
    family: SlmpDeviceRangeFamily,
) -> SlmpRouteValidationOptions {
    if family == SlmpDeviceRangeFamily::IqF {
        if options.word_device == default_word_device() {
            options.word_device = "D1000".to_string();
        }
        if options.dword_device == default_dword_device() {
            options.dword_device = "D1002".to_string();
        }
        if options.float_device == default_float_device() {
            options.float_device = "D1004".to_string();
        }
    }
    options
}

fn expected_range_end_code(family: SlmpDeviceRangeFamily) -> u16 {
    match family {
        SlmpDeviceRangeFamily::IqF => IQF_RANGE_END_CODE,
        _ => DEFAULT_RANGE_END_CODE,
    }
}

fn route_capabilities(family: SlmpDeviceRangeFamily) -> RouteCapabilities {
    match family {
        SlmpDeviceRangeFamily::QCpu
        | SlmpDeviceRangeFamily::LCpu
        | SlmpDeviceRangeFamily::QnU
        | SlmpDeviceRangeFamily::QnUDV => RouteCapabilities {
            block: false,
            random: false,
            lz: false,
        },
        _ => RouteCapabilities {
            block: true,
            random: true,
            lz: true,
        },
    }
}

fn default_word_device() -> String {
    "D9000".to_string()
}

fn default_dword_device() -> String {
    "D9002".to_string()
}

fn default_float_device() -> String {
    "D9004".to_string()
}

fn default_bit_device() -> String {
    "M100".to_string()
}

fn default_lz_device() -> String {
    "LZ0".to_string()
}

fn default_range_error_devices() -> Vec<String> {
    ["X", "Y", "M", "D", "R", "ZR", "RD", "LZ", "SM", "SD"]
        .into_iter()
        .map(ToOwned::to_owned)
        .collect()
}

#[allow(dead_code)]
fn _is_word_like_category(category: SlmpDeviceRangeCategory) -> bool {
    matches!(
        category,
        SlmpDeviceRangeCategory::Word
            | SlmpDeviceRangeCategory::Index
            | SlmpDeviceRangeCategory::FileRefresh
            | SlmpDeviceRangeCategory::TimerCounter
    )
}
