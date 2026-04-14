use crate::device_ranges::{
    SlmpDeviceRangeCatalog, SlmpDeviceRangeFamily, build_catalog as build_device_range_catalog,
    build_catalog_for_family as build_device_range_catalog_for_family,
    read_registers as read_device_range_registers,
    resolve_profile as resolve_device_range_profile,
    resolve_profile_for_family as resolve_device_range_profile_for_family,
};
use crate::address::device_spec_size;
use crate::error::SlmpError;
use crate::model::{
    SlmpBlockRead, SlmpBlockReadResult, SlmpBlockWrite, SlmpBlockWriteOptions, SlmpCommand,
    SlmpCompatibilityMode, SlmpConnectionOptions, SlmpDeviceAddress, SlmpDeviceCode,
    SlmpCpuOperationState, SlmpCpuOperationStatus, SlmpExtensionSpec, SlmpFrameType,
    SlmpLongTimerResult, SlmpQualifiedDeviceAddress, SlmpRandomReadResult, SlmpTargetAddress,
    SlmpTrafficStats, SlmpTransportMode, SlmpTypeNameInfo,
};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Mutex;
use tokio::time::timeout;

#[derive(Clone)]
pub struct SlmpClient {
    inner: Arc<Mutex<ClientInner>>,
}

enum Transport {
    Tcp(TcpStream),
    Udp(UdpSocket),
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
        let transport = match options.transport_mode {
            SlmpTransportMode::Tcp => {
                let stream = timeout(
                    options.timeout,
                    TcpStream::connect((options.host.as_str(), options.port)),
                )
                .await
                .map_err(|_| SlmpError::new("tcp connect timed out"))??;
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

    pub async fn read_type_name(&self) -> Result<SlmpTypeNameInfo, SlmpError> {
        self.inner.lock().await.read_type_name().await
    }

    pub async fn read_cpu_operation_state(&self) -> Result<SlmpCpuOperationState, SlmpError> {
        self.inner.lock().await.read_cpu_operation_state().await
    }

    pub async fn read_device_range_catalog(&self) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
        let type_info = self.read_type_name().await?;
        let profile = resolve_device_range_profile(&type_info)?;
        let registers = read_device_range_registers(self, &profile).await?;
        build_device_range_catalog(&type_info, &profile, &registers)
    }

    pub async fn read_device_range_catalog_for_family(
        &self,
        family: SlmpDeviceRangeFamily,
    ) -> Result<SlmpDeviceRangeCatalog, SlmpError> {
        let profile = resolve_device_range_profile_for_family(family);
        let registers = read_device_range_registers(self, &profile).await?;
        build_device_range_catalog_for_family(family, &registers)
    }

    pub async fn read_words_raw(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        self.inner.lock().await.read_words_raw(device, points).await
    }

    pub async fn write_words(
        &self,
        device: SlmpDeviceAddress,
        values: &[u16],
    ) -> Result<(), SlmpError> {
        self.inner.lock().await.write_words(device, values).await
    }

    pub async fn read_bits(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<bool>, SlmpError> {
        self.inner.lock().await.read_bits(device, points).await
    }

    pub async fn write_bits(
        &self,
        device: SlmpDeviceAddress,
        values: &[bool],
    ) -> Result<(), SlmpError> {
        self.inner.lock().await.write_bits(device, values).await
    }

    pub async fn read_dwords_raw(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<u32>, SlmpError> {
        self.inner
            .lock()
            .await
            .read_dwords_raw(device, points)
            .await
    }

    pub async fn write_dwords(
        &self,
        device: SlmpDeviceAddress,
        values: &[u32],
    ) -> Result<(), SlmpError> {
        self.inner.lock().await.write_dwords(device, values).await
    }

    pub async fn read_float32s(
        &self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<f32>, SlmpError> {
        self.inner.lock().await.read_float32s(device, points).await
    }

    pub async fn write_float32s(
        &self,
        device: SlmpDeviceAddress,
        values: &[f32],
    ) -> Result<(), SlmpError> {
        self.inner.lock().await.write_float32s(device, values).await
    }

    pub async fn read_words_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        points: u16,
        extension: SlmpExtensionSpec,
    ) -> Result<Vec<u16>, SlmpError> {
        self.inner
            .lock()
            .await
            .read_words_extended(device, points, extension)
            .await
    }

    pub async fn write_words_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        values: &[u16],
        extension: SlmpExtensionSpec,
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .write_words_extended(device, values, extension)
            .await
    }

    pub async fn read_bits_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        points: u16,
        extension: SlmpExtensionSpec,
    ) -> Result<Vec<bool>, SlmpError> {
        self.inner
            .lock()
            .await
            .read_bits_extended(device, points, extension)
            .await
    }

    pub async fn write_bits_extended(
        &self,
        device: SlmpQualifiedDeviceAddress,
        values: &[bool],
        extension: SlmpExtensionSpec,
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .write_bits_extended(device, values, extension)
            .await
    }

    pub async fn read_random(
        &self,
        word_devices: &[SlmpDeviceAddress],
        dword_devices: &[SlmpDeviceAddress],
    ) -> Result<SlmpRandomReadResult, SlmpError> {
        self.inner
            .lock()
            .await
            .read_random(word_devices, dword_devices)
            .await
    }

    pub async fn write_random_words(
        &self,
        word_entries: &[(SlmpDeviceAddress, u16)],
        dword_entries: &[(SlmpDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .write_random_words(word_entries, dword_entries)
            .await
    }

    pub async fn write_random_bits(
        &self,
        bit_entries: &[(SlmpDeviceAddress, bool)],
    ) -> Result<(), SlmpError> {
        self.inner.lock().await.write_random_bits(bit_entries).await
    }

    pub async fn read_block(
        &self,
        word_blocks: &[SlmpBlockRead],
        bit_blocks: &[SlmpBlockRead],
    ) -> Result<SlmpBlockReadResult, SlmpError> {
        self.inner
            .lock()
            .await
            .read_block(word_blocks, bit_blocks)
            .await
    }

    pub async fn write_block(
        &self,
        word_blocks: &[SlmpBlockWrite],
        bit_blocks: &[SlmpBlockWrite],
        options: Option<SlmpBlockWriteOptions>,
    ) -> Result<(), SlmpError> {
        self.inner
            .lock()
            .await
            .write_block(word_blocks, bit_blocks, options.unwrap_or_default())
            .await
    }

    pub async fn remote_run(&self, force: bool, clear_mode: u16) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_run(force, clear_mode).await
    }

    pub async fn remote_stop(&self) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_stop().await
    }

    pub async fn remote_pause(&self, force: bool) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_pause(force).await
    }

    pub async fn remote_latch_clear(&self) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_latch_clear().await
    }

    pub async fn remote_reset(&self, expect_response: bool) -> Result<(), SlmpError> {
        self.inner.lock().await.remote_reset(expect_response).await
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

    pub async fn request(
        &self,
        command: SlmpCommand,
        subcommand: u16,
        payload: &[u8],
        expect_response: bool,
    ) -> Result<Vec<u8>, SlmpError> {
        self.inner
            .lock()
            .await
            .request(command, subcommand, payload, expect_response)
            .await
    }
}

impl ClientInner {
    async fn read_type_name(&mut self) -> Result<SlmpTypeNameInfo, SlmpError> {
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
            .read_words_raw(SlmpDeviceAddress::new(SlmpDeviceCode::SD, 203), 1)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| SlmpError::new("read_cpu_operation_state expected one word"))?;
        Ok(decode_cpu_operation_state(status_word))
    }

    async fn read_words_raw(
        &mut self,
        device: SlmpDeviceAddress,
        points: u16,
    ) -> Result<Vec<u16>, SlmpError> {
        Self::validate_direct_word_read(device, points)?;
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
        Self::validate_direct_bit_read(device)?;
        let payload = self.build_read_write_payload(device, points, None, true);
        let data = self
            .request(
                SlmpCommand::DeviceRead,
                self.word_subcommand(true),
                &payload,
                true,
            )
            .await?;
        Self::unpack_bit_values(&data, points as usize)
    }

    async fn write_bits(
        &mut self,
        device: SlmpDeviceAddress,
        values: &[bool],
    ) -> Result<(), SlmpError> {
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
        Self::validate_direct_dword_read(device)?;
        let words = self.read_words_raw(device, points * 2).await?;
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
        extension: SlmpExtensionSpec,
    ) -> Result<Vec<u16>, SlmpError> {
        let extension = Self::resolve_effective_extension(device, extension);
        let payload =
            self.build_read_write_payload_extended(device.device, points, None, extension, false);
        let sub = if extension.direct_memory_specification == 0xF9 {
            0x0080
        } else if matches!(
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
        extension: SlmpExtensionSpec,
    ) -> Result<(), SlmpError> {
        let extension = Self::resolve_effective_extension(device, extension);
        let payload = self.build_read_write_payload_extended(
            device.device,
            values.len() as u16,
            Some(values),
            extension,
            false,
        );
        let sub = if extension.direct_memory_specification == 0xF9 {
            0x0080
        } else if matches!(
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
        extension: SlmpExtensionSpec,
    ) -> Result<Vec<bool>, SlmpError> {
        let extension = Self::resolve_effective_extension(device, extension);
        let payload =
            self.build_read_write_payload_extended(device.device, points, None, extension, true);
        let sub = if extension.direct_memory_specification == 0xF9 {
            0x0081
        } else if matches!(
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
        Self::unpack_bit_values(&data, points as usize)
    }

    async fn write_bits_extended(
        &mut self,
        device: SlmpQualifiedDeviceAddress,
        values: &[bool],
        extension: SlmpExtensionSpec,
    ) -> Result<(), SlmpError> {
        let extension = Self::resolve_effective_extension(device, extension);
        let words: Vec<u16> = values.iter().map(|value| u16::from(*value)).collect();
        let payload = self.build_read_write_payload_extended(
            device.device,
            values.len() as u16,
            Some(&words),
            extension,
            true,
        );
        let sub = if extension.direct_memory_specification == 0xF9 {
            0x0081
        } else if matches!(
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
        Self::validate_no_lcs_lcc_random(word_devices, dword_devices)?;
        if word_devices.len() > 0xFF || dword_devices.len() > 0xFF {
            return Err(SlmpError::new("random counts must be <= 255"));
        }
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

    async fn write_random_words(
        &mut self,
        word_entries: &[(SlmpDeviceAddress, u16)],
        dword_entries: &[(SlmpDeviceAddress, u32)],
    ) -> Result<(), SlmpError> {
        if word_entries.len() > 0xFF || dword_entries.len() > 0xFF {
            return Err(SlmpError::new("random counts must be <= 255"));
        }
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

    async fn write_random_bits(
        &mut self,
        bit_entries: &[(SlmpDeviceAddress, bool)],
    ) -> Result<(), SlmpError> {
        if bit_entries.len() > 0xFF {
            return Err(SlmpError::new("random bit count must be <= 255"));
        }
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

    async fn read_block(
        &mut self,
        word_blocks: &[SlmpBlockRead],
        bit_blocks: &[SlmpBlockRead],
    ) -> Result<SlmpBlockReadResult, SlmpError> {
        Self::validate_no_lcs_lcc_block_read(word_blocks, bit_blocks)?;
        if word_blocks.len() > 0xFF || bit_blocks.len() > 0xFF {
            return Err(SlmpError::new("block counts must be <= 255"));
        }
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
        options: SlmpBlockWriteOptions,
    ) -> Result<(), SlmpError> {
        Self::validate_no_lcs_lcc_block_write(word_blocks, bit_blocks)?;
        if options.split_mixed_blocks && !word_blocks.is_empty() && !bit_blocks.is_empty() {
            self.write_block_once(word_blocks, &[]).await?;
            self.write_block_once(&[], bit_blocks).await?;
            return Ok(());
        }
        match self.write_block_once(word_blocks, bit_blocks).await {
            Ok(_) => Ok(()),
            Err(error)
                if options.retry_mixed_on_error
                    && !word_blocks.is_empty()
                    && !bit_blocks.is_empty()
                    && matches!(error.end_code, Some(0xC056 | 0xC05B | 0xC061 | 0x414A)) =>
            {
                self.write_block_once(word_blocks, &[]).await?;
                self.write_block_once(&[], bit_blocks).await
            }
            Err(error) => Err(error),
        }
    }

    async fn write_block_once(
        &mut self,
        word_blocks: &[SlmpBlockWrite],
        bit_blocks: &[SlmpBlockWrite],
    ) -> Result<(), SlmpError> {
        if word_blocks.len() > 0xFF || bit_blocks.len() > 0xFF {
            return Err(SlmpError::new("block counts must be <= 255"));
        }
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
        let mut offset = 2;
        for block in word_blocks {
            offset += self.encode_device_spec(block.device, &mut payload[offset..]);
            let count = block.values.len() as u16;
            payload[offset..offset + 2].copy_from_slice(&count.to_le_bytes());
            offset += 2;
        }
        for block in bit_blocks {
            offset += self.encode_device_spec(block.device, &mut payload[offset..]);
            let count = block.values.len() as u16;
            payload[offset..offset + 2].copy_from_slice(&count.to_le_bytes());
            offset += 2;
        }
        for block in word_blocks {
            for value in &block.values {
                payload[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                offset += 2;
            }
        }
        for block in bit_blocks {
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

    async fn remote_run(&mut self, force: bool, clear_mode: u16) -> Result<(), SlmpError> {
        let mode = if force { 0x03 } else { 0x01 };
        let payload = [
            mode,
            0x00,
            (clear_mode & 0xFF) as u8,
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

    async fn remote_pause(&mut self, force: bool) -> Result<(), SlmpError> {
        let mode = if force { 0x03 } else { 0x01 };
        self.request(SlmpCommand::RemotePause, 0x0000, &[mode, 0x00], true)
            .await?;
        Ok(())
    }

    async fn remote_latch_clear(&mut self) -> Result<(), SlmpError> {
        self.request(SlmpCommand::RemoteLatchClear, 0x0000, &[0x01, 0x00], true)
            .await?;
        Ok(())
    }

    async fn remote_reset(&mut self, expect_response: bool) -> Result<(), SlmpError> {
        self.request(SlmpCommand::RemoteReset, 0x0000, &[], expect_response)
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
        if data.len() > u16::MAX as usize {
            return Err(SlmpError::new("loopback payload must be <= 65535 bytes"));
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
        let mut payload = Vec::with_capacity(8);
        payload.extend_from_slice(&head_address.to_le_bytes());
        payload.extend_from_slice(&byte_length.to_le_bytes());
        payload.extend_from_slice(&module_no.to_le_bytes());
        self.request(SlmpCommand::ExtendUnitRead, 0x0000, &payload, true)
            .await
    }

    async fn extend_unit_read_words(
        &mut self,
        head_address: u32,
        word_length: u16,
        module_no: u16,
    ) -> Result<Vec<u16>, SlmpError> {
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

    async fn read_long_timer(
        &mut self,
        head_no: u32,
        points: usize,
    ) -> Result<Vec<SlmpLongTimerResult>, SlmpError> {
        let words = self
            .read_words_raw(
                SlmpDeviceAddress::new(SlmpDeviceCode::LTN, head_no),
                (points * 4) as u16,
            )
            .await?;
        Ok(Self::parse_long_timer_words(&words, head_no, "LTN"))
    }

    async fn read_long_retentive_timer(
        &mut self,
        head_no: u32,
        points: usize,
    ) -> Result<Vec<SlmpLongTimerResult>, SlmpError> {
        let words = self
            .read_words_raw(
                SlmpDeviceAddress::new(SlmpDeviceCode::LSTN, head_no),
                (points * 4) as u16,
            )
            .await?;
        Ok(Self::parse_long_timer_words(&words, head_no, "LSTN"))
    }

    async fn request(
        &mut self,
        command: SlmpCommand,
        subcommand: u16,
        payload: &[u8],
        expect_response: bool,
    ) -> Result<Vec<u8>, SlmpError> {
        self.validate_request_payload(command, subcommand, payload)?;
        let frame = self.build_request_frame(command, subcommand, payload);
        self.last_request_frame = frame.clone();
        let tx_len = frame.len() as u64;

        match &mut self.transport {
            Transport::Tcp(stream) => {
                timeout(self.options.timeout, stream.write_all(&frame))
                    .await
                    .map_err(|_| SlmpError::new("tcp write timed out"))??;
                self.traffic_stats.request_count += 1;
                self.traffic_stats.tx_bytes += tx_len;
                if !expect_response {
                    self.last_response_frame.clear();
                    return Ok(Vec::new());
                }
                let response = Self::receive_tcp_frame(stream, self.options.timeout).await?;
                self.last_response_frame = response.clone();
                self.traffic_stats.rx_bytes += response.len() as u64;
                Self::parse_response(command, subcommand, &response)
            }
            Transport::Udp(socket) => {
                timeout(self.options.timeout, socket.send(&frame))
                    .await
                    .map_err(|_| SlmpError::new("udp send timed out"))??;
                self.traffic_stats.request_count += 1;
                self.traffic_stats.tx_bytes += tx_len;
                if !expect_response {
                    self.last_response_frame.clear();
                    return Ok(Vec::new());
                }
                let mut buffer = vec![0u8; 8192];
                let received = timeout(self.options.timeout, socket.recv(&mut buffer))
                    .await
                    .map_err(|_| SlmpError::new("udp receive timed out"))??;
                buffer.truncate(received);
                self.last_response_frame = buffer.clone();
                self.traffic_stats.rx_bytes += buffer.len() as u64;
                Self::parse_response(command, subcommand, &buffer)
            }
        }
    }

    fn validate_request_payload(
        &self,
        command: SlmpCommand,
        subcommand: u16,
        payload: &[u8],
    ) -> Result<(), SlmpError> {
        if matches!(command, SlmpCommand::MonitorRegister) && matches!(subcommand, 0x0000 | 0x0002)
        {
            Self::validate_plain_monitor_register_payload(
                self.options.compatibility_mode,
                payload,
            )?;
        }
        Ok(())
    }

    fn validate_plain_monitor_register_payload(
        mode: SlmpCompatibilityMode,
        payload: &[u8],
    ) -> Result<(), SlmpError> {
        if payload.len() < 2 {
            return Err(SlmpError::new("Monitor register payload is too short."));
        }
        let word_count = payload[0] as usize;
        let dword_count = payload[1] as usize;
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
                    "Entry Monitor Device (0x0801) does not support LCS/LCC. Use read_typed/read_named or read the LCN 4-word status block.",
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
    ) -> Vec<u8> {
        let header_size = match self.options.frame_type {
            SlmpFrameType::Frame4E => 19,
            SlmpFrameType::Frame3E => 15,
        };
        let mut frame = vec![0u8; header_size + payload.len()];
        match self.options.frame_type {
            SlmpFrameType::Frame4E => {
                frame[0] = 0x54;
                frame[1] = 0x00;
                frame[2..4].copy_from_slice(&self.serial.to_le_bytes());
                self.serial = self.serial.wrapping_add(1);
                Self::write_target(&mut frame[6..11], self.options.target);
                frame[11..13].copy_from_slice(&(6u16 + payload.len() as u16).to_le_bytes());
                frame[13..15].copy_from_slice(&self.options.monitoring_timer.to_le_bytes());
                frame[15..17].copy_from_slice(&command.as_u16().to_le_bytes());
                frame[17..19].copy_from_slice(&subcommand.to_le_bytes());
            }
            SlmpFrameType::Frame3E => {
                frame[0] = 0x50;
                frame[1] = 0x00;
                Self::write_target(&mut frame[2..7], self.options.target);
                frame[7..9].copy_from_slice(&(6u16 + payload.len() as u16).to_le_bytes());
                frame[9..11].copy_from_slice(&self.options.monitoring_timer.to_le_bytes());
                frame[11..13].copy_from_slice(&command.as_u16().to_le_bytes());
                frame[13..15].copy_from_slice(&subcommand.to_le_bytes());
            }
        }
        frame[header_size..].copy_from_slice(payload);
        frame
    }

    fn write_target(buffer: &mut [u8], target: SlmpTargetAddress) {
        buffer[0] = target.network;
        buffer[1] = target.station;
        buffer[2..4].copy_from_slice(&target.module_io.to_le_bytes());
        buffer[4] = target.multidrop;
    }

    async fn receive_tcp_frame(
        stream: &mut TcpStream,
        timeout_duration: std::time::Duration,
    ) -> Result<Vec<u8>, SlmpError> {
        let mut header = [0u8; 13];
        timeout(timeout_duration, stream.read_exact(&mut header[0..2]))
            .await
            .map_err(|_| SlmpError::new("tcp read timed out"))??;

        if header[0] == 0xD4 && header[1] == 0x00 {
            timeout(timeout_duration, stream.read_exact(&mut header[2..13]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            let length = u16::from_le_bytes([header[11], header[12]]) as usize;
            let mut frame = vec![0u8; 13 + length];
            frame[0..13].copy_from_slice(&header);
            timeout(timeout_duration, stream.read_exact(&mut frame[13..]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            return Ok(frame);
        }

        if header[0] == 0xD0 && header[1] == 0x00 {
            timeout(timeout_duration, stream.read_exact(&mut header[2..9]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            let length = u16::from_le_bytes([header[7], header[8]]) as usize;
            let mut frame = vec![0u8; 9 + length];
            frame[0..9].copy_from_slice(&header[..9]);
            timeout(timeout_duration, stream.read_exact(&mut frame[9..]))
                .await
                .map_err(|_| SlmpError::new("tcp read timed out"))??;
            return Ok(frame);
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
            return Err(SlmpError::with_context(
                format!(
                    "SLMP error end_code=0x{end_code:04X} command=0x{:04X} subcommand=0x{subcommand:04X}",
                    command.as_u16()
                ),
                Some(end_code),
                Some(command),
                Some(subcommand),
            ));
        }
        if data_length == 2 {
            return Ok(Vec::new());
        }
        Ok(response[header_size + 2..header_size + data_length].to_vec())
    }

    fn word_subcommand(&self, bit_unit: bool) -> u16 {
        match (self.options.compatibility_mode, bit_unit) {
            (SlmpCompatibilityMode::Legacy, false) => 0x0000,
            (SlmpCompatibilityMode::Legacy, true) => 0x0001,
            (SlmpCompatibilityMode::Iqr, false) => 0x0002,
            (SlmpCompatibilityMode::Iqr, true) => 0x0003,
        }
    }

    fn validate_direct_bit_read(device: SlmpDeviceAddress) -> Result<(), SlmpError> {
        match device.code {
            SlmpDeviceCode::LTS
            | SlmpDeviceCode::LTC
            | SlmpDeviceCode::LSTS
            | SlmpDeviceCode::LSTC => Err(SlmpError::new(
                "Direct bit read is not supported for long timer state devices. Use read_typed/read_named or a 4-word current-value block read.",
            )),
            _ => Ok(()),
        }
    }

    fn validate_direct_word_read(device: SlmpDeviceAddress, points: u16) -> Result<(), SlmpError> {
        match device.code {
            SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN if points == 0 || points % 4 != 0 => {
                Err(SlmpError::new(
                    "Long timer and long retentive timer current values must be read as 4-word blocks.",
                ))
            }
            _ => Ok(()),
        }
    }

    fn validate_direct_dword_read(device: SlmpDeviceAddress) -> Result<(), SlmpError> {
        match device.code {
            SlmpDeviceCode::LTN | SlmpDeviceCode::LSTN => Err(SlmpError::new(
                "Direct dword read is not supported for long timer current values. Use read_typed/read_named or a 4-word block read.",
            )),
            _ => Ok(()),
        }
    }

    fn validate_no_lcs_lcc_random(
        word_devices: &[SlmpDeviceAddress],
        dword_devices: &[SlmpDeviceAddress],
    ) -> Result<(), SlmpError> {
        for device in word_devices.iter().chain(dword_devices.iter()) {
            if matches!(device.code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC) {
                return Err(SlmpError::new(
                    "Read Random (0x0403) does not support LCS/LCC. Use read_typed/read_named or read the LCN 4-word status block.",
                ));
            }
        }
        Ok(())
    }

    fn validate_no_lcs_lcc_block_read(
        word_blocks: &[SlmpBlockRead],
        bit_blocks: &[SlmpBlockRead],
    ) -> Result<(), SlmpError> {
        for block in word_blocks.iter().chain(bit_blocks.iter()) {
            if matches!(block.device.code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC) {
                return Err(SlmpError::new(
                    "Read Block (0x0406) does not support LCS/LCC. Use read_typed/read_named or read the LCN 4-word status block.",
                ));
            }
        }
        Ok(())
    }

    fn validate_no_lcs_lcc_block_write(
        word_blocks: &[SlmpBlockWrite],
        bit_blocks: &[SlmpBlockWrite],
    ) -> Result<(), SlmpError> {
        for block in word_blocks.iter().chain(bit_blocks.iter()) {
            if matches!(block.device.code, SlmpDeviceCode::LCS | SlmpDeviceCode::LCC) {
                return Err(SlmpError::new(
                    "Write Block (0x1406) does not support LCS/LCC. Use write_typed/write_named or other supported write routes.",
                ));
            }
        }
        Ok(())
    }

    fn encode_device_spec(&self, device: SlmpDeviceAddress, output: &mut [u8]) -> usize {
        match self.options.compatibility_mode {
            SlmpCompatibilityMode::Legacy => {
                output[0] = (device.number & 0xFF) as u8;
                output[1] = ((device.number >> 8) & 0xFF) as u8;
                output[2] = ((device.number >> 16) & 0xFF) as u8;
                output[3] = device.code.as_u8();
                4
            }
            SlmpCompatibilityMode::Iqr => {
                output[0..4].copy_from_slice(&device.number.to_le_bytes());
                output[4..6].copy_from_slice(&device.code.as_u16().to_le_bytes());
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
        extension: SlmpExtensionSpec,
    ) -> SlmpExtensionSpec {
        let mut result = extension;
        if let Some(extension_specification) = device.extension_specification {
            result.extension_specification = extension_specification;
        }
        if let Some(direct_memory_specification) = device.direct_memory_specification {
            result.direct_memory_specification = direct_memory_specification;
        }
        result
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
                (device.number & 0xFF) as u8,
                ((device.number >> 8) & 0xFF) as u8,
                ((device.number >> 16) & 0xFF) as u8,
                device.code.as_u8(),
                0x00,
                0x00,
                (extension.extension_specification & 0xFF) as u8,
                0x00,
                0xF9,
            ];
        }

        let capture_aligned = matches!(device.code, SlmpDeviceCode::G | SlmpDeviceCode::HG)
            && matches!(extension.direct_memory_specification, 0xF8 | 0xFA);
        let mut device_spec = vec![0u8; device_spec_size(self.options.compatibility_mode)];
        let _ = self.encode_device_spec(device, &mut device_spec);

        if capture_aligned {
            let mut payload = Vec::with_capacity(2 + device_spec.len() + 1 + 1 + 2 + 1);
            payload.push(extension.extension_specification_modification);
            payload.push(extension.device_modification_index);
            payload.extend_from_slice(&device_spec);
            payload.push(extension.device_modification_flags);
            payload.push(0x00);
            payload.extend_from_slice(&extension.extension_specification.to_le_bytes());
            payload.push(extension.direct_memory_specification);
            return payload;
        }

        let mut payload = Vec::with_capacity(2 + 1 + 1 + 1 + device_spec.len() + 1);
        payload.extend_from_slice(&extension.extension_specification.to_le_bytes());
        payload.push(extension.extension_specification_modification);
        payload.push(extension.device_modification_index);
        payload.push(extension.device_modification_flags);
        payload.extend_from_slice(&device_spec);
        payload.push(extension.direct_memory_specification);
        payload
    }

    fn unpack_bit_values(data: &[u8], points: usize) -> Result<Vec<bool>, SlmpError> {
        let need = points.div_ceil(2);
        if data.len() < need {
            return Err(SlmpError::new("read_bits payload size mismatch"));
        }
        let mut result = Vec::with_capacity(points);
        for byte in data.iter().take(need) {
            if result.len() < points {
                result.push(((byte >> 4) & 0x01) != 0);
            }
            if result.len() < points {
                result.push((byte & 0x01) != 0);
            }
        }
        Ok(result)
    }

    fn parse_long_timer_words(
        words: &[u16],
        head_no: u32,
        prefix: &str,
    ) -> Vec<SlmpLongTimerResult> {
        let mut result = Vec::with_capacity(words.len() / 4);
        for (index, chunk) in words.chunks_exact(4).enumerate() {
            let status_word = chunk[2];
            let current_value = chunk[0] as u32 | ((chunk[1] as u32) << 16);
            result.push(SlmpLongTimerResult {
                index: head_no + index as u32,
                device: format!("{prefix}{}", head_no + index as u32),
                current_value,
                contact: (status_word & 0x0002) != 0,
                coil: (status_word & 0x0001) != 0,
                status_word,
                raw_words: chunk.to_vec(),
            });
        }
        result
    }

    fn encode_password(&self, password: &str) -> Result<Vec<u8>, SlmpError> {
        let raw = password.as_bytes();
        match self.options.compatibility_mode {
            SlmpCompatibilityMode::Iqr => {
                if raw.len() < 6 || raw.len() > 32 {
                    return Err(SlmpError::new("iQ-R password length must be 6..32"));
                }
                let mut payload = Vec::with_capacity(2 + raw.len());
                payload.extend_from_slice(&(raw.len() as u16).to_le_bytes());
                payload.extend_from_slice(raw);
                Ok(payload)
            }
            SlmpCompatibilityMode::Legacy => {
                if raw.len() < 6 || raw.len() > 8 {
                    return Err(SlmpError::new("Q/L password length must be 6..8"));
                }
                let mut payload = vec![0u8; 8];
                payload[..raw.len()].copy_from_slice(raw);
                Ok(payload)
            }
        }
    }
}

fn decode_cpu_operation_state(status_word: u16) -> SlmpCpuOperationState {
    let raw_code = (status_word & 0x000F) as u8;
    let status = match raw_code {
        0x00 => SlmpCpuOperationStatus::Run,
        0x02 => SlmpCpuOperationStatus::Stop,
        0x03 => SlmpCpuOperationStatus::Pause,
        _ => SlmpCpuOperationStatus::Unknown,
    };
    SlmpCpuOperationState {
        status,
        raw_status_word: status_word,
        raw_code,
    }
}

impl std::fmt::Debug for SlmpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SlmpClient").finish_non_exhaustive()
    }
}

pub fn encode_device_spec(mode: SlmpCompatibilityMode, device: SlmpDeviceAddress) -> Vec<u8> {
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
