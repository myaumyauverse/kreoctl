//! Packet layer for the Kreo Hive lighting interface.
//!
//! The EVision V2 controller frames every transaction as a 64-byte report on
//! Report ID `0x04`:
//!
//! | Byte  | Meaning                                              |
//! | :---: | :--------------------------------------------------- |
//! | 0     | Report ID (`0x04`)                                   |
//! | 1–2   | 16-bit LE checksum of bytes 3..64                    |
//! | 3     | Command                                              |
//! | 4     | Payload size (≤ 56)                                  |
//! | 5–6   | 16-bit LE address / offset                           |
//! | 7     | Status (0 = OK, in replies only)                     |
//! | 8..64 | Payload                                              |
//!
//! The keyboard echoes each request back with the status byte and, for read
//! commands, the requested data filled in.

use crate::color::Rgb;

/// The exact size in bytes of the vendor-specific HID Output Report.
pub const REPORT_LEN: usize = 64;

/// The HID Output Report ID used for lighting control on interface 0003 (/dev/hidraw2).
pub const REPORT_ID: u8 = 0x04;

/// Maximum payload bytes carried by a single EVision V2 report.
pub const MAX_PAYLOAD: usize = REPORT_LEN - 8;

// EVision V2 commands
pub const CMD_BEGIN: u8 = 0x01;
pub const CMD_END: u8 = 0x02;
pub const CMD_READ_CAPABILITIES: u8 = 0x03;
pub const CMD_READ_CONFIG: u8 = 0x05;
pub const CMD_WRITE_CONFIG: u8 = 0x06;
pub const CMD_READ_CUSTOM_COLORS: u8 = 0x0A;
pub const CMD_WRITE_CUSTOM_COLORS: u8 = 0x0B;
pub const CMD_SEND_DYNAMIC_COLORS: u8 = 0x12;
pub const CMD_END_DYNAMIC_COLORS: u8 = 0x13;

/// Vendor command prefix verified on the Kreo Hive controller.
pub const CMD_PREFIX: [u8; 3] = [0x3b, 0x02, 0x06];

/// Default command header sequence for static color lighting.
pub const CMD_HEADER: [u8; 4] = [0x3b, 0x02, 0x06, 0x22];

/// Parameter header sequence preceding the RGB payload.
pub const PARAM_HEADER: [u8; 3] = [0x06, 0x04, 0x02];

/// Lighting mode byte identifiers for the 4th command byte.
pub const MODE_BYTE_STATIC: u8 = 0x22;
pub const MODE_BYTE_BREATHING: u8 = 0x23;
pub const MODE_BYTE_CYCLE: u8 = 0x24;
pub const MODE_BYTE_WAVE: u8 = 0x25;
pub const MODE_BYTE_REACTIVE: u8 = 0x26;

/// Default static mode byte parameter (brightness / zone level).
pub const MODE_STATIC: u8 = 0x08;

/// Activation / commit flag indicating the controller should apply the payload.
pub const COMMIT_FLAG: u8 = 0x01;

// Verified byte offsets within the 64-byte report payload
pub const OFFSET_REPORT_ID: usize = 0;
pub const OFFSET_CMD_START: usize = 1;
pub const OFFSET_SPEED: usize = 5;
pub const OFFSET_PARAM_START: usize = 9;
pub const OFFSET_RED: usize = 14;
pub const OFFSET_GREEN: usize = 15;
pub const OFFSET_BLUE: usize = 16;
pub const OFFSET_MODE: usize = 17;
pub const OFFSET_COMMIT: usize = 28;

/// Available hardware and software lighting modes.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum LightingMode {
    #[default]
    Static,
    Breathing,
    Cycle,
    Wave,
    Reactive,
    Off,
}

impl LightingMode {
    /// Returns the firmware mode identifier byte sent at offset 4.
    pub fn hardware_byte(&self) -> u8 {
        match self {
            Self::Static | Self::Off => MODE_BYTE_STATIC,
            Self::Breathing => MODE_BYTE_BREATHING,
            Self::Cycle => MODE_BYTE_CYCLE,
            Self::Wave => MODE_BYTE_WAVE,
            Self::Reactive => MODE_BYTE_REACTIVE,
        }
    }

    /// User-friendly display name of the lighting mode.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Static => "Static",
            Self::Breathing => "Breathing",
            Self::Cycle => "Spectrum Cycle",
            Self::Wave => "Rainbow Wave",
            Self::Reactive => "Reactive",
            Self::Off => "Off",
        }
    }
}

/// Comprehensive configuration bundle for a lighting state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightingConfig {
    pub mode: LightingMode,
    pub color: Rgb,
    /// Brightness level from 1 (dimmest) to 5 (maximum).
    pub brightness: u8,
    /// Animation speed level from 1 (slowest) to 5 (fastest).
    pub speed: u8,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            mode: LightingMode::Static,
            color: Rgb::new(255, 255, 255),
            brightness: 4,
            speed: 3,
        }
    }
}

impl LightingConfig {
    pub fn new_static(color: Rgb) -> Self {
        Self {
            mode: LightingMode::Static,
            color,
            brightness: 4,
            speed: 3,
        }
    }

    pub fn new_off() -> Self {
        Self {
            mode: LightingMode::Off,
            color: Rgb::new(0, 0, 0),
            brightness: 1,
            speed: 1,
        }
    }
}

/// Constructs a 64-byte HID Output Report tailored to the specified LightingConfig.
pub fn build_report(config: &LightingConfig) -> [u8; REPORT_LEN] {
    let mut report = [0_u8; REPORT_LEN];

    let effective_color = if config.mode == LightingMode::Off {
        Rgb::new(0, 0, 0)
    } else {
        config.color
    };

    report[OFFSET_REPORT_ID] = REPORT_ID;
    report[OFFSET_CMD_START..OFFSET_CMD_START + 3].copy_from_slice(&CMD_PREFIX);
    report[OFFSET_CMD_START + 3] = config.mode.hardware_byte();

    let speed_clamped = config.speed.clamp(1, 5);
    report[OFFSET_SPEED] = speed_clamped;

    report[OFFSET_PARAM_START..OFFSET_PARAM_START + PARAM_HEADER.len()]
        .copy_from_slice(&PARAM_HEADER);

    report[OFFSET_RED] = effective_color.r;
    report[OFFSET_GREEN] = effective_color.g;
    report[OFFSET_BLUE] = effective_color.b;

    let brightness_clamped = config.brightness.clamp(1, 5);
    report[OFFSET_MODE] = brightness_clamped * 2; // e.g. 4 -> 0x08 (matching verified packet)
    report[OFFSET_COMMIT] = COMMIT_FLAG;

    report
}

/// Constructs a 64-byte HID Output Report to set static backlighting on the Kreo Hive RGB keyboard.
pub fn color_report(color: Rgb) -> [u8; REPORT_LEN] {
    build_report(&LightingConfig::new_static(color))
}

/// Computes and sets the 16-bit little-endian checksum across bytes 3..64.
pub fn compute_checksum(report: &mut [u8; REPORT_LEN]) {
    let mut sum: u16 = 0;
    for &b in &report[3..REPORT_LEN] {
        sum = sum.wrapping_add(b as u16);
    }
    report[1] = (sum & 0xff) as u8;
    report[2] = (sum >> 8) as u8;
}

/// Builds a request report. Write commands carry `data`; read commands pass
/// an empty slice and the number of bytes wanted in `size`.
pub fn build_query(cmd: u8, offset: u16, size: u8, data: &[u8]) -> [u8; REPORT_LEN] {
    let mut report = [0_u8; REPORT_LEN];
    let len = data.len().min(MAX_PAYLOAD);
    report[0] = REPORT_ID;
    report[3] = cmd;
    report[4] = size.min(MAX_PAYLOAD as u8);
    report[5] = (offset & 0xff) as u8;
    report[6] = (offset >> 8) as u8;
    report[8..8 + len].copy_from_slice(&data[..len]);
    compute_checksum(&mut report);
    report
}

/// Builds a write request whose size field matches the payload length.
pub fn build_write(cmd: u8, offset: u16, data: &[u8]) -> [u8; REPORT_LEN] {
    build_query(cmd, offset, data.len().min(MAX_PAYLOAD) as u8, data)
}

/// Builds the 64-byte Begin transaction packet.
pub fn build_begin_packet() -> [u8; REPORT_LEN] {
    build_query(CMD_BEGIN, 0, 0, &[])
}

/// Builds the 64-byte End transaction packet.
pub fn build_end_packet() -> [u8; REPORT_LEN] {
    build_query(CMD_END, 0, 0, &[])
}

/// Returns whether `reply` is the keyboard's answer to `request`.
pub fn is_reply_to(request: &[u8; REPORT_LEN], reply: &[u8; REPORT_LEN]) -> bool {
    reply[0] == REPORT_ID && reply[3] == request[3] && reply[5..7] == request[5..7]
}

/// Validates a reply and returns its payload, or the firmware status code on failure.
pub fn reply_payload<'a>(
    request: &[u8; REPORT_LEN],
    reply: &'a [u8; REPORT_LEN],
) -> Result<&'a [u8], String> {
    if !is_reply_to(request, reply) || reply[1..3] != request[1..3] {
        return Err(format!(
            "unexpected reply to command 0x{:02x} (got cmd 0x{:02x})",
            request[3], reply[3]
        ));
    }
    if reply[7] != 0 {
        return Err(format!(
            "keyboard rejected command 0x{:02x} with status 0x{:02x}",
            request[3], reply[7]
        ));
    }
    let size = (reply[4] as usize).min(MAX_PAYLOAD);
    Ok(&reply[8..8 + size])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_matches_verified_hardware_layout() {
        let report = color_report(Rgb::new(10, 20, 30));

        assert_eq!(report[OFFSET_REPORT_ID], REPORT_ID);
        assert_eq!(
            &report[OFFSET_CMD_START..OFFSET_CMD_START + 4],
            &[0x3b, 0x02, 0x06, 0x22]
        );
        assert_eq!(
            &report[OFFSET_PARAM_START..OFFSET_PARAM_START + 3],
            &[0x06, 0x04, 0x02]
        );
        assert_eq!(report[OFFSET_RED], 10);
        assert_eq!(report[OFFSET_GREEN], 20);
        assert_eq!(report[OFFSET_BLUE], 30);
        assert_eq!(report[OFFSET_MODE], 0x08);
        assert_eq!(report[OFFSET_COMMIT], 0x01);
    }

    #[test]
    fn report_handles_edge_cases() {
        let black = color_report(Rgb::new(0, 0, 0));
        assert_eq!(black[OFFSET_RED], 0);
        assert_eq!(black[OFFSET_GREEN], 0);
        assert_eq!(black[OFFSET_BLUE], 0);

        let white = color_report(Rgb::new(255, 255, 255));
        assert_eq!(white[OFFSET_RED], 255);
        assert_eq!(white[OFFSET_GREEN], 255);
        assert_eq!(white[OFFSET_BLUE], 255);
    }

    #[test]
    fn build_report_supports_modes_and_brightness() {
        let cfg = LightingConfig {
            mode: LightingMode::Breathing,
            color: Rgb::new(255, 0, 0),
            brightness: 5,
            speed: 4,
        };
        let report = build_report(&cfg);
        assert_eq!(report[OFFSET_CMD_START + 3], MODE_BYTE_BREATHING);
        assert_eq!(report[OFFSET_SPEED], 4);
        assert_eq!(report[OFFSET_MODE], 10); // 5 * 2
    }

    #[test]
    fn build_report_turns_off_leds_in_off_mode() {
        let cfg = LightingConfig {
            mode: LightingMode::Off,
            color: Rgb::new(255, 255, 255),
            brightness: 5,
            speed: 3,
        };
        let report = build_report(&cfg);
        assert_eq!(report[OFFSET_RED], 0);
        assert_eq!(report[OFFSET_GREEN], 0);
        assert_eq!(report[OFFSET_BLUE], 0);
    }
}

#[cfg(test)]
mod framing_tests {
    use super::*;

    #[test]
    fn checksum_covers_bytes_after_header() {
        let report = build_write(CMD_WRITE_CONFIG, 0x41, &[0x06, 0x04, 0x02]);
        let sum: u16 = report[3..].iter().map(|&b| b as u16).sum();
        assert_eq!(report[1], (sum & 0xff) as u8);
        assert_eq!(report[2], (sum >> 8) as u8);
        assert_eq!(report[3], CMD_WRITE_CONFIG);
        assert_eq!(report[4], 3);
        assert_eq!(report[5], 0x41);
        assert_eq!(report[6], 0x00);
        assert_eq!(&report[8..11], &[0x06, 0x04, 0x02]);
    }

    #[test]
    fn begin_and_end_packets() {
        let begin = build_begin_packet();
        assert_eq!(begin[0], REPORT_ID);
        assert_eq!(begin[3], CMD_BEGIN);
        assert_eq!(begin[1], CMD_BEGIN); // checksum of a lone command byte

        let end = build_end_packet();
        assert_eq!(end[3], CMD_END);
        assert_eq!(end[1], CMD_END);
    }

    #[test]
    fn reply_validation() {
        let req = build_query(CMD_READ_CONFIG, 0x01, 18, &[]);
        let mut reply = req;
        reply[8] = 0x05;
        assert_eq!(reply_payload(&req, &reply).unwrap()[0], 0x05);

        reply[7] = 0x03;
        assert!(reply_payload(&req, &reply).is_err());

        let other = build_query(CMD_READ_CONFIG, 0x41, 18, &[]);
        assert!(!is_reply_to(&req, &other));
    }
}
