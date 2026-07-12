use crate::address::device_spec_size;
use crate::capability_profiles::{
    self, SlmpProfileFeature, SlmpProfileFeatureState, SlmpProfileLimit,
};
use crate::client_rules as rules;
use crate::device_ranges::{
    SlmpDeviceRangeCatalog,
    build_catalog_for_plc_profile as build_device_range_catalog_for_plc_profile,
    read_registers as read_device_range_registers,
    resolve_profile_for_plc_profile as resolve_device_range_profile_for_plc_profile,
};
use crate::error::{SlmpError, SlmpErrorInfo};
use crate::model::{
    RawSlmpDeviceAddress, SlmpBlockRead, SlmpBlockReadResult, SlmpBlockWrite, SlmpCommand,
    SlmpCompatibilityMode, SlmpConnectionOptions, SlmpCpuOperationState, SlmpDeviceAddress,
    SlmpDeviceCode, SlmpDeviceModification, SlmpExtensionSpec, SlmpFrameType,
    SlmpLabelArrayReadPoint, SlmpLabelArrayReadResult, SlmpLabelArrayWritePoint,
    SlmpLabelRandomReadResult, SlmpLabelRandomWritePoint, SlmpLongTimerResult, SlmpPlcProfile,
    SlmpQualifiedDeviceAddress, SlmpRandomReadResult, SlmpRemoteClearMode, SlmpRemoteMode,
    SlmpTargetAddress, SlmpTrafficStats, SlmpTransportMode, SlmpTypeNameInfo,
};
use socket2::{SockRef, TcpKeepalive};
use std::net::{TcpStream as StdTcpStream, ToSocketAddrs};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::timeout;

const MAX_RUNTIME_RANGE_PROBE_COUNT: u32 = 1_048_576;
const UDP_RECEIVE_BUFFER_SIZE: usize = 65_535;

#[derive(Clone)]
pub struct SlmpClient {
    inner: Arc<Mutex<ClientInner>>,
}

enum Transport {
    Tcp(TcpStream),
    Udp(UdpSocket),
    Closed,
}

struct ClientInner {
    options: SlmpConnectionOptions,
    transport: Transport,
    serial: u16,
    last_request_frame: Vec<u8>,
    last_response_frame: Vec<u8>,
    traffic_stats: SlmpTrafficStats,
}

impl SlmpClient {
    pub async fn connect(options: SlmpConnectionOptions) -> Result<Self, SlmpError> {
        if options.port == 0 {
            return Err(SlmpError::new(
                "port is required and must be in range 1..=65535",
            ));
        }
        if options.timeout.is_zero() {
            return Err(SlmpError::new("timeout must be greater than zero"));
        }
        let transport = match options.transport_mode {
            SlmpTransportMode::Tcp => {
                let stream = connect_tcp_stream(&options).await?;
                Transport::Tcp(stream)
            }
            SlmpTransportMode::Udp => {
                let socket = UdpSocket::bind("0.0.0.0:0").await?;
                socket
                    .connect((options.host.as_str(), options.port))
                    .await?;
                Transport::Udp(socket)
            }
        };
        Ok(Self {
            inner: Arc::new(Mutex::new(ClientInner {
                options,
                transport,
                serial: 0,
                last_request_frame: Vec::new(),
                last_response_frame: Vec::new(),
                traffic_stats: SlmpTrafficStats::default(),
            })),
        })
    }

    pub async fn close(&self) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        if let Transport::Tcp(stream) = &mut inner.transport {
            stream.shutdown().await?;
        }
        inner.transport = Transport::Closed;
        inner.last_request_frame.clear();
        inner.last_response_frame.clear();
        Ok(())
    }

    pub async fn last_request_frame(&self) -> Vec<u8> {
        self.inner.lock().await.last_request_frame.clone()
    }

    pub async fn last_response_frame(&self) -> Vec<u8> {
        self.inner.lock().await.last_response_frame.clone()
    }

    pub async fn traffic_stats(&self) -> SlmpTrafficStats {
        self.inner.lock().await.traffic_stats
    }

    pub async fn plc_profile(&self) -> SlmpPlcProfile {
        self.inner.lock().await.options.plc_profile
    }

    pub async fn read_type_name(&self) -> Result<SlmpTypeNameInfo, SlmpError> {
        self.inner.lock().await.read_type_name().await
    }

    pub async fn read_cpu_operation_state(&self) -> Result<SlmpCpuOperationState, SlmpError> {
        self.inner.lock().await.read_cpu_operation_state().await
    }

    pub async fn read_latest_self_diagnosis_error_code(&self) -> Result<u16, SlmpError> {
        self.inner
            .lock()
            .await
            .read_latest_self_diagnosis_error_code()
            .await
    }

    pub async fn read_device_range_catalog(&self) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
        let plc_profile = self.plc_profile().await;
        let profile = resolve_device_range_profile_for_plc_profile(plc_profile);
        let registers = read_device_range_registers(self, &profile).await?;
        let catalog = build_device_range_catalog_for_plc_profile(plc_profile, &registers)?;
        self.resolve_device_range_runtime_limits(catalog).await
    }

    pub async fn read_device_range_catalog_for_plc_profile(
        &self,
        plc_profile: SlmpPlcProfile,
    ) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
        let profile = resolve_device_range_profile_for_plc_profile(plc_profile);
        let registers = read_device_range_registers(self, &profile).await?;
        let catalog = build_device_range_catalog_for_plc_profile(plc_profile, &registers)?;
        self.resolve_device_range_runtime_limits(catalog).await
    }

    async fn resolve_device_range_runtime_limits(
        &self,
        mut catalog: SlmpDeviceRangeCatalog,
    ) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
        let address_profile = catalog.plc_profile.address_profile();
        if !matches!(
            address_profile,
            SlmpPlcProfile::QCpu
                | SlmpPlcProfile::LCpu
                | SlmpPlcProfile::QnU
                | SlmpPlcProfile::QnUDV
        ) {
            return Ok(catalog);
        }

        if address_profile == SlmpPlcProfile::QCpu {
            let z_count = if self.can_read_one_word(SlmpDeviceCode::Z, 15).await {
                16
            } else {
                10
            };
            catalog = crate::device_ranges::replace_fixed_point_count(
                catalog,
                "Z",
                z_count,
                "Runtime access check",
                "QCPU Z register count is selected by probing Z15.",
            );
        }

        let zr_count = self.resolve_readable_point_count(SlmpDeviceCode::ZR).await;
        catalog = crate::device_ranges::replace_fixed_point_count(
            catalog,
            "ZR",
            zr_count,
            "Runtime access check",
            "ZR register count is selected by probing readable ZR addresses.",
        );
        Ok(crate::device_ranges::replace_fixed_point_count(
            catalog,
            "R",
            zr_count.min(32_768),
            "Runtime access check",
            "R register count matches the probed ZR size and is capped at R32767.",
        ))
    }

    async fn resolve_readable_point_count(&self, device: SlmpDeviceCode) -> u32 {
        if !self.can_read_one_word(device, 0).await {
            return 0;
        }

        let upper_limit = MAX_RUNTIME_RANGE_PROBE_COUNT - 1;
        let mut low = 0;
        let mut high = 1;
        while high < upper_limit && self.can_read_one_word(device, high).await {
            low = high;
            high = ((high * 2) + 1).min(upper_limit);
        }

        if high == upper_limit && self.can_read_one_word(device, high).await {
            return MAX_RUNTIME_RANGE_PROBE_COUNT;
        }

        let mut left = low + 1;
        let mut right = high - 1;
        while left <= right {
            let mid = left + ((right - left) / 2);
            if self.can_read_one_word(device, mid).await {
                low = mid;
                left = mid + 1;
            } else {
                if mid == 0 {
                    break;
                }
                right = mid - 1;
            }
        }

        low + 1
    }

    async fn can_read_one_word(&self, device: SlmpDeviceCode, number: u32) -> bool {
        self.read_words_raw(
            SlmpDeviceAddress::new(device, number, self.plc_profile().await),
            1,
        )
        .await
        .is_ok()
    }

    pub async fn read_words_raw(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.read_words_raw(device, points).await
    }

    pub async fn write_words(
        &self,
        device: SlmpDeviceAddress,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.write_words(device, values).await
    }

    pub async fn read_bits(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<bool>, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.read_bits(device, points).await
    }

    pub async fn write_bits(
        &self,
        device: SlmpDeviceAddress,
        values: &[bool],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.write_bits(device, values).await
    }

    pub async fn read_dwords_raw(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<u32>, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.read_dwords_raw(device, points).await
    }

    pub async fn write_dwords(
        &self,
        device: SlmpDeviceAddress,
        values: &[u32],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.write_dwords(device, values).await
    }

    pub async fn read_float32s(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<f32>, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.read_float32s(device, points).await
    }

    pub async fn write_float32s(
        &self,
        device: SlmpDeviceAddress,
        values: &[f32],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device)?;
        inner.write_float32s(device, values).await
    }

    pub async fn read_words_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        points: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device.device())?;
        inner.read_words_extended(device, points).await
    }

    pub async fn write_words_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device.device())?;
        inner.write_words_extended(device, values).await
    }

    pub async fn read_bits_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        points: u16,
    ) -> Result<Vec<bool>, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device.device())?;
        inner.read_bits_extended(device, points).await
    }

    pub async fn write_bits_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        values: &[bool],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profile(device.device())?;
        inner.write_bits_extended(device, values).await
    }

    pub async fn read_random(
        &self,
        word_devices: &[SlmpDeviceAddress],
        dword_devices: &[SlmpDeviceAddress],
    ) -> Result<SlmpRandomReadResult, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(word_devices.iter().chain(dword_devices))?;
        inner.read_random(word_devices, dword_devices).await
    }

    pub async fn read_random_words(
        &self,
        word_devices: &[SlmpDeviceAddress],
    ) -> Result<Vec<u16>, SlmpError> {
        Ok(self.read_random(word_devices, &[]).await?.word_values)
    }

    pub async fn read_random_dwords(
        &self,
        dword_devices: &[SlmpDeviceAddress],
    ) -> Result<Vec<u32>, SlmpError> {
        Ok(self.read_random(&[], dword_devices).await?.dword_values)
    }

    pub async fn read_random_ext(
        &self,
        word_devices: &[SlmpQualifiedDeviceAddress],
        dword_devices: &[SlmpQualifiedDeviceAddress],
    ) -> Result<SlmpRandomReadResult, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(
            word_devices
                .iter()
                .map(SlmpQualifiedDeviceAddress::device_ref)
                .chain(
                    dword_devices
                        .iter()
                        .map(SlmpQualifiedDeviceAddress::device_ref),
                ),
        )?;
        inner.read_random_ext(word_devices, dword_devices).await
    }

    pub async fn read_random_words_extended(
        &self,
        word_devices: &[SlmpQualifiedDeviceAddress],
    ) -> Result<Vec<u16>, SlmpError> {
        Ok(self.read_random_ext(word_devices, &[]).await?.word_values)
    }

    pub async fn read_random_dwords_extended(
        &self,
        dword_devices: &[SlmpQualifiedDeviceAddress],
    ) -> Result<Vec<u32>, SlmpError> {
        Ok(self.read_random_ext(&[], dword_devices).await?.dword_values)
    }

    pub async fn write_random_words(
        &self,
        word_entries: &[(SlmpDeviceAddress, u16)],
        dword_entries: &[(SlmpDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(
            word_entries
                .iter()
                .map(|(device, _)| device)
                .chain(dword_entries.iter().map(|(device, _)| device)),
        )?;
        inner.write_random_words(word_entries, dword_entries).await
    }

    pub async fn write_random_u16s(
        &self,
        word_entries: &[(SlmpDeviceAddress, u16)],
    ) -> Result<(), SlmpError> {
        self.write_random_words(word_entries, &[]).await
    }

    pub async fn write_random_u32s(
        &self,
        dword_entries: &[(SlmpDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        self.write_random_words(&[], dword_entries).await
    }

    pub async fn write_random_words_ext(
        &self,
        word_entries: &[(SlmpQualifiedDeviceAddress, u16)],
        dword_entries: &[(SlmpQualifiedDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(
            word_entries
                .iter()
                .map(|(device, _)| device.device_ref())
                .chain(dword_entries.iter().map(|(device, _)| device.device_ref())),
        )?;
        inner
            .write_random_words_ext(word_entries, dword_entries)
            .await
    }

    pub async fn write_random_u16s_extended(
        &self,
        word_entries: &[(SlmpQualifiedDeviceAddress, u16)],
    ) -> Result<(), SlmpError> {
        self.write_random_words_ext(word_entries, &[]).await
    }

    pub async fn write_random_u32s_extended(
        &self,
        dword_entries: &[(SlmpQualifiedDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        self.write_random_words_ext(&[], dword_entries).await
    }

    pub async fn write_random_bits(
        &self,
        bit_entries: &[(SlmpDeviceAddress, bool)],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(bit_entries.iter().map(|(device, _)| device))?;
        inner.write_random_bits(bit_entries).await
    }

    pub async fn write_random_bits_ext(
        &self,
        bit_entries: &[(SlmpQualifiedDeviceAddress, bool)],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(bit_entries.iter().map(|(device, _)| device.device_ref()))?;
        inner.write_random_bits_ext(bit_entries).await
    }

    pub async fn read_block(
        &self,
        word_blocks: &[SlmpBlockRead],
        bit_blocks: &[SlmpBlockRead],
    ) -> Result<SlmpBlockReadResult, SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(
            word_blocks
                .iter()
                .map(|block| &block.device)
                .chain(bit_blocks.iter().map(|block| &block.device)),
        )?;
        inner.read_block(word_blocks, bit_blocks).await
    }

    pub async fn read_word_blocks(
        &self,
        word_blocks: &[SlmpBlockRead],
    ) -> Result<SlmpBlockReadResult, SlmpError> {
        self.read_block(word_blocks, &[]).await
    }

    pub async fn read_bit_blocks(
        &self,
        bit_blocks: &[SlmpBlockRead],
    ) -> Result<SlmpBlockReadResult, SlmpError> {
        self.read_block(&[], bit_blocks).await
    }

    pub async fn write_block(
        &self,
        word_blocks: &[SlmpBlockWrite],
        bit_blocks: &[SlmpBlockWrite],
    ) -> Result<(), SlmpError> {
        let mut inner = self.inner.lock().await;
        inner.ensure_address_profiles(
            word_blocks
                .iter()
                .map(|block| &block.device)
                .chain(bit_blocks.iter().map(|block| &block.device)),
        )?;
        inner.write_block(word_blocks, bit_blocks).await
    }

    pub async fn write_word_blocks(&self, word_blocks: &[SlmpBlockWrite]) -> Result<(), SlmpError> {
        self.write_block(word_blocks, &[]).await
    }

    pub async fn write_bit_blocks(&self, bit_blocks: &[SlmpBlockWrite]) -> Result<(), SlmpError> {
        self.write_block(&[], bit_blocks).await
    }

    pub async fn remote_run(
        &self,
        mode: SlmpRemoteMode,
        clear_mode: SlmpRemoteClearMode,
    ) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_run(mode, clear_mode).await
    }

    pub async fn remote_stop(&self) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_stop().await
    }

    pub async fn remote_pause(&self, mode: SlmpRemoteMode) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_pause(mode).await
    }

    pub async fn remote_latch_clear(&self) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_latch_clear().await
    }

    pub async fn remote_reset(&self) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_reset().await
    }

    pub async fn remote_password_unlock(&self, password: &str) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .remote_password_unlock(password)
            .await
    }

    pub async fn remote_password_lock(&self, password: &str) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_password_lock(password).await
    }

    pub async fn self_test_loopback(&self, data: &[u8]) -> Result<Vec<u8>, SlmpError> {
        self.inner.lock().await.self_test_loopback(data).await
    }

    pub async fn memory_read_words(
        &self,
        head_address: u32,
        word_length: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        self.inner
            .lock()
            .await
            .memory_read_words(head_address, word_length)
            .await
    }

    pub async fn memory_write_words(
        &self,
        head_address: u32,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .memory_write_words(head_address, values)
            .await
    }

    pub async fn extend_unit_read_words(
        &self,
        head_address: u32,
        word_length: u16,
        module_no: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        self.inner
            .lock()
            .await
            .extend_unit_read_words(head_address, word_length, module_no)
            .await
    }

    pub async fn extend_unit_write_words(
        &self,
        head_address: u32,
        module_no: u16,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .extend_unit_write_words(head_address, module_no, values)
            .await
    }

    pub async fn read_array_labels(
        &self,
        points: &[SlmpLabelArrayReadPoint],
    ) -> Result<Vec<SlmpLabelArrayReadResult>, SlmpError> {
        self.read_array_labels_with_abbreviations(points, &[]).await
    }

    pub async fn read_array_labels_with_abbreviations(
        &self,
        points: &[SlmpLabelArrayReadPoint],
        abbreviation_labels: &[String],
    ) -> Result<Vec<SlmpLabelArrayReadResult>, SlmpError> {
        self.inner
            .lock()
            .await
            .read_array_labels(points, abbreviation_labels)
            .await
    }

    pub async fn write_array_labels(
        &self,
        points: &[SlmpLabelArrayWritePoint],
    ) -> Result<(), SlmpError> {
        self.write_array_labels_with_abbreviations(points, &[])
            .await
    }

    pub async fn write_array_labels_with_abbreviations(
        &self,
        points: &[SlmpLabelArrayWritePoint],
        abbreviation_labels: &[String],
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .write_array_labels(points, abbreviation_labels)
            .await
    }

    pub async fn read_random_labels(
        &self,
        labels: &[String],
    ) -> Result<Vec<SlmpLabelRandomReadResult>, SlmpError> {
        self.read_random_labels_with_abbreviations(labels, &[])
            .await
    }

    pub async fn read_random_labels_with_abbreviations(
        &self,
        labels: &[String],
        abbreviation_labels: &[String],
    ) -> Result<Vec<SlmpLabelRandomReadResult>, SlmpError> {
        self.inner
            .lock()
            .await
            .read_random_labels(labels, abbreviation_labels)
            .await
    }

    pub async fn write_random_labels(
        &self,
        points: &[SlmpLabelRandomWritePoint],
    ) -> Result<(), SlmpError> {
        self.write_random_labels_with_abbreviations(points, &[])
            .await
    }

    pub async fn write_random_labels_with_abbreviations(
        &self,
        points: &[SlmpLabelRandomWritePoint],
        abbreviation_labels: &[String],
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .write_random_labels(points, abbreviation_labels)
            .await
    }

    pub async fn read_long_timer(
        &self,
        head_no: u32,
        points: usize,
    ) -> Result<Vec<SlmpLongTimerResult>, SlmpError> {
        self.inner
            .lock()
            .await
            .read_long_timer(head_no, points)
            .await
    }

    pub async fn read_long_retentive_timer(
        &self,
        head_no: u32,
        points: usize,
    ) -> Result<Vec<SlmpLongTimerResult>, SlmpError> {
        self.inner
            .lock()
            .await
            .read_long_retentive_timer(head_no, points)
            .await
    }

    pub async fn raw_command(
        &self,
        command: SlmpCommand,
        subcommand: u16,
        payload: &[u8],
    ) -> Result<Vec<u8>, SlmpError> {
        self.inner
            .lock()
            .await
            .request(command, subcommand, payload, true)
            .await
    }
}

async fn connect_tcp_stream(options: &SlmpConnectionOptions) -> Result<TcpStream, SlmpError> {
    let host = options.host.clone();
    let port = options.port;
    let timeout_duration = options.timeout;
    let tcp_keepalive = options.tcp_keepalive;
    let std_stream = task::spawn_blocking(move || {
        let addrs: Vec<_> = (host.as_str(), port).to_socket_addrs()?.collect();
        if addrs.is_empty() {
            return Err(SlmpError::new(format!(
                "tcp connect failed: no socket addresses resolved for {host}:{port}"
            )));
        }

        let mut last_error = None;
        for addr in addrs {
            match StdTcpStream::connect_timeout(&addr, timeout_duration) {
                Ok(stream) => {
                    stream.set_nodelay(true)?;
                    if let Some(keepalive_idle) = tcp_keepalive {
                        configure_tcp_keepalive(&stream, keepalive_idle)?;
                    }
                    stream.set_nonblocking(true)?;
                    return Ok(stream);
                }
                Err(error) => last_error = Some(error),
            }
        }

        Err(last_error
            .map(SlmpError::from)
            .unwrap_or_else(|| SlmpError::new("tcp connect failed")))
    })
    .await
    .map_err(|error| SlmpError::new(format!("tcp connect task failed: {error}")))??;

    TcpStream::from_std(std_stream).map_err(SlmpError::from)
}

fn configure_tcp_keepalive(
    stream: &StdTcpStream,
    idle: std::time::Duration,
) -> Result<(), SlmpError> {
    if idle.is_zero() {
        return Err(SlmpError::new(
            "tcp_keepalive must be greater than zero when enabled",
        ));
    }

    let socket = SockRef::from(stream);
    socket.set_keepalive(true)?;
    socket.set_tcp_keepalive(&TcpKeepalive::new().with_time(idle))?;
    Ok(())
}

impl ClientInner {
    fn ensure_address_profile(&self, device: SlmpDeviceAddress) -> Result<(), SlmpError> {
        let actual = device.plc_profile();
        let expected = self.options.plc_profile;
        if actual != expected {
            return Err(SlmpError::new(format!(
                "device address profile mismatch: address={} client={}",
                actual.canonical_name(),
                expected.canonical_name()
            )));
        }
        Ok(())
    }

    fn ensure_address_profiles<'a>(
        &self,
        devices: impl IntoIterator<Item = &'a SlmpDeviceAddress>,
    ) -> Result<(), SlmpError> {
        for device in devices {
            self.ensure_address_profile(*device)?;
        }
        Ok(())
    }

    fn validate_qualified_random_write_overlap(
        word_entries: &[(SlmpQualifiedDeviceAddress, u16)],
        dword_entries: &[(SlmpQualifiedDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        let mut spans = Vec::with_capacity(word_entries.len() + dword_entries.len());
        spans.extend(word_entries.iter().map(|(device, _)| (*device, 1u32)));
        spans.extend(dword_entries.iter().map(|(device, _)| (*device, 2u32)));
        for (index, (left, left_width)) in spans.iter().enumerate() {
            let left_device = left.device();
            let left_end = left_device
                .number()
                .checked_add(*left_width - 1)
                .ok_or_else(|| SlmpError::new("extended random write device span overflows u32"))?;
            for (right, right_width) in &spans[index + 1..] {
                let right_device = right.device();
                if left_device.plc_profile() != right_device.plc_profile()
                    || left_device.code() != right_device.code()
                    || left.extension_specification() != right.extension_specification()
                    || left.direct_memory_specification() != right.direct_memory_specification()
                    || left.modification() != right.modification()
                {
                    continue;
                }
                let right_end = right_device
                    .number()
                    .checked_add(*right_width - 1)
                    .ok_or_else(|| {
                        SlmpError::new("extended random write device span overflows u32")
                    })?;
                if left_device.number() <= right_end && right_device.number() <= left_end {
                    return Err(SlmpError::new(
                        "extended random write device ranges must not overlap within one request",
                    ));
                }
            }
        }
        Ok(())
    }

    async fn read_type_name(&mut self) -> Result<SlmpTypeNameInfo, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::TypeName)?;
        let payload = self
            .request(SlmpCommand::ReadTypeName, 0x0000, &[], true)
            .await?;
        if payload.len() < 16 {
            return Err(SlmpError::new("read_type_name response too short"));
        }
        let model = String::from_utf8_lossy(&payload[..16])
            .trim_end_matches('\0')
            .trim_end()
            .to_string();
        let (model_code, has_model_code) = if payload.len() >= 18 {
            (u16::from_le_bytes([payload[16], payload[17]]), true)
        } else {
            (0, false)
        };
        Ok(SlmpTypeNameInfo {
            model,
            model_code,
            has_model_code,
        })
    }

    async fn read_cpu_operation_state(&mut self) -> Result<SlmpCpuOperationState, SlmpError> {
        let status_word = self
            .read_words_raw(
                SlmpDeviceAddress::new(SlmpDeviceCode::SD, 203, self.options.plc_profile),
                1,
            )
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| SlmpError::new("read_cpu_operation_state expected one word"))?;
        Ok(rules::decode_cpu_operation_state(status_word))
    }

    async fn read_latest_self_diagnosis_error_code(&mut self) -> Result<u16, SlmpError> {
        self.read_words_raw(
            SlmpDeviceAddress::new(SlmpDeviceCode::SD, 0, self.options.plc_profile),
            1,
        )
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| SlmpError::new("read_latest_self_diagnosis_error_code expected one word"))
    }

    async fn read_words_raw(
        &mut self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            points as usize,
            false,
            false,
            "read_words",
            self.options.plc_profile,
        )?;
        rules::validate_direct_word_read(device, points)?;
        let payload = self.build_read_write_payload(device, points, None, false);
        let sub = self.word_subcommand(false);
        let data = self
            .request(SlmpCommand::DeviceRead, sub, &payload, true)
            .await?;
        if data.len() != points as usize * 2 {
            return Err(SlmpError::new("read_words payload size mismatch"));
        }
        Ok(data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect())
    }

    async fn write_words(
        &mut self,
        device: SlmpDeviceAddress,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            values.len(),
            false,
            true,
            "write_words",
            self.options.plc_profile,
        )?;
        rules::validate_direct_word_write(device, self.options.plc_profile)?;
        let payload =
            self.build_read_write_payload(device, values.len() as u16, Some(values), false);
        let sub = self.word_subcommand(false);
        let _ = self
            .request(SlmpCommand::DeviceWrite, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn read_bits(
        &mut self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<bool>, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            points as usize,
            true,
            false,
            "read_bits",
            self.options.plc_profile,
        )?;
        rules::validate_direct_bit_read(device)?;
        let payload = self.build_read_write_payload(device, points, None, true);
        let data = self
            .request(
                SlmpCommand::DeviceRead,
                self.word_subcommand(true),
                &payload,
                true,
            )
            .await?;
        rules::unpack_bit_values(&data, points as usize)
    }

    async fn write_bits(
        &mut self,
        device: SlmpDeviceAddress,
        values: &[bool],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            values.len(),
            true,
            true,
            "write_bits",
            self.options.plc_profile,
        )?;
        rules::validate_direct_bit_write(device, self.options.plc_profile)?;
        let words: Vec<u16> = values.iter().map(|value| u16::from(*value)).collect();
        let payload =
            self.build_read_write_payload(device, values.len() as u16, Some(&words), true);
        let _ = self
            .request(
                SlmpCommand::DeviceWrite,
                self.word_subcommand(true),
                &payload,
                true,
            )
            .await?;
        Ok(())
    }

    async fn read_dwords_raw(
        &mut self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<u32>, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_dword_read(device)?;
        let word_points = (points as usize) * 2;
        rules::validate_direct_access_points(
            word_points,
            false,
            false,
            "read_dwords",
            self.options.plc_profile,
        )?;
        let words = self.read_words_raw(device, word_points as u16).await?;
        Ok(words
            .chunks_exact(2)
            .map(|chunk| chunk[0] as u32 | ((chunk[1] as u32) << 16))
            .collect())
    }

    async fn write_dwords(
        &mut self,
        device: SlmpDeviceAddress,
        values: &[u32],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_dword_write(device, self.options.plc_profile)?;
        rules::validate_direct_access_points(
            values.len() * 2,
            false,
            true,
            "write_dwords",
            self.options.plc_profile,
        )?;
        let mut words = Vec::with_capacity(values.len() * 2);
        for value in values {
            words.push((value & 0xFFFF) as u16);
            words.push((value >> 16) as u16);
        }
        self.write_words(device, &words).await
    }

    async fn read_float32s(
        &mut self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<f32>, SlmpError> {
        Ok(self
            .read_dwords_raw(device, points)
            .await?
            .into_iter()
            .map(f32::from_bits)
            .collect())
    }

    async fn write_float32s(
        &mut self,
        device: SlmpDeviceAddress,
        values: &[f32],
    ) -> Result<(), SlmpError> {
        let values: Vec<u32> = values.iter().map(|value| value.to_bits()).collect();
        self.write_dwords(device, &values).await
    }

    async fn read_words_extended(
        &mut self,
        device: SlmpQualifiedDeviceAddress,
        points: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            points as usize,
            false,
            false,
            "read_words_ext",
            self.options.plc_profile,
        )?;
        let extension = Self::resolve_effective_extension(device, self.options.plc_profile)?;
        self.ensure_extended_profile_feature_allowed(device, extension)?;
        if !matches!(
            device.device().code(),
            SlmpDeviceCode::G | SlmpDeviceCode::HG
        ) {
            rules::validate_direct_word_read(device.device(), points)?;
        }
        let payload =
            self.build_read_write_payload_extended(device.device(), points, None, extension, false);
        let sub = if extension.direct_memory_specification == 0xF9
            || matches!(
                self.options.compatibility_mode,
                SlmpCompatibilityMode::Legacy
            ) {
            0x0080
        } else {
            0x0082
        };
        let data = self
            .request(SlmpCommand::DeviceRead, sub, &payload, true)
            .await?;
        if data.len() != points as usize * 2 {
            return Err(SlmpError::new("read_words_ext payload size mismatch"));
        }
        Ok(data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect())
    }

    async fn write_words_extended(
        &mut self,
        device: SlmpQualifiedDeviceAddress,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            values.len(),
            false,
            true,
            "write_words_ext",
            self.options.plc_profile,
        )?;
        let extension = Self::resolve_effective_extension(device, self.options.plc_profile)?;
        self.ensure_extended_profile_feature_allowed(device, extension)?;
        if !matches!(
            device.device().code(),
            SlmpDeviceCode::G | SlmpDeviceCode::HG
        ) {
            rules::validate_direct_word_write(device.device(), self.options.plc_profile)?;
        }
        let payload = self.build_read_write_payload_extended(
            device.device(),
            values.len() as u16,
            Some(values),
            extension,
            false,
        );
        let sub = if extension.direct_memory_specification == 0xF9
            || matches!(
                self.options.compatibility_mode,
                SlmpCompatibilityMode::Legacy
            ) {
            0x0080
        } else {
            0x0082
        };
        let _ = self
            .request(SlmpCommand::DeviceWrite, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn read_bits_extended(
        &mut self,
        device: SlmpQualifiedDeviceAddress,
        points: u16,
    ) -> Result<Vec<bool>, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            points as usize,
            true,
            false,
            "read_bits_ext",
            self.options.plc_profile,
        )?;
        let extension = Self::resolve_effective_extension(device, self.options.plc_profile)?;
        self.ensure_extended_profile_feature_allowed(device, extension)?;
        if !matches!(
            device.device().code(),
            SlmpDeviceCode::G | SlmpDeviceCode::HG
        ) {
            rules::validate_direct_bit_read(device.device())?;
        }
        let payload =
            self.build_read_write_payload_extended(device.device(), points, None, extension, true);
        let sub = if extension.direct_memory_specification == 0xF9
            || matches!(
                self.options.compatibility_mode,
                SlmpCompatibilityMode::Legacy
            ) {
            0x0081
        } else {
            0x0083
        };
        let data = self
            .request(SlmpCommand::DeviceRead, sub, &payload, true)
            .await?;
        rules::unpack_bit_values(&data, points as usize)
    }

    async fn write_bits_extended(
        &mut self,
        device: SlmpQualifiedDeviceAddress,
        values: &[bool],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Direct)?;
        rules::validate_direct_access_points(
            values.len(),
            true,
            true,
            "write_bits_ext",
            self.options.plc_profile,
        )?;
        let extension = Self::resolve_effective_extension(device, self.options.plc_profile)?;
        self.ensure_extended_profile_feature_allowed(device, extension)?;
        if !matches!(
            device.device().code(),
            SlmpDeviceCode::G | SlmpDeviceCode::HG
        ) {
            rules::validate_direct_bit_write(device.device(), self.options.plc_profile)?;
        }
        let words: Vec<u16> = values.iter().map(|value| u16::from(*value)).collect();
        let payload = self.build_read_write_payload_extended(
            device.device(),
            values.len() as u16,
            Some(&words),
            extension,
            true,
        );
        let sub = if extension.direct_memory_specification == 0xF9
            || matches!(
                self.options.compatibility_mode,
                SlmpCompatibilityMode::Legacy
            ) {
            0x0081
        } else {
            0x0083
        };
        let _ = self
            .request(SlmpCommand::DeviceWrite, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn read_random(
        &mut self,
        word_devices: &[SlmpDeviceAddress],
        dword_devices: &[SlmpDeviceAddress],
    ) -> Result<SlmpRandomReadResult, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Random)?;
        rules::validate_random_read_devices(word_devices, dword_devices, false)?;
        if word_devices.len() > 0xFF || dword_devices.len() > 0xFF {
            return Err(SlmpError::new("random counts must be <= 255"));
        }
        rules::validate_random_read_like_counts(
            word_devices.len(),
            dword_devices.len(),
            self.options.compatibility_mode,
            self.options.plc_profile,
            SlmpProfileLimit::RandomReadWord,
            "read_random",
        )?;
        let spec_size = device_spec_size(self.options.compatibility_mode);
        let mut payload = vec![word_devices.len() as u8, dword_devices.len() as u8];
        payload.resize(
            2 + (word_devices.len() + dword_devices.len()) * spec_size,
            0,
        );
        let mut offset = 2;
        for device in word_devices {
            offset += self.encode_device_spec(*device, &mut payload[offset..]);
        }
        for device in dword_devices {
            offset += self.encode_device_spec(*device, &mut payload[offset..]);
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0000
        } else {
            0x0002
        };
        let data = self
            .request(SlmpCommand::DeviceReadRandom, sub, &payload, true)
            .await?;
        let expected = word_devices.len() * 2 + dword_devices.len() * 4;
        if data.len() != expected {
            return Err(SlmpError::new(format!(
                "read_random response size mismatch expected={expected} actual={}",
                data.len()
            )));
        }
        let mut cursor = 0;
        let mut result = SlmpRandomReadResult::default();
        for _ in 0..word_devices.len() {
            result
                .word_values
                .push(u16::from_le_bytes([data[cursor], data[cursor + 1]]));
            cursor += 2;
        }
        for _ in 0..dword_devices.len() {
            result.dword_values.push(u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]));
            cursor += 4;
        }
        Ok(result)
    }

    async fn read_random_ext(
        &mut self,
        word_devices: &[SlmpQualifiedDeviceAddress],
        dword_devices: &[SlmpQualifiedDeviceAddress],
    ) -> Result<SlmpRandomReadResult, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Random)?;
        if word_devices.len() > 0xFF || dword_devices.len() > 0xFF {
            return Err(SlmpError::new("random counts must be <= 255"));
        }
        rules::validate_random_read_like_counts(
            word_devices.len(),
            dword_devices.len(),
            self.options.compatibility_mode,
            self.options.plc_profile,
            SlmpProfileLimit::RandomReadWordExt,
            "read_random_ext",
        )?;

        let word_refs: Vec<_> = word_devices.iter().map(|entry| entry.device()).collect();
        let dword_refs: Vec<_> = dword_devices.iter().map(|entry| entry.device()).collect();
        rules::validate_random_read_devices(&word_refs, &dword_refs, true)?;

        let mut payload = vec![word_devices.len() as u8, dword_devices.len() as u8];
        for device in word_devices {
            let extension = Self::resolve_effective_extension(*device, self.options.plc_profile)?;
            self.ensure_extended_profile_feature_allowed(*device, extension)?;
            payload
                .extend_from_slice(&self.encode_extended_device_spec(device.device(), extension));
        }
        for device in dword_devices {
            let extension = Self::resolve_effective_extension(*device, self.options.plc_profile)?;
            self.ensure_extended_profile_feature_allowed(*device, extension)?;
            payload
                .extend_from_slice(&self.encode_extended_device_spec(device.device(), extension));
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0080
        } else {
            0x0082
        };
        let data = self
            .request(SlmpCommand::DeviceReadRandom, sub, &payload, true)
            .await?;
        let expected = word_devices.len() * 2 + dword_devices.len() * 4;
        if data.len() != expected {
            return Err(SlmpError::new(format!(
                "read_random_ext response size mismatch expected={expected} actual={}",
                data.len()
            )));
        }
        let mut cursor = 0;
        let mut result = SlmpRandomReadResult::default();
        for _ in 0..word_devices.len() {
            result
                .word_values
                .push(u16::from_le_bytes([data[cursor], data[cursor + 1]]));
            cursor += 2;
        }
        for _ in 0..dword_devices.len() {
            result.dword_values.push(u32::from_le_bytes([
                data[cursor],
                data[cursor + 1],
                data[cursor + 2],
                data[cursor + 3],
            ]));
            cursor += 4;
        }
        Ok(result)
    }

    async fn write_random_words(
        &mut self,
        word_entries: &[(SlmpDeviceAddress, u16)],
        dword_entries: &[(SlmpDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Random)?;
        rules::validate_random_write_word_devices(
            word_entries,
            dword_entries,
            self.options.plc_profile,
            false,
        )?;
        if word_entries.len() > 0xFF || dword_entries.len() > 0xFF {
            return Err(SlmpError::new("random counts must be <= 255"));
        }
        rules::validate_random_write_word_counts(
            word_entries.len(),
            dword_entries.len(),
            self.options.compatibility_mode,
            self.options.plc_profile,
            SlmpProfileLimit::RandomWriteWord,
            "write_random_words",
        )?;
        let spec_size = device_spec_size(self.options.compatibility_mode);
        let size =
            2 + (word_entries.len() * (spec_size + 2)) + (dword_entries.len() * (spec_size + 4));
        let mut payload = vec![0; size];
        payload[0] = word_entries.len() as u8;
        payload[1] = dword_entries.len() as u8;
        let mut offset = 2;
        for (device, value) in word_entries {
            offset += self.encode_device_spec(*device, &mut payload[offset..]);
            payload[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
            offset += 2;
        }
        for (device, value) in dword_entries {
            offset += self.encode_device_spec(*device, &mut payload[offset..]);
            payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            offset += 4;
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0000
        } else {
            0x0002
        };
        let _ = self
            .request(SlmpCommand::DeviceWriteRandom, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn write_random_words_ext(
        &mut self,
        word_entries: &[(SlmpQualifiedDeviceAddress, u16)],
        dword_entries: &[(SlmpQualifiedDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Random)?;
        if word_entries.len() > 0xFF || dword_entries.len() > 0xFF {
            return Err(SlmpError::new("random counts must be <= 255"));
        }
        rules::validate_random_write_word_counts(
            word_entries.len(),
            dword_entries.len(),
            self.options.compatibility_mode,
            self.options.plc_profile,
            SlmpProfileLimit::RandomWriteWordExt,
            "write_random_words_ext",
        )?;

        let word_refs: Vec<_> = word_entries
            .iter()
            .map(|entry| (entry.0.device(), entry.1))
            .collect();
        let dword_refs: Vec<_> = dword_entries
            .iter()
            .map(|entry| (entry.0.device(), entry.1))
            .collect();
        rules::validate_random_write_word_devices(
            &word_refs,
            &dword_refs,
            self.options.plc_profile,
            true,
        )?;
        Self::validate_qualified_random_write_overlap(word_entries, dword_entries)?;

        let mut payload = vec![word_entries.len() as u8, dword_entries.len() as u8];
        for (device, value) in word_entries {
            let extension = Self::resolve_effective_extension(*device, self.options.plc_profile)?;
            self.ensure_extended_profile_feature_allowed(*device, extension)?;
            payload
                .extend_from_slice(&self.encode_extended_device_spec(device.device(), extension));
            payload.extend_from_slice(&value.to_le_bytes());
        }
        for (device, value) in dword_entries {
            let extension = Self::resolve_effective_extension(*device, self.options.plc_profile)?;
            self.ensure_extended_profile_feature_allowed(*device, extension)?;
            payload
                .extend_from_slice(&self.encode_extended_device_spec(device.device(), extension));
            payload.extend_from_slice(&value.to_le_bytes());
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0080
        } else {
            0x0082
        };
        let _ = self
            .request(SlmpCommand::DeviceWriteRandom, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn write_random_bits(
        &mut self,
        bit_entries: &[(SlmpDeviceAddress, bool)],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Random)?;
        if bit_entries.len() > 0xFF {
            return Err(SlmpError::new("random bit count must be <= 255"));
        }
        rules::validate_random_bit_write_count(
            bit_entries.len(),
            self.options.compatibility_mode,
            self.options.plc_profile,
            SlmpProfileLimit::RandomWriteBit,
            "write_random_bits",
        )?;
        rules::validate_random_bit_write_devices(bit_entries, self.options.plc_profile, true)?;
        let spec_size = device_spec_size(self.options.compatibility_mode);
        let bit_value_size = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            1
        } else {
            2
        };
        let mut payload = vec![0; 1 + (bit_entries.len() * (spec_size + bit_value_size))];
        payload[0] = bit_entries.len() as u8;
        let mut offset = 1;
        for (device, value) in bit_entries {
            offset += self.encode_device_spec(*device, &mut payload[offset..]);
            if matches!(
                self.options.compatibility_mode,
                SlmpCompatibilityMode::Legacy
            ) {
                payload[offset] = u8::from(*value);
                offset += 1;
            } else {
                let raw = u16::from(*value).to_le_bytes();
                payload[offset..offset + 2].copy_from_slice(&raw);
                offset += 2;
            }
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0001
        } else {
            0x0003
        };
        let _ = self
            .request(SlmpCommand::DeviceWriteRandom, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn write_random_bits_ext(
        &mut self,
        bit_entries: &[(SlmpQualifiedDeviceAddress, bool)],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Random)?;
        if bit_entries.len() > 0xFF {
            return Err(SlmpError::new("random bit count must be <= 255"));
        }
        rules::validate_random_bit_write_count(
            bit_entries.len(),
            self.options.compatibility_mode,
            self.options.plc_profile,
            SlmpProfileLimit::RandomWriteBitExt,
            "write_random_bits_ext",
        )?;
        let bit_refs: Vec<_> = bit_entries
            .iter()
            .map(|entry| (entry.0.device(), entry.1))
            .collect();
        rules::validate_random_bit_write_devices(&bit_refs, self.options.plc_profile, false)?;
        let mut seen = std::collections::HashSet::new();
        for (device, _) in bit_entries {
            if !seen.insert(*device) {
                return Err(SlmpError::new(
                    "extended random bit write devices must not be duplicated within one request",
                ));
            }
        }

        let bit_value_size = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            1
        } else {
            2
        };
        let mut payload = Vec::with_capacity(bit_entries.len() * (13 + bit_value_size) + 1);
        payload.push(bit_entries.len() as u8);
        for (device, value) in bit_entries {
            let extension = Self::resolve_effective_extension(*device, self.options.plc_profile)?;
            self.ensure_extended_profile_feature_allowed(*device, extension)?;
            payload
                .extend_from_slice(&self.encode_extended_device_spec(device.device(), extension));
            if matches!(
                self.options.compatibility_mode,
                SlmpCompatibilityMode::Legacy
            ) {
                payload.push(u8::from(*value));
            } else {
                payload.extend_from_slice(&u16::from(*value).to_le_bytes());
            }
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0081
        } else {
            0x0083
        };
        let _ = self
            .request(SlmpCommand::DeviceWriteRandom, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn read_block(
        &mut self,
        word_blocks: &[SlmpBlockRead],
        bit_blocks: &[SlmpBlockRead],
    ) -> Result<SlmpBlockReadResult, SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Block)?;
        rules::validate_block_route_for_profile(self.options.plc_profile, "Read Block (0x0406)")?;
        rules::validate_no_lcs_lcc_block_read(word_blocks, bit_blocks)?;
        if word_blocks.len() > 0xFF || bit_blocks.len() > 0xFF {
            return Err(SlmpError::new("block counts must be <= 255"));
        }
        rules::validate_block_read_limits(
            word_blocks,
            bit_blocks,
            self.options.compatibility_mode,
        )?;
        let spec_size = device_spec_size(self.options.compatibility_mode);
        let total_word_points: usize = word_blocks.iter().map(|block| block.points as usize).sum();
        let total_bit_points: usize = bit_blocks.iter().map(|block| block.points as usize).sum();
        let mut payload = vec![0; 2 + ((word_blocks.len() + bit_blocks.len()) * (spec_size + 2))];
        payload[0] = word_blocks.len() as u8;
        payload[1] = bit_blocks.len() as u8;
        let mut offset = 2;
        for block in word_blocks {
            offset += self.encode_device_spec(block.device, &mut payload[offset..]);
            payload[offset..offset + 2].copy_from_slice(&block.points.to_le_bytes());
            offset += 2;
        }
        for block in bit_blocks {
            offset += self.encode_device_spec(block.device, &mut payload[offset..]);
            payload[offset..offset + 2].copy_from_slice(&block.points.to_le_bytes());
            offset += 2;
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0000
        } else {
            0x0002
        };
        let data = self
            .request(SlmpCommand::DeviceReadBlock, sub, &payload, true)
            .await?;
        let expected = (total_word_points + total_bit_points) * 2;
        if data.len() != expected {
            return Err(SlmpError::new(format!(
                "read_block response size mismatch expected={expected} actual={}",
                data.len()
            )));
        }
        let mut cursor = 0;
        let mut result = SlmpBlockReadResult::default();
        for _ in 0..total_word_points {
            result
                .word_values
                .push(u16::from_le_bytes([data[cursor], data[cursor + 1]]));
            cursor += 2;
        }
        for _ in 0..total_bit_points {
            result
                .bit_values
                .push(u16::from_le_bytes([data[cursor], data[cursor + 1]]));
            cursor += 2;
        }
        Ok(result)
    }

    async fn write_block(
        &mut self,
        word_blocks: &[SlmpBlockWrite],
        bit_blocks: &[SlmpBlockWrite],
    ) -> Result<(), SlmpError> {
        self.ensure_profile_feature_allowed(SlmpProfileFeature::Block)?;
        rules::validate_block_route_for_profile(self.options.plc_profile, "Write Block (0x1406)")?;
        rules::validate_no_lcs_lcc_block_write(word_blocks, bit_blocks, self.options.plc_profile)?;
        self.write_block_once(word_blocks, bit_blocks).await
    }

    async fn write_block_once(
        &mut self,
        word_blocks: &[SlmpBlockWrite],
        bit_blocks: &[SlmpBlockWrite],
    ) -> Result<(), SlmpError> {
        if word_blocks.len() > 0xFF || bit_blocks.len() > 0xFF {
            return Err(SlmpError::new("block counts must be <= 255"));
        }
        rules::validate_block_write_limits(
            word_blocks,
            bit_blocks,
            self.options.compatibility_mode,
        )?;
        let spec_size = device_spec_size(self.options.compatibility_mode);
        let total_word_points: usize = word_blocks.iter().map(|block| block.values.len()).sum();
        let total_bit_points: usize = bit_blocks.iter().map(|block| block.values.len()).sum();
        let mut payload = vec![
            0;
            2 + ((word_blocks.len() + bit_blocks.len()) * (spec_size + 2))
                + ((total_word_points + total_bit_points) * 2)
        ];
        payload[0] = word_blocks.len() as u8;
        payload[1] = bit_blocks.len() as u8;
        // Each block's write data follows that block's own spec (SLMP
        // reference manual Write Block request format); data must not be
        // batched after the block specs, or multi-block/mixed requests
        // misparse on the PLC.
        let mut offset = 2;
        for block in word_blocks.iter().chain(bit_blocks.iter()) {
            offset += self.encode_device_spec(block.device, &mut payload[offset..]);
            let count = block.values.len() as u16;
            payload[offset..offset + 2].copy_from_slice(&count.to_le_bytes());
            offset += 2;
            for value in &block.values {
                payload[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                offset += 2;
            }
        }
        let sub = if matches!(
            self.options.compatibility_mode,
            SlmpCompatibilityMode::Legacy
        ) {
            0x0000
        } else {
            0x0002
        };
        self.request(SlmpCommand::DeviceWriteBlock, sub, &payload, true)
            .await?;
        Ok(())
    }

    async fn remote_run(
        &mut self,
        mode: SlmpRemoteMode,
        clear_mode: SlmpRemoteClearMode,
    ) -> Result<(), SlmpError> {
        let mode = mode.wire_value();
        let clear_mode = clear_mode.wire_value();
        let payload = [
            mode as u8,
            (mode >> 8) as u8,
            clear_mode as u8,
            (clear_mode >> 8) as u8,
        ];
        self.request(SlmpCommand::RemoteRun, 0x0000, &payload, true)
            .await?;
        Ok(())
    }

    async fn remote_stop(&mut self) -> Result<(), SlmpError> {
        self.request(SlmpCommand::RemoteStop, 0x0000, &[0x01, 0x00], true)
            .await?;
        Ok(())
    }

    async fn remote_pause(&mut self, mode: SlmpRemoteMode) -> Result<(), SlmpError> {
        let mode = mode.wire_value().to_le_bytes();
        self.request(SlmpCommand::RemotePause, 0x0000, &mode, true)
            .await?;
        Ok(())
    }

    async fn remote_latch_clear(&mut self) -> Result<(), SlmpError> {
        self.request(SlmpCommand::RemoteLatchClear, 0x0000, &[0x01, 0x00], true)
            .await?;
        Ok(())
    }

    async fn remote_reset(&mut self) -> Result<(), SlmpError> {
        self.request(SlmpCommand::RemoteReset, 0x0000, &[0x01, 0x00], false)
            .await?;
        Ok(())
    }

    async fn remote_password_unlock(&mut self, password: &str) -> Result<(), SlmpError> {
        let payload = self.encode_password(password)?;
        self.request(SlmpCommand::RemotePasswordUnlock, 0x0000, &payload, true)
            .await?;
        Ok(())
    }

    async fn remote_password_lock(&mut self, password: &str) -> Result<(), SlmpError> {
        let payload = self.encode_password(password)?;
        self.request(SlmpCommand::RemotePasswordLock, 0x0000, &payload, true)
            .await?;
        Ok(())
    }

    async fn self_test_loopback(&mut self, data: &[u8]) -> Result<Vec<u8>, SlmpError> {
        if data.is_empty() || data.len() > 960 {
            return Err(SlmpError::new(
                "loopback payload size out of range (1..960 bytes)",
            ));
        }
        if data
            .iter()
            .any(|value| !matches!(*value, b'0'..=b'9' | b'A'..=b'F'))
        {
            return Err(SlmpError::new(
                "loopback payload must contain only ASCII 0-9/A-F bytes",
            ));
        }
        let mut payload = Vec::with_capacity(2 + data.len());
        payload.extend_from_slice(&(data.len() as u16).to_le_bytes());
        payload.extend_from_slice(data);
        let response = self
            .request(SlmpCommand::SelfTest, 0x0000, &payload, true)
            .await?;
        if response.len() < 2 {
            return Err(SlmpError::new("self_test response too short"));
        }
        let length = u16::from_le_bytes([response[0], response[1]]) as usize;
        if response.len() < length + 2 {
            return Err(SlmpError::new("self_test response length mismatch"));
        }
        Ok(response[2..2 + length].to_vec())
    }

    async fn memory_read_words(
        &mut self,
        head_address: u32,
        word_length: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        rules::validate_memory_word_length(word_length as usize, "memory_read")?;
        let mut payload = Vec::with_capacity(6);
        payload.extend_from_slice(&head_address.to_le_bytes());
        payload.extend_from_slice(&word_length.to_le_bytes());
        let data = self
            .request(SlmpCommand::MemoryRead, 0x0000, &payload, true)
            .await?;
        if data.len() != word_length as usize * 2 {
            return Err(SlmpError::new("memory_read response size mismatch"));
        }
        Ok(data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect())
    }

    async fn memory_write_words(
        &mut self,
        head_address: u32,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        rules::validate_memory_word_length(values.len(), "memory_write")?;
        let mut payload = Vec::with_capacity(6 + (values.len() * 2));
        payload.extend_from_slice(&head_address.to_le_bytes());
        payload.extend_from_slice(&(values.len() as u16).to_le_bytes());
        for value in values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        self.request(SlmpCommand::MemoryWrite, 0x0000, &payload, true)
            .await?;
        Ok(())
    }

    async fn extend_unit_read_bytes(
        &mut self,
        head_address: u32,
        byte_length: u16,
        module_no: u16,
    ) -> Result<Vec<u8>, SlmpError> {
        rules::validate_extend_unit_byte_length(byte_length as usize, "extend_unit_read")?;
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&head_address.to_le_bytes());
        payload.extend_from_slice(&byte_length.to_le_bytes());
        payload.extend_from_slice(&module_no.to_le_bytes());
        let data = self
            .request(SlmpCommand::ExtendUnitRead, 0x0000, &payload, true)
            .await?;
        if data.len() != byte_length as usize {
            return Err(SlmpError::new("extend_unit_read response size mismatch"));
        }
        Ok(data)
    }

    async fn extend_unit_read_words(
        &mut self,
        head_address: u32,
        word_length: u16,
        module_no: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        rules::validate_extend_unit_word_length(word_length as usize, "extend_unit_read_words")?;
        let data = self
            .extend_unit_read_bytes(head_address, word_length * 2, module_no)
            .await?;
        Ok(data
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect())
    }

    async fn extend_unit_write_words(
        &mut self,
        head_address: u32,
        module_no: u16,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        rules::validate_extend_unit_word_length(values.len(), "extend_unit_write_words")?;
        let mut payload = Vec::with_capacity(8 + values.len() * 2);
        payload.extend_from_slice(&head_address.to_le_bytes());
        payload.extend_from_slice(&((values.len() * 2) as u16).to_le_bytes());
        payload.extend_from_slice(&module_no.to_le_bytes());
        for value in values {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        self.request(SlmpCommand::ExtendUnitWrite, 0x0000, &payload, true)
            .await?;
        Ok(())
    }

    async fn read_array_labels(
        &mut self,
        points: &[SlmpLabelArrayReadPoint],
        abbreviation_labels: &[String],
    ) -> Result<Vec<SlmpLabelArrayReadResult>, SlmpError> {
        let payload = Self::build_label_array_read_payload(points, abbreviation_labels)?;
        let data = self
            .request(SlmpCommand::LabelArrayRead, 0x0000, &payload, true)
            .await?;
        Self::parse_array_label_read_response(&data, points.len())
    }

    async fn write_array_labels(
        &mut self,
        points: &[SlmpLabelArrayWritePoint],
        abbreviation_labels: &[String],
    ) -> Result<(), SlmpError> {
        let payload = Self::build_label_array_write_payload(points, abbreviation_labels)?;
        self.request(SlmpCommand::LabelArrayWrite, 0x0000, &payload, true)
            .await?;
        Ok(())
    }

    async fn read_random_labels(
        &mut self,
        labels: &[String],
        abbreviation_labels: &[String],
    ) -> Result<Vec<SlmpLabelRandomReadResult>, SlmpError> {
        let payload = Self::build_label_random_read_payload(labels, abbreviation_labels)?;
        let data = self
            .request(SlmpCommand::LabelReadRandom, 0x0000, &payload, true)
            .await?;
        Self::parse_label_random_read_response(&data, labels.len())
    }

    async fn write_random_labels(
        &mut self,
        points: &[SlmpLabelRandomWritePoint],
        abbreviation_labels: &[String],
    ) -> Result<(), SlmpError> {
        let payload = Self::build_label_random_write_payload(points, abbreviation_labels)?;
        self.request(SlmpCommand::LabelWriteRandom, 0x0000, &payload, true)
            .await?;
        Ok(())
    }

    fn build_label_array_read_payload(
        points: &[SlmpLabelArrayReadPoint],
        abbreviation_labels: &[String],
    ) -> Result<Vec<u8>, SlmpError> {
        rules::validate_non_empty_u16_count(points.len(), "array label points")?;
        rules::validate_u16_count(abbreviation_labels.len(), "abbreviation labels")?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&(points.len() as u16).to_le_bytes());
        payload.extend_from_slice(&(abbreviation_labels.len() as u16).to_le_bytes());
        for label in abbreviation_labels {
            Self::append_label_name(&mut payload, label)?;
        }
        for point in points {
            Self::validate_abbreviation_references(&point.label, abbreviation_labels.len())?;
            Self::append_label_name(&mut payload, &point.label)?;
            Self::label_array_data_bytes(point.unit_specification, point.array_data_length)?;
            payload.push(point.unit_specification);
            payload.push(0x00);
            payload.extend_from_slice(&point.array_data_length.to_le_bytes());
        }
        Ok(payload)
    }

    fn build_label_array_write_payload(
        points: &[SlmpLabelArrayWritePoint],
        abbreviation_labels: &[String],
    ) -> Result<Vec<u8>, SlmpError> {
        rules::validate_non_empty_u16_count(points.len(), "array label points")?;
        rules::validate_u16_count(abbreviation_labels.len(), "abbreviation labels")?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&(points.len() as u16).to_le_bytes());
        payload.extend_from_slice(&(abbreviation_labels.len() as u16).to_le_bytes());
        for label in abbreviation_labels {
            Self::append_label_name(&mut payload, label)?;
        }
        for point in points {
            Self::validate_abbreviation_references(&point.label, abbreviation_labels.len())?;
            let expected =
                Self::label_array_data_bytes(point.unit_specification, point.array_data_length)?;
            if point.data.len() != expected {
                return Err(SlmpError::new(format!(
                    "array label write data size mismatch: expected={expected} actual={}",
                    point.data.len()
                )));
            }
            Self::append_label_name(&mut payload, &point.label)?;
            payload.push(point.unit_specification);
            payload.push(0x00);
            payload.extend_from_slice(&point.array_data_length.to_le_bytes());
            payload.extend_from_slice(&point.data);
        }
        Ok(payload)
    }

    fn build_label_random_read_payload(
        labels: &[String],
        abbreviation_labels: &[String],
    ) -> Result<Vec<u8>, SlmpError> {
        rules::validate_non_empty_u16_count(labels.len(), "labels")?;
        rules::validate_u16_count(abbreviation_labels.len(), "abbreviation labels")?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&(labels.len() as u16).to_le_bytes());
        payload.extend_from_slice(&(abbreviation_labels.len() as u16).to_le_bytes());
        for label in abbreviation_labels {
            Self::append_label_name(&mut payload, label)?;
        }
        for label in labels {
            Self::validate_abbreviation_references(label, abbreviation_labels.len())?;
            Self::append_label_name(&mut payload, label)?;
        }
        Ok(payload)
    }

    fn build_label_random_write_payload(
        points: &[SlmpLabelRandomWritePoint],
        abbreviation_labels: &[String],
    ) -> Result<Vec<u8>, SlmpError> {
        rules::validate_non_empty_u16_count(points.len(), "random label points")?;
        rules::validate_u16_count(abbreviation_labels.len(), "abbreviation labels")?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&(points.len() as u16).to_le_bytes());
        payload.extend_from_slice(&(abbreviation_labels.len() as u16).to_le_bytes());
        for label in abbreviation_labels {
            Self::append_label_name(&mut payload, label)?;
        }
        for point in points {
            Self::validate_abbreviation_references(&point.label, abbreviation_labels.len())?;
            rules::validate_u16_count(point.data.len(), "write data length")?;
            Self::append_label_name(&mut payload, &point.label)?;
            payload.extend_from_slice(&(point.data.len() as u16).to_le_bytes());
            payload.extend_from_slice(&point.data);
        }
        Ok(payload)
    }

    fn parse_array_label_read_response(
        data: &[u8],
        expected_points: usize,
    ) -> Result<Vec<SlmpLabelArrayReadResult>, SlmpError> {
        if data.len() < 2 {
            return Err(SlmpError::new("array label read response too short"));
        }
        let count = u16::from_le_bytes([data[0], data[1]]) as usize;
        if count != expected_points {
            return Err(SlmpError::new(format!(
                "array label read point count mismatch: expected={expected_points} actual={count}"
            )));
        }
        let mut offset = 2usize;
        let mut results = Vec::with_capacity(count);
        for _ in 0..count {
            if offset + 4 > data.len() {
                return Err(SlmpError::new(
                    "array label read response truncated before metadata",
                ));
            }
            let data_type_id = data[offset];
            let unit_specification = data[offset + 1];
            let array_data_length = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            offset += 4;
            let data_size = Self::label_array_data_bytes(unit_specification, array_data_length)?;
            if offset + data_size > data.len() {
                return Err(SlmpError::new(
                    "array label read response truncated in data payload",
                ));
            }
            results.push(SlmpLabelArrayReadResult {
                data_type_id,
                unit_specification,
                array_data_length,
                data: data[offset..offset + data_size].to_vec(),
            });
            offset += data_size;
        }
        if offset != data.len() {
            return Err(SlmpError::new(
                "array label read response has trailing bytes",
            ));
        }
        Ok(results)
    }

    fn parse_label_random_read_response(
        data: &[u8],
        expected_points: usize,
    ) -> Result<Vec<SlmpLabelRandomReadResult>, SlmpError> {
        if data.len() < 2 {
            return Err(SlmpError::new("label random read response too short"));
        }
        let count = u16::from_le_bytes([data[0], data[1]]) as usize;
        if count != expected_points {
            return Err(SlmpError::new(format!(
                "label random read point count mismatch: expected={expected_points} actual={count}"
            )));
        }
        let mut offset = 2usize;
        let mut results = Vec::with_capacity(count);
        for _ in 0..count {
            if offset + 4 > data.len() {
                return Err(SlmpError::new(
                    "label random read response truncated before metadata",
                ));
            }
            let data_type_id = data[offset];
            let spare = data[offset + 1];
            let read_data_length = u16::from_le_bytes([data[offset + 2], data[offset + 3]]);
            offset += 4;
            let data_size = read_data_length as usize;
            if offset + data_size > data.len() {
                return Err(SlmpError::new(
                    "label random read response truncated in data payload",
                ));
            }
            results.push(SlmpLabelRandomReadResult {
                data_type_id,
                spare,
                read_data_length,
                data: data[offset..offset + data_size].to_vec(),
            });
            offset += data_size;
        }
        if offset != data.len() {
            return Err(SlmpError::new(
                "label random read response has trailing bytes",
            ));
        }
        Ok(results)
    }

    fn append_label_name(payload: &mut Vec<u8>, label: &str) -> Result<(), SlmpError> {
        if label.trim().is_empty() {
            return Err(SlmpError::new("label must not be empty"));
        }
        let utf16: Vec<u16> = label.encode_utf16().collect();
        rules::validate_u16_count(utf16.len(), "label name length")?;
        payload.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
        for ch in utf16 {
            payload.extend_from_slice(&ch.to_le_bytes());
        }
        Ok(())
    }

    fn validate_abbreviation_references(
        label: &str,
        abbreviation_count: usize,
    ) -> Result<(), SlmpError> {
        if label.trim().is_empty() {
            return Err(SlmpError::new("label must not be empty"));
        }
        let bytes = label.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            if bytes[index] != b'%' {
                index += 1;
                continue;
            }
            let digit_start = index + 1;
            let mut digit_end = digit_start;
            let mut reference = 0usize;
            while digit_end < bytes.len() && bytes[digit_end].is_ascii_digit() {
                reference = reference
                    .checked_mul(10)
                    .and_then(|value| value.checked_add((bytes[digit_end] - b'0') as usize))
                    .ok_or_else(|| SlmpError::new("abbreviation reference is too large"))?;
                digit_end += 1;
            }
            if digit_end == digit_start || reference == 0 || reference > abbreviation_count {
                return Err(SlmpError::new(format!(
                    "label contains an invalid abbreviation reference; use %1 through %{abbreviation_count}"
                )));
            }
            index = digit_end;
        }
        Ok(())
    }

    fn label_array_data_bytes(
        unit_specification: u8,
        array_data_length: u16,
    ) -> Result<usize, SlmpError> {
        match unit_specification {
            0 => Ok(array_data_length as usize * 2),
            1 => Ok(array_data_length as usize),
            other => Err(SlmpError::new(format!(
                "unit_specification must be 0(word) or 1(byte): {other}"
            ))),
        }
    }

    async fn read_long_timer(
        &mut self,
        head_no: u32,
        points: usize,
    ) -> Result<Vec<SlmpLongTimerResult>, SlmpError> {
        let word_points = Self::long_timer_word_points(points)?;
        let words = self
            .read_words_raw(
                SlmpDeviceAddress::new(SlmpDeviceCode::LTN, head_no, self.options.plc_profile),
                word_points,
            )
            .await?;
        Ok(rules::parse_long_timer_words(&words, head_no, "LTN"))
    }

    async fn read_long_retentive_timer(
        &mut self,
        head_no: u32,
        points: usize,
    ) -> Result<Vec<SlmpLongTimerResult>, SlmpError> {
        let word_points = Self::long_timer_word_points(points)?;
        let words = self
            .read_words_raw(
                SlmpDeviceAddress::new(SlmpDeviceCode::LSTN, head_no, self.options.plc_profile),
                word_points,
            )
            .await?;
        Ok(rules::parse_long_timer_words(&words, head_no, "LSTN"))
    }

    fn long_timer_word_points(points: usize) -> Result<u16, SlmpError> {
        let word_points = points
            .checked_mul(4)
            .and_then(|value| u16::try_from(value).ok())
            .ok_or_else(|| {
                SlmpError::new("long timer points exceed the one-request word-count field")
            })?;
        if word_points == 0 {
            return Err(SlmpError::new("long timer points must be at least 1"));
        }
        Ok(word_points)
    }

    fn ensure_profile_feature_allowed(&self, feature: SlmpProfileFeature) -> Result<(), SlmpError> {
        if !self.options.strict_profile {
            return Ok(());
        }
        let Some(capability_feature) =
            capability_profiles::profile_feature(self.options.plc_profile, feature)
        else {
            return Ok(());
        };
        if !matches!(
            capability_feature.state,
            SlmpProfileFeatureState::Blocked | SlmpProfileFeatureState::Unverified
        ) {
            return Ok(());
        }

        Err(SlmpError::profile_feature(
            self.options.plc_profile.canonical_name(),
            capability_profiles::feature_key(feature),
            capability_profiles::state_name(capability_feature.state),
            Some(capability_profiles::feature_evidence(*capability_feature)),
        ))
    }

    fn ensure_extended_profile_feature_allowed(
        &self,
        device: SlmpQualifiedDeviceAddress,
        extension: SlmpExtensionSpec,
    ) -> Result<(), SlmpError> {
        if extension.direct_memory_specification == 0xF9 {
            return self.ensure_profile_feature_allowed(SlmpProfileFeature::ExtLinkDirect);
        }
        if matches!(device.device().code(), SlmpDeviceCode::HG)
            || extension.direct_memory_specification == 0xFA
        {
            return self.ensure_profile_feature_allowed(SlmpProfileFeature::HgCpuBuffer);
        }
        if matches!(device.device().code(), SlmpDeviceCode::G)
            || extension.direct_memory_specification == 0xF8
        {
            return self.ensure_profile_feature_allowed(SlmpProfileFeature::ExtModuleAccess);
        }
        Ok(())
    }

    async fn request(
        &mut self,
        command: SlmpCommand,
        subcommand: u16,
        payload: &[u8],
        expect_response: bool,
    ) -> Result<Vec<u8>, SlmpError> {
        self.validate_request_payload(command, subcommand, payload)?;
        self.build_request_frame(command, subcommand, payload)?;
        let expected_serial = if matches!(self.options.frame_type, SlmpFrameType::Frame4E) {
            Some(u16::from_le_bytes([
                self.last_request_frame[2],
                self.last_request_frame[3],
            ]))
        } else {
            None
        };
        let tx_len = self.last_request_frame.len() as u64;

        // Keep the client in Closed while an exchange is in flight. If this future is
        // externally cancelled, the local socket is dropped and cannot leak a partial or
        // delayed response into a later request. Restore it only after a complete exchange.
        let transport = std::mem::replace(&mut self.transport, Transport::Closed);
        match transport {
            Transport::Tcp(mut stream) => {
                let io_result = async {
                    timeout(
                        self.options.timeout,
                        stream.write_all(&self.last_request_frame),
                    )
                    .await
                    .map_err(|_| SlmpError::new("tcp write timed out"))??;
                    self.traffic_stats.request_count += 1;
                    self.traffic_stats.tx_bytes += tx_len;
                    if !expect_response {
                        self.last_response_frame.clear();
                        return Ok::<(), SlmpError>(());
                    }
                    loop {
                        Self::receive_tcp_frame(
                            &mut stream,
                            self.options.frame_type,
                            self.options.timeout,
                            &mut self.last_response_frame,
                        )
                        .await?;
                        self.traffic_stats.rx_bytes += self.last_response_frame.len() as u64;
                        if !Self::has_expected_response_frame_type(
                            self.options.frame_type,
                            &self.last_response_frame,
                        ) {
                            return Err(SlmpError::new("unexpected response frame type"));
                        }
                        if Self::has_expected_response_serial(
                            &self.last_response_frame,
                            expected_serial,
                        ) {
                            break;
                        }
                    }
                    Ok(())
                }
                .await;
                if let Err(error) = io_result {
                    let _ = stream.shutdown().await;
                    return Err(error);
                }
                if !expect_response {
                    let _ = stream.shutdown().await;
                    return Ok(Vec::new());
                }
                self.transport = Transport::Tcp(stream);
                Self::parse_response(command, subcommand, &self.last_response_frame)
            }
            Transport::Udp(socket) => {
                let io_result = async {
                    timeout(self.options.timeout, socket.send(&self.last_request_frame))
                        .await
                        .map_err(|_| SlmpError::new("udp send timed out"))??;
                    self.traffic_stats.request_count += 1;
                    self.traffic_stats.tx_bytes += tx_len;
                    if !expect_response {
                        self.last_response_frame.clear();
                        return Ok::<(), SlmpError>(());
                    }
                    // A failed UDP exchange invalidates the socket. Closing it prevents a delayed
                    // datagram from being consumed as the response to a later 3E request.
                    loop {
                        self.last_response_frame.resize(UDP_RECEIVE_BUFFER_SIZE, 0);
                        let received = timeout(
                            self.options.timeout,
                            socket.recv(&mut self.last_response_frame),
                        )
                        .await
                        .map_err(|_| SlmpError::new("udp receive timed out"))??;
                        self.last_response_frame.truncate(received);
                        self.traffic_stats.rx_bytes += self.last_response_frame.len() as u64;
                        if !Self::has_expected_response_frame_type(
                            self.options.frame_type,
                            &self.last_response_frame,
                        ) {
                            return Err(SlmpError::new("unexpected response frame type"));
                        }
                        if Self::has_expected_response_serial(
                            &self.last_response_frame,
                            expected_serial,
                        ) {
                            break;
                        }
                    }
                    Ok(())
                }
                .await;
                io_result?;
                if !expect_response {
                    return Ok(Vec::new());
                }
                self.transport = Transport::Udp(socket);
                Self::parse_response(command, subcommand, &self.last_response_frame)
            }
            Transport::Closed => Err(SlmpError::new(
                "transport is closed after a previous transport failure",
            )),
        }
    }

    fn validate_request_payload(
        &self,
        command: SlmpCommand,
        subcommand: u16,
        payload: &[u8],
    ) -> Result<(), SlmpError> {
        Self::request_data_length(command, subcommand, payload.len())?;
        if matches!(command, SlmpCommand::MonitorRegister) && matches!(subcommand, 0x0000 | 0x0002)
        {
            self.validate_plain_monitor_register_payload(self.options.compatibility_mode, payload)?;
        }
        Ok(())
    }

    fn request_data_length(
        command: SlmpCommand,
        subcommand: u16,
        payload_len: usize,
    ) -> Result<u16, SlmpError> {
        let total = payload_len.checked_add(6).ok_or_else(|| {
            SlmpError::with_context(
                format!("request data length overflow: payload={payload_len}"),
                None,
                Some(command),
                Some(subcommand),
            )
        })?;
        u16::try_from(total).map_err(|_| {
            SlmpError::with_context(
                format!("request data length must be <= 65535 bytes: payload={payload_len}, total={total}"),
                None,
                Some(command),
                Some(subcommand),
            )
        })
    }

    fn validate_plain_monitor_register_payload(
        &self,
        mode: SlmpCompatibilityMode,
        payload: &[u8],
    ) -> Result<(), SlmpError> {
        if payload.len() < 2 {
            return Err(SlmpError::new("Monitor register payload is too short."));
        }
        let word_count = payload[0] as usize;
        let dword_count = payload[1] as usize;
        rules::validate_random_read_like_counts(
            word_count,
            dword_count,
            mode,
            self.options.plc_profile,
            SlmpProfileLimit::MonitorRegisterWord,
            "register_monitor_devices",
        )?;
        let spec_size = device_spec_size(mode);
        let expected = 2 + (word_count + dword_count) * spec_size;
        if payload.len() != expected {
            return Err(SlmpError::new(format!(
                "Monitor register payload size mismatch: expected={expected} actual={}",
                payload.len()
            )));
        }
        let mut offset = 2usize;
        for _ in 0..(word_count + dword_count) {
            let code = match mode {
                SlmpCompatibilityMode::Legacy => u16::from(payload[offset + 3]),
                SlmpCompatibilityMode::Iqr => {
                    u16::from_le_bytes([payload[offset + 4], payload[offset + 5]])
                }
            };
            if code == SlmpDeviceCode::LCS.as_u16() || code == SlmpDeviceCode::LCC.as_u16() {
                return Err(SlmpError::new(
                    "Entry Monitor Device (0x0801) does not support LCS/LCC. Poll them through read_typed/read_named instead.",
                ));
            }
            if code == SlmpDeviceCode::G.as_u16() || code == SlmpDeviceCode::HG.as_u16() {
                return Err(SlmpError::new(
                    "Entry Monitor Device (0x0801) does not support standalone G/HG. Use U-qualified extended access.",
                ));
            }
            offset += spec_size;
        }
        Ok(())
    }

    fn build_request_frame(
        &mut self,
        command: SlmpCommand,
        subcommand: u16,
        payload: &[u8],
    ) -> Result<(), SlmpError> {
        let request_data_length = Self::request_data_length(command, subcommand, payload.len())?;
        let header_size = match self.options.frame_type {
            SlmpFrameType::Frame4E => 19,
            SlmpFrameType::Frame3E => 15,
        };
        self.last_request_frame
            .resize(header_size + payload.len(), 0);
        let frame = &mut self.last_request_frame;
        match self.options.frame_type {
            SlmpFrameType::Frame4E => {
                frame[0] = 0x54;
                frame[1] = 0x00;
                frame[2..4].copy_from_slice(&self.serial.to_le_bytes());
                self.serial = self.serial.wrapping_add(1);
                Self::write_target(&mut frame[6..11], self.options.target);
                frame[11..13].copy_from_slice(&request_data_length.to_le_bytes());
                frame[13..15].copy_from_slice(&self.options.monitoring_timer.to_le_bytes());
                frame[15..17].copy_from_slice(&command.as_u16().to_le_bytes());
                frame[17..19].copy_from_slice(&subcommand.to_le_bytes());
            }
            SlmpFrameType::Frame3E => {
                frame[0] = 0x50;
                frame[1] = 0x00;
                Self::write_target(&mut frame[2..7], self.options.target);
                frame[7..9].copy_from_slice(&request_data_length.to_le_bytes());
                frame[9..11].copy_from_slice(&self.options.monitoring_timer.to_le_bytes());
                frame[11..13].copy_from_slice(&command.as_u16().to_le_bytes());
                frame[13..15].copy_from_slice(&subcommand.to_le_bytes());
            }
        }
        frame[header_size..].copy_from_slice(payload);
        Ok(())
    }

    fn write_target(buffer: &mut [u8], target: SlmpTargetAddress) {
        buffer[0] = target.network;
        buffer[1] = target.station;
        buffer[2..4].copy_from_slice(&target.module_io.to_le_bytes());
        buffer[4] = target.multidrop;
    }

    async fn receive_tcp_frame(
        stream: &mut TcpStream,
        frame_type: SlmpFrameType,
        timeout_duration: std::time::Duration,
        frame: &mut Vec<u8>,
    ) -> Result<(), SlmpError> {
        let mut header = [0u8; 13];
        timeout(timeout_duration, stream.read_exact(&mut header[0..2]))
            .await
            .map_err(|_| SlmpError::new("tcp read timed out"))??;

        if matches!(frame_type, SlmpFrameType::Frame4E) && header[0] == 0xD4 && header[1] == 0x00 {
            timeout(timeout_duration, stream.read_exact(&mut header[2..13]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            let length = u16::from_le_bytes([header[11], header[12]]) as usize;
            frame.resize(13 + length, 0);
            frame[0..13].copy_from_slice(&header);
            timeout(timeout_duration, stream.read_exact(&mut frame[13..]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            return Ok(());
        }

        if matches!(frame_type, SlmpFrameType::Frame3E) && header[0] == 0xD0 && header[1] == 0x00 {
            timeout(timeout_duration, stream.read_exact(&mut header[2..9]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            let length = u16::from_le_bytes([header[7], header[8]]) as usize;
            frame.resize(9 + length, 0);
            frame[0..9].copy_from_slice(&header[..9]);
            timeout(timeout_duration, stream.read_exact(&mut frame[9..]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            return Ok(());
        }

        Err(SlmpError::new("invalid response subheader"))
    }

    fn parse_response(
        command: SlmpCommand,
        subcommand: u16,
        response: &[u8],
    ) -> Result<Vec<u8>, SlmpError> {
        let is_4e = response.len() >= 13 && response[0] == 0xD4 && response[1] == 0x00;
        let is_3e = response.len() >= 9 && response[0] == 0xD0 && response[1] == 0x00;
        if !is_4e && !is_3e {
            return Err(SlmpError::with_context(
                "invalid response header",
                None,
                Some(command),
                Some(subcommand),
            ));
        }
        let header_size = if is_4e { 13 } else { 9 };
        let length_index = if is_4e { 11 } else { 7 };
        let data_length =
            u16::from_le_bytes([response[length_index], response[length_index + 1]]) as usize;
        if response.len() < header_size + data_length || data_length < 2 {
            return Err(SlmpError::with_context(
                "malformed response",
                None,
                Some(command),
                Some(subcommand),
            ));
        }
        let end_index = header_size;
        let end_code = u16::from_le_bytes([response[end_index], response[end_index + 1]]);
        if end_code != 0 {
            let error_info =
                SlmpErrorInfo::parse(&response[header_size + 2..header_size + data_length]);
            return Err(SlmpError::with_error_info(
                format!(
                    "SLMP error end_code=0x{end_code:04X} command=0x{:04X} subcommand=0x{subcommand:04X}",
                    command.as_u16()
                ),
                Some(end_code),
                Some(command),
                Some(subcommand),
                error_info,
            ));
        }
        if data_length == 2 {
            return Ok(Vec::new());
        }
        Ok(response[header_size + 2..header_size + data_length].to_vec())
    }

    fn has_expected_response_frame_type(frame_type: SlmpFrameType, response: &[u8]) -> bool {
        match frame_type {
            SlmpFrameType::Frame4E => {
                response.len() >= 2 && response[0] == 0xD4 && response[1] == 0x00
            }
            SlmpFrameType::Frame3E => {
                response.len() >= 2 && response[0] == 0xD0 && response[1] == 0x00
            }
        }
    }

    fn has_expected_response_serial(response: &[u8], expected_serial: Option<u16>) -> bool {
        let Some(expected_serial) = expected_serial else {
            return true;
        };
        if response.len() < 4 || response[0] != 0xD4 || response[1] != 0x00 {
            return false;
        }
        u16::from_le_bytes([response[2], response[3]]) == expected_serial
    }

    fn word_subcommand(&self, bit_unit: bool) -> u16 {
        match (self.options.compatibility_mode, bit_unit) {
            (SlmpCompatibilityMode::Legacy, false) => 0x0000,
            (SlmpCompatibilityMode::Legacy, true) => 0x0001,
            (SlmpCompatibilityMode::Iqr, false) => 0x0002,
            (SlmpCompatibilityMode::Iqr, true) => 0x0003,
        }
    }

    fn encode_device_spec(&self, device: SlmpDeviceAddress, output: &mut [u8]) -> usize {
        match self.options.compatibility_mode {
            SlmpCompatibilityMode::Legacy => {
                output[0] = (device.number() & 0xFF) as u8;
                output[1] = ((device.number() >> 8) & 0xFF) as u8;
                output[2] = ((device.number() >> 16) & 0xFF) as u8;
                output[3] = device.code().as_u8();
                4
            }
            SlmpCompatibilityMode::Iqr => {
                output[0..4].copy_from_slice(&device.number().to_le_bytes());
                output[4..6].copy_from_slice(&device.code().as_u16().to_le_bytes());
                6
            }
        }
    }

    fn build_read_write_payload(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
        values: Option<&[u16]>,
        bit_unit: bool,
    ) -> Vec<u8> {
        let write_bytes = if let Some(values) = values {
            if bit_unit {
                values.len().div_ceil(2)
            } else {
                values.len() * 2
            }
        } else {
            0
        };
        let mut payload =
            vec![0u8; device_spec_size(self.options.compatibility_mode) + 2 + write_bytes];
        let mut offset = self.encode_device_spec(device, &mut payload);
        payload[offset..offset + 2].copy_from_slice(&points.to_le_bytes());
        offset += 2;
        if let Some(values) = values {
            if bit_unit {
                let mut index = 0;
                while index < values.len() {
                    let high = if values[index] != 0 { 0x10 } else { 0x00 };
                    index += 1;
                    let low = if index < values.len() && values[index] != 0 {
                        0x01
                    } else {
                        0x00
                    };
                    if index < values.len() {
                        index += 1;
                    }
                    payload[offset] = high | low;
                    offset += 1;
                }
            } else {
                for value in values {
                    payload[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                    offset += 2;
                }
            }
        }
        payload
    }

    fn resolve_effective_extension(
        device: SlmpQualifiedDeviceAddress,
        plc_profile: SlmpPlcProfile,
    ) -> Result<SlmpExtensionSpec, SlmpError> {
        let mut result = SlmpExtensionSpec::default();
        if let Some(extension_specification) = device.extension_specification() {
            result.extension_specification = extension_specification;
        }
        if let Some(direct_memory_specification) = device.direct_memory_specification() {
            result.direct_memory_specification = direct_memory_specification;
        }
        if device.direct_memory_specification() == Some(0xF9) && device.modification().is_some() {
            return Err(SlmpError::new(
                "J-qualified link-direct devices do not support Z, LZ, or indirect modification",
            ));
        }
        match device.modification() {
            None => {}
            Some(SlmpDeviceModification::IndexZ(index)) => {
                result.device_modification_index = index;
                result.device_modification_flags = 0x40;
            }
            Some(SlmpDeviceModification::IndexLz(index)) => {
                if !plc_profile.uses_iqr_protocol() {
                    return Err(SlmpError::new(format!(
                        "LZ index modification is not supported for plc_profile '{}'",
                        plc_profile.canonical_name()
                    )));
                }
                if index > 1 {
                    return Err(SlmpError::new("LZ index must be 0 or 1."));
                }
                result.device_modification_index = index;
                result.device_modification_flags = 0x80;
            }
            Some(SlmpDeviceModification::Indirect) => {
                result.device_modification_flags = 0x08;
            }
        }
        match device.device().code() {
            SlmpDeviceCode::G => {
                if device.extension_specification().is_none() {
                    return Err(SlmpError::new(
                        "G Extended Device access requires U-qualified module access such as U1\\G0.",
                    ));
                }
                if result.direct_memory_specification == 0 {
                    result.direct_memory_specification = 0xF8;
                } else if result.direct_memory_specification != 0xF8 {
                    return Err(SlmpError::new(format!(
                        "G Extended Device access requires direct_memory_specification=0xF8; got 0x{:02X}.",
                        result.direct_memory_specification
                    )));
                }
            }
            SlmpDeviceCode::HG => {
                let Some(extension_specification) = device.extension_specification() else {
                    return Err(SlmpError::new(
                        "HG Extended Device access requires U-qualified CPU-buffer access U3E0\\HG through U3E3\\HG.",
                    ));
                };
                if !matches!(extension_specification, 0x03E0..=0x03E3) {
                    return Err(SlmpError::new(
                        "HG Extended Device access is valid only for U3E0\\HG through U3E3\\HG.",
                    ));
                }
                if result.direct_memory_specification == 0 {
                    result.direct_memory_specification = 0xFA;
                } else if result.direct_memory_specification != 0xFA {
                    return Err(SlmpError::new(format!(
                        "HG Extended Device access requires direct_memory_specification=0xFA; got 0x{:02X}.",
                        result.direct_memory_specification
                    )));
                }
            }
            _ => {}
        }
        Ok(result)
    }

    fn build_read_write_payload_extended(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
        values: Option<&[u16]>,
        extension: SlmpExtensionSpec,
        bit_unit: bool,
    ) -> Vec<u8> {
        let mut payload = self.encode_extended_device_spec(device, extension);
        payload.extend_from_slice(&points.to_le_bytes());
        if let Some(values) = values {
            if bit_unit {
                let mut index = 0;
                while index < values.len() {
                    let high = if values[index] != 0 { 0x10 } else { 0x00 };
                    index += 1;
                    let low = if index < values.len() && values[index] != 0 {
                        0x01
                    } else {
                        0x00
                    };
                    if index < values.len() {
                        index += 1;
                    }
                    payload.push(high | low);
                }
            } else {
                for value in values {
                    payload.extend_from_slice(&value.to_le_bytes());
                }
            }
        }
        payload
    }

    pub(crate) fn encode_extended_device_spec(
        &self,
        device: SlmpDeviceAddress,
        extension: SlmpExtensionSpec,
    ) -> Vec<u8> {
        if extension.direct_memory_specification == 0xF9 {
            return vec![
                0x00,
                0x00,
                (device.number() & 0xFF) as u8,
                ((device.number() >> 8) & 0xFF) as u8,
                ((device.number() >> 16) & 0xFF) as u8,
                device.code().as_u8(),
                0x00,
                0x00,
                (extension.extension_specification & 0xFF) as u8,
                0x00,
                0xF9,
            ];
        }

        let mut device_spec = vec![0u8; device_spec_size(self.options.compatibility_mode)];
        let _ = self.encode_device_spec(device, &mut device_spec);

        let mut payload = Vec::with_capacity(2 + device_spec.len() + 2 + 2 + 1);
        payload.push(extension.device_modification_index);
        payload.push(extension.device_modification_flags);
        payload.extend_from_slice(&device_spec);
        payload.push(extension.extension_specification_modification);
        payload.push(0x00);
        payload.extend_from_slice(&extension.extension_specification.to_le_bytes());
        payload.push(extension.direct_memory_specification);
        payload
    }

    fn encode_password(&self, password: &str) -> Result<Vec<u8>, SlmpError> {
        let raw = password.as_bytes();
        if self.options.plc_profile.uses_iqr_protocol() {
            if raw.len() < 6 || raw.len() > 32 {
                return Err(SlmpError::new("iQ-R password length must be 6..32"));
            }
        } else if raw.len() != 4 {
            return Err(SlmpError::new("Q/L password length must be exactly 4"));
        }

        let mut payload = Vec::with_capacity(2 + raw.len());
        payload.extend_from_slice(&(raw.len() as u16).to_le_bytes());
        payload.extend_from_slice(raw);
        Ok(payload)
    }
}

impl std::fmt::Debug for SlmpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlmpClient").finish_non_exhaustive()
    }
}

pub fn encode_raw_device_spec(
    mode: SlmpCompatibilityMode,
    device: RawSlmpDeviceAddress,
) -> Vec<u8> {
    let size = device_spec_size(mode);
    let mut output = vec![0u8; size];
    match mode {
        SlmpCompatibilityMode::Legacy => {
            output[0] = (device.number & 0xFF) as u8;
            output[1] = ((device.number >> 8) & 0xFF) as u8;
            output[2] = ((device.number >> 16) & 0xFF) as u8;
            output[3] = device.code.as_u8();
        }
        SlmpCompatibilityMode::Iqr => {
            output[0..4].copy_from_slice(&device.number.to_le_bytes());
            output[4..6].copy_from_slice(&device.code.as_u16().to_le_bytes());
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SlmpModuleIo;

    async fn udp_inner(plc_profile: SlmpPlcProfile) -> ClientInner {
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        socket.connect("127.0.0.1:9").await.unwrap();
        ClientInner {
            options: SlmpConnectionOptions::new(
                "127.0.0.1",
                1025,
                SlmpTransportMode::Tcp,
                SlmpTargetAddress::default(),
                plc_profile,
            )
            .unwrap(),
            transport: Transport::Udp(socket),
            serial: 0,
            last_request_frame: Vec::new(),
            last_response_frame: Vec::new(),
            traffic_stats: SlmpTrafficStats::default(),
        }
    }

    #[tokio::test]
    async fn encode_extended_device_spec_uses_manual_ql_layout() {
        let inner = udp_inner(SlmpPlcProfile::QCpuQj71E71100).await;
        let device = SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::QCpuQj71E71100);

        assert_eq!(
            inner.encode_extended_device_spec(device, SlmpExtensionSpec::default()),
            vec![
                0x00, 0x00, 0x64, 0x00, 0x00, 0xA8, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );

        assert_eq!(
            inner.encode_extended_device_spec(
                device,
                SlmpExtensionSpec {
                    device_modification_index: 0x04,
                    device_modification_flags: 0x40,
                    ..SlmpExtensionSpec::default()
                },
            ),
            vec![
                0x04, 0x40, 0x64, 0x00, 0x00, 0xA8, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }

    #[test]
    fn semantic_extended_device_modifications_derive_wire_flags() {
        let base = SlmpQualifiedDeviceAddress::module_access(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 100, SlmpPlcProfile::IqR),
            1,
        )
        .unwrap();

        let z = ClientInner::resolve_effective_extension(
            base.with_modification(SlmpDeviceModification::IndexZ(4))
                .unwrap(),
            SlmpPlcProfile::IqR,
        )
        .unwrap();
        assert_eq!(z.device_modification_index, 4);
        assert_eq!(z.device_modification_flags, 0x40);

        let lz = ClientInner::resolve_effective_extension(
            base.with_modification(SlmpDeviceModification::IndexLz(1))
                .unwrap(),
            SlmpPlcProfile::IqR,
        )
        .unwrap();
        assert_eq!(lz.device_modification_index, 1);
        assert_eq!(lz.device_modification_flags, 0x80);
        assert!(
            base.with_modification(SlmpDeviceModification::IndexLz(2))
                .is_err()
        );

        let indirect = ClientInner::resolve_effective_extension(
            base.with_modification(SlmpDeviceModification::Indirect)
                .unwrap(),
            SlmpPlcProfile::IqR,
        )
        .unwrap();
        assert_eq!(indirect.device_modification_index, 0);
        assert_eq!(indirect.device_modification_flags, 0x08);
    }

    #[test]
    fn semantic_extended_device_rejects_invalid_modification_combinations() {
        let link = SlmpQualifiedDeviceAddress::link_direct(
            SlmpDeviceAddress::new(SlmpDeviceCode::W, 10, SlmpPlcProfile::IqR),
            1,
        );
        assert!(
            link.with_modification(SlmpDeviceModification::IndexZ(1))
                .unwrap_err()
                .message
                .contains("link-direct")
        );

        let legacy = SlmpQualifiedDeviceAddress::module_access(
            SlmpDeviceAddress::new(SlmpDeviceCode::D, 10, SlmpPlcProfile::QCpuQj71E71100),
            1,
        )
        .unwrap();
        assert!(
            legacy
                .with_modification(SlmpDeviceModification::IndexLz(1))
                .unwrap_err()
                .message
                .contains("not supported")
        );
    }

    #[tokio::test]
    async fn target_address_accepts_module_io_constants_in_request_header() {
        let mut inner = udp_inner(SlmpPlcProfile::IqR).await;
        inner.options.target.module_io = SlmpModuleIo::MULTIPLE_CPU_2;

        inner
            .build_request_frame(SlmpCommand::ReadTypeName, 0x0000, &[])
            .unwrap();

        assert_eq!(SlmpModuleIo::CONTROL_SYSTEM_CPU, 0x03D0);
        assert_eq!(SlmpModuleIo::STANDBY_SYSTEM_CPU, 0x03D1);
        assert_eq!(SlmpModuleIo::SYSTEM_A_CPU, 0x03D2);
        assert_eq!(SlmpModuleIo::SYSTEM_B_CPU, 0x03D3);
        assert_eq!(SlmpModuleIo::MULTIPLE_CPU_1, 0x03E0);
        assert_eq!(SlmpModuleIo::MULTIPLE_CPU_2, 0x03E1);
        assert_eq!(SlmpModuleIo::MULTIPLE_CPU_3, 0x03E2);
        assert_eq!(SlmpModuleIo::MULTIPLE_CPU_4, 0x03E3);
        assert_eq!(SlmpModuleIo::REMOTE_HEAD_1, SlmpModuleIo::MULTIPLE_CPU_1);
        assert_eq!(SlmpModuleIo::REMOTE_HEAD_2, SlmpModuleIo::MULTIPLE_CPU_2);
        assert_eq!(
            SlmpModuleIo::CONTROL_SYSTEM_REMOTE_HEAD,
            SlmpModuleIo::CONTROL_SYSTEM_CPU
        );
        assert_eq!(
            SlmpModuleIo::STANDBY_SYSTEM_REMOTE_HEAD,
            SlmpModuleIo::STANDBY_SYSTEM_CPU
        );
        assert_eq!(SlmpModuleIo::OWN_STATION, 0x03FF);
        assert_eq!(
            SlmpTargetAddress::default().module_io,
            SlmpModuleIo::OWN_STATION
        );
        assert_eq!(
            u16::from_le_bytes([inner.last_request_frame[8], inner.last_request_frame[9]]),
            SlmpModuleIo::MULTIPLE_CPU_2
        );
    }

    #[test]
    fn plc_error_exposes_structured_error_information() {
        let error_data = [0x00, 0xFF, 0xFF, 0x03, 0x00, 0x01, 0x04, 0x01, 0x00];
        let mut response = vec![
            0xD4, 0x00, // subheader
            0x34, 0x12, // serial
            0x00, 0x00, // reserved
            0x00, 0xFF, 0xFF, 0x03, 0x00, // target
            0x0B, 0x00, // response data length: end_code + 9-byte error info
            0x51, 0xC0, // end_code
        ];
        response.extend_from_slice(&error_data);

        let error =
            ClientInner::parse_response(SlmpCommand::DeviceRead, 0x0001, &response).unwrap_err();

        assert_eq!(error.end_code, Some(0xC051));
        let info = error.error_info.as_ref().expect("error info");
        assert_eq!(info.network, 0x00);
        assert_eq!(info.station, 0xFF);
        assert_eq!(info.module_io, 0x03FF);
        assert_eq!(info.multidrop, 0x00);
        assert_eq!(info.command, 0x0401);
        assert_eq!(info.subcommand, 0x0001);
        assert_eq!(info.raw.as_slice(), error_data);
    }
}
