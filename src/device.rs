use crate::backend::{DeviceBackend, HidBackend};
use crate::color::Rgb;
use crate::error::KreoError;
use crate::protocol::{self, LightingConfig, LightingMode, MAX_PAYLOAD, REPORT_LEN};
use hidapi::HidApi;

pub const VENDOR_ID: u16 = 0x320f;
pub const PRODUCT_ID: u16 = 0x5055;
pub const RGB_INTERFACE: i32 = 1; // USB interface 1 -> hidraw vendor lighting channel

/// Discover and open the keyboard's vendor lighting interface (Usage Page 0xFF1C).
pub fn open_device() -> Result<HidBackend, KreoError> {
    let api = HidApi::new().map_err(|e| KreoError::Hid(e.to_string()))?;
    let candidates: Vec<_> = api
        .device_list()
        .filter(|info| info.vendor_id() == VENDOR_ID && info.product_id() == PRODUCT_ID)
        .collect();

    if candidates.is_empty() {
        return Err(KreoError::DeviceNotFound);
    }

    let info = candidates
        .iter()
        .find(|info| info.usage_page() == 0xff1c)
        .or_else(|| {
            candidates
                .iter()
                .find(|info| info.interface_number() == RGB_INTERFACE)
        })
        .ok_or(KreoError::InterfaceNotFound {
            interface: RGB_INTERFACE,
        })?;

    let device = api.open_path(info.path()).map_err(|e| {
        let err_msg = e.to_string();
        if err_msg.to_lowercase().contains("permission")
            || err_msg.to_lowercase().contains("access")
        {
            KreoError::PermissionDenied(err_msg)
        } else {
            KreoError::Hid(err_msg)
        }
    })?;

    Ok(HidBackend::new(device))
}

/// Sends one request and waits for the matching reply, returning its payload.
pub fn query(
    backend: &mut dyn DeviceBackend,
    request: &[u8; REPORT_LEN],
) -> Result<Vec<u8>, KreoError> {
    backend.write_report(request)?;
    // The interface may also deliver unrelated input reports; skip until our echo arrives.
    for _ in 0..8 {
        let reply = backend.read_report()?;
        if protocol::is_reply_to(request, &reply) {
            return protocol::reply_payload(request, &reply)
                .map(<[u8]>::to_vec)
                .map_err(KreoError::Protocol);
        }
    }
    Err(KreoError::Protocol(format!(
        "no reply to command 0x{:02x}",
        request[3]
    )))
}

/// Reads `len` bytes of memory addressed by a read command, in 56-byte chunks.
fn read_chunked(
    backend: &mut dyn DeviceBackend,
    cmd: u8,
    offset: u16,
    len: usize,
) -> Result<Vec<u8>, KreoError> {
    let mut out = Vec::with_capacity(len);
    while out.len() < len {
        let want = (len - out.len()).min(MAX_PAYLOAD) as u8;
        let request = protocol::build_query(cmd, offset + out.len() as u16, want, &[]);
        let mut payload = query(backend, &request)?;
        if payload.is_empty() {
            return Err(KreoError::Protocol(format!(
                "empty reply reading 0x{offset:04x}"
            )));
        }
        payload.truncate(want as usize);
        out.extend_from_slice(&payload);
    }
    Ok(out)
}

/// Writes `data` to memory addressed by a write command, in 56-byte chunks.
fn write_chunked(
    backend: &mut dyn DeviceBackend,
    cmd: u8,
    offset: u16,
    data: &[u8],
) -> Result<(), KreoError> {
    for (i, chunk) in data.chunks(MAX_PAYLOAD).enumerate() {
        let request = protocol::build_write(cmd, offset + (i * MAX_PAYLOAD) as u16, chunk);
        query(backend, &request)?;
    }
    Ok(())
}

fn begin(backend: &mut dyn DeviceBackend) -> Result<(), KreoError> {
    query(backend, &protocol::build_begin_packet()).map(|_| ())
}

fn end(backend: &mut dyn DeviceBackend) -> Result<(), KreoError> {
    query(backend, &protocol::build_end_packet()).map(|_| ())
}

/// Runs `body` inside a Begin/End configuration transaction, always closing it.
fn transaction<T>(
    backend: &mut dyn DeviceBackend,
    body: impl FnOnce(&mut dyn DeviceBackend) -> Result<T, KreoError>,
) -> Result<T, KreoError> {
    begin(backend)?;
    let result = body(backend);
    let closed = end(backend);
    let value = result?;
    closed?;
    Ok(value)
}

/// Returns the onboard profile currently selected on the keyboard (0..=2).
pub fn read_active_profile(backend: &mut dyn DeviceBackend) -> Result<u8, KreoError> {
    let data = read_chunked(
        backend,
        protocol::CMD_READ_CONFIG,
        protocol::OFFSET_CURRENT_PROFILE,
        1,
    )?;
    Ok(data[0])
}

/// Reads the profile index, repairing it if it holds an out-of-range value.
fn active_profile_for_write(backend: &mut dyn DeviceBackend) -> Result<u8, KreoError> {
    let profile = read_active_profile(backend)?;
    if profile > protocol::MAX_PROFILE_INDEX {
        write_chunked(
            backend,
            protocol::CMD_WRITE_CONFIG,
            protocol::OFFSET_CURRENT_PROFILE,
            &[0],
        )?;
        return Ok(0);
    }
    Ok(profile)
}

/// Returns the number of LED slots in the keyboard's color memory.
pub fn read_led_map_size(backend: &mut dyn DeviceBackend) -> usize {
    match read_chunked(backend, protocol::CMD_READ_CAPABILITIES, 0, 7) {
        Ok(caps) if caps[0] == 0xAA && caps[1] == 0x55 && caps[5] > 0 => caps[5] as usize,
        _ => protocol::DEFAULT_MAP_SIZE,
    }
}

/// Apply a static color to the physical keyboard using the auto-discovered HID backend.
pub fn set_color(color: Rgb) -> Result<(), KreoError> {
    let cfg = LightingConfig::new_static(color);
    set_lighting(&cfg)
}

/// Apply a static color using an explicit backend (useful for testing and dependency injection).
pub fn set_color_with_backend(
    backend: &mut dyn DeviceBackend,
    color: Rgb,
) -> Result<(), KreoError> {
    let cfg = LightingConfig::new_static(color);
    set_lighting_with_backend(backend, &cfg)
}

/// Apply an arbitrary LightingConfig (mode, speed, brightness, color) to the keyboard.
pub fn set_lighting(config: &LightingConfig) -> Result<(), KreoError> {
    let mut backend = open_device()?;
    set_lighting_with_backend(&mut backend, config)
}

/// Apply a LightingConfig using an explicit backend.
pub fn set_lighting_with_backend(
    backend: &mut dyn DeviceBackend,
    config: &LightingConfig,
) -> Result<(), KreoError> {
    let block = protocol::encode_mode_block(config);
    transaction(backend, |backend| {
        let profile = active_profile_for_write(backend)?;
        let offset = protocol::profile_offset(profile);
        write_chunked(backend, protocol::CMD_WRITE_CONFIG, offset, &block)?;
        if config.mode == LightingMode::Custom {
            write_chunked(
                backend,
                protocol::CMD_WRITE_CONFIG,
                offset + protocol::PARAM_CUSTOM_COLORSET,
                &[0],
            )?;
        }
        Ok(())
    })
}

/// Apply custom per-key colors (3 bytes per LED slot) to the hardware keyboard.
pub fn set_per_key_colors(color_buffer: &[u8], brightness: u8) -> Result<(), KreoError> {
    let mut backend = open_device()?;
    set_per_key_colors_with_backend(&mut backend, color_buffer, brightness)
}

/// Uploads per-key colors into custom colorset 1 and switches the keyboard to Custom mode.
pub fn set_per_key_colors_with_backend(
    backend: &mut dyn DeviceBackend,
    color_buffer: &[u8],
    brightness: u8,
) -> Result<(), KreoError> {
    let map_size = read_led_map_size(backend);
    let mut colors = color_buffer.to_vec();
    colors.resize(map_size * 3, 0);

    transaction(backend, |backend| {
        write_chunked(backend, protocol::CMD_WRITE_CUSTOM_COLORS, 0, &colors)
    })?;

    let cfg = LightingConfig {
        mode: LightingMode::Custom,
        brightness,
        multicolor: false,
        ..LightingConfig::default()
    };
    set_lighting_with_backend(backend, &cfg)
}

/// Query the keyboard directly for its current onboard lighting state.
pub fn read_current_lighting() -> Result<LightingConfig, KreoError> {
    let mut backend = open_device()?;
    read_current_lighting_with_backend(&mut backend)
}

/// Query the keyboard state with an explicit backend.
pub fn read_current_lighting_with_backend(
    backend: &mut dyn DeviceBackend,
) -> Result<LightingConfig, KreoError> {
    let profile = read_active_profile(backend)?.min(protocol::MAX_PROFILE_INDEX);
    let data = read_chunked(
        backend,
        protocol::CMD_READ_CONFIG,
        protocol::profile_offset(profile),
        protocol::MODE_BLOCK_LEN,
    )?;
    let mut block = [0_u8; protocol::MODE_BLOCK_LEN];
    block.copy_from_slice(&data);
    Ok(protocol::decode_mode_block(&block))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;
    use crate::protocol::{
        CMD_BEGIN, CMD_END, CMD_READ_CAPABILITIES, CMD_READ_CONFIG, CMD_WRITE_CONFIG,
        CMD_WRITE_CUSTOM_COLORS, MODE_BYTE_BREATH, MODE_BYTE_CUSTOM, MODE_BYTE_NORMAL,
    };

    fn commands(mock: &MockBackend) -> Vec<u8> {
        mock.written_reports.iter().map(|r| r[3]).collect()
    }

    #[test]
    fn set_color_writes_block_into_active_profile() {
        let mut mock = MockBackend::new();
        let color = Rgb::new(0, 128, 255);

        set_color_with_backend(&mut mock, color).unwrap();
        assert_eq!(
            commands(&mock),
            vec![CMD_BEGIN, CMD_READ_CONFIG, CMD_WRITE_CONFIG, CMD_END]
        );
        let write = &mock.written_reports[2];
        assert_eq!(write[4] as usize, protocol::MODE_BLOCK_LEN);
        assert_eq!(write[5], 0x01); // profile 0 block, never the profile-index byte
        assert_eq!(write[8], MODE_BYTE_NORMAL);
        assert_eq!(write[8 + protocol::PARAM_RANDOM_COLOR], 0);
        assert_eq!(&write[13..16], &[0, 128, 255]);
    }

    #[test]
    fn set_lighting_encodes_speed_brightness_direction() {
        let mut mock = MockBackend::new();
        let cfg = LightingConfig {
            mode: LightingMode::Breath,
            color: Rgb::new(255, 100, 0),
            brightness: 4,
            speed: 4,
            direction: 1,
            multicolor: false,
        };

        set_lighting_with_backend(&mut mock, &cfg).unwrap();
        let write = &mock.written_reports[2];
        assert_eq!(write[8], MODE_BYTE_BREATH);
        assert_eq!(write[9], 4); // brightness
        assert_eq!(write[10], 1); // hw speed (5 - 4)
        assert_eq!(write[11], 0); // left-to-right
        assert_eq!(&write[13..16], &[255, 100, 0]);
    }

    #[test]
    fn corrupted_profile_index_is_repaired() {
        let mut mock = MockBackend::new();
        let mut reply = protocol::build_query(CMD_BEGIN, 0, 0, &[]);
        mock.queue_read_report(reply);
        reply = protocol::build_query(CMD_READ_CONFIG, 0, 1, &[]);
        reply[8] = 0x06; // what the old buggy writer left behind
        mock.queue_read_report(reply);

        set_color_with_backend(&mut mock, Rgb::new(1, 2, 3)).unwrap();
        assert_eq!(
            commands(&mock),
            vec![
                CMD_BEGIN,
                CMD_READ_CONFIG,
                CMD_WRITE_CONFIG,
                CMD_WRITE_CONFIG,
                CMD_END
            ]
        );
        assert_eq!(mock.written_reports[2][5], 0x00);
        assert_eq!(mock.written_reports[2][8], 0x00);
        assert_eq!(mock.written_reports[3][5], 0x01);
    }

    #[test]
    fn per_key_upload_uses_custom_color_memory() {
        let mut mock = MockBackend::new();
        let buf = [0x55_u8; protocol::TOTAL_COLOR_BYTES];

        set_per_key_colors_with_backend(&mut mock, &buf, 3).unwrap();
        let cmds = commands(&mock);
        assert_eq!(cmds[0], CMD_READ_CAPABILITIES);
        assert_eq!(cmds[1], CMD_BEGIN);
        let chunks = protocol::TOTAL_COLOR_BYTES.div_ceil(MAX_PAYLOAD);
        for c in &cmds[2..2 + chunks] {
            assert_eq!(*c, CMD_WRITE_CUSTOM_COLORS);
        }
        assert_eq!(cmds[2 + chunks], CMD_END);

        let mode_write = mock
            .written_reports
            .iter()
            .find(|r| r[3] == CMD_WRITE_CONFIG && r[5] == 0x01)
            .unwrap();
        assert_eq!(mode_write[8], MODE_BYTE_CUSTOM);
        assert_eq!(mode_write[9], 3);
        let colorset = mock.written_reports.iter().find(|r| {
            r[3] == CMD_WRITE_CONFIG && r[5] as u16 == 0x01 + protocol::PARAM_CUSTOM_COLORSET
        });
        assert!(colorset.is_some());
    }

    #[test]
    fn read_back_decodes_profile_block() {
        let mut mock = MockBackend::new();
        let mut reply = protocol::build_query(CMD_READ_CONFIG, 0, 1, &[]);
        reply[8] = 1; // profile 2
        mock.queue_read_report(reply);
        let block = protocol::encode_mode_block(&LightingConfig {
            mode: LightingMode::Wave,
            color: Rgb::new(9, 8, 7),
            brightness: 2,
            speed: 5,
            direction: 1,
            multicolor: true,
        });
        let req = protocol::build_query(CMD_READ_CONFIG, 0x41, 18, &[]);
        let mut reply = req;
        reply[8..26].copy_from_slice(&block);
        mock.queue_read_report(reply);

        let cfg = read_current_lighting_with_backend(&mut mock).unwrap();
        assert_eq!(cfg.mode, LightingMode::Wave);
        assert_eq!(cfg.color, Rgb::new(9, 8, 7));
        assert_eq!(cfg.speed, 5);
        assert!(cfg.multicolor);
    }

    #[test]
    fn failing_backend_reports_error() {
        let mut mock = MockBackend::new();
        mock.simulate_failure = true;
        assert!(set_color_with_backend(&mut mock, Rgb::new(1, 1, 1)).is_err());
    }
}
