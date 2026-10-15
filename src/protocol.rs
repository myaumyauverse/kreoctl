//! Packet layer for the EVision V2 controller used by the Kreo Hive.
//!
//! Every transaction is a 64-byte report on Report ID `0x04`:
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

/// The exact size in bytes of the vendor-specific HID report.
pub const REPORT_LEN: usize = 64;

/// The HID Report ID used for lighting control on interface 1 (Usage Page 0xFF1C).
pub const REPORT_ID: u8 = 0x04;

/// Maximum payload bytes carried by a single report.
pub const MAX_PAYLOAD: usize = REPORT_LEN - 8;

// Commands
pub const CMD_BEGIN: u8 = 0x01;
pub const CMD_END: u8 = 0x02;
pub const CMD_READ_CAPABILITIES: u8 = 0x03;
pub const CMD_READ_CONFIG: u8 = 0x05;
pub const CMD_WRITE_CONFIG: u8 = 0x06;
pub const CMD_READ_CUSTOM_COLORS: u8 = 0x0A;
pub const CMD_WRITE_CUSTOM_COLORS: u8 = 0x0B;
pub const CMD_SEND_DYNAMIC_COLORS: u8 = 0x12;
pub const CMD_END_DYNAMIC_COLORS: u8 = 0x13;

// Config memory layout: byte 0 holds the active onboard profile, followed by
// one 0x40-byte parameter block per profile.
pub const OFFSET_CURRENT_PROFILE: u16 = 0x00;
pub const OFFSET_FIRST_PROFILE: u16 = 0x01;
pub const PROFILE_STRIDE: u16 = 0x40;
pub const MAX_PROFILE_INDEX: u8 = 2;

// Parameter offsets inside a profile block
pub const PARAM_MODE: usize = 0x00;
pub const PARAM_BRIGHTNESS: usize = 0x01;
pub const PARAM_SPEED: usize = 0x02;
pub const PARAM_DIRECTION: usize = 0x03;
pub const PARAM_RANDOM_COLOR: usize = 0x04;
pub const PARAM_MODE_COLOR: usize = 0x05;
pub const PARAM_COLOR_OFFSET: usize = 0x08;
pub const PARAM_LED_MODE: usize = 0x11;
pub const PARAM_CUSTOM_COLORSET: u16 = 0x19;

/// Length of the keyboard lighting parameter block (mode .. led mode).
pub const MODE_BLOCK_LEN: usize = 18;

/// Each of the ten custom colorsets occupies 512 bytes of custom color memory.
pub const CUSTOM_COLORSET_STRIDE: u16 = 512;

/// LED slots used when the keyboard does not report its capabilities
/// (the Hive reports 128: 16 matrix columns x 8 slots).
pub const DEFAULT_MAP_SIZE: usize = 128;
pub const TOTAL_LEDS: usize = DEFAULT_MAP_SIZE;
pub const TOTAL_COLOR_BYTES: usize = TOTAL_LEDS * 3;

/// Brightness range accepted by the firmware (0 = off, 4 = max).
pub const BRIGHTNESS_MAX: u8 = 4;
/// Speed range exposed to users (1 = slowest, 5 = fastest).
pub const SPEED_MIN: u8 = 1;
pub const SPEED_MAX: u8 = 5;
/// Firmware speed value for the slowest setting (0 = fastest).
pub const HW_SPEED_SLOWEST: u8 = 5;

/// Hardware mode bytes of the EVision V2 firmware.
pub const MODE_BYTE_CORRUGATED: u8 = 0x01;
pub const MODE_BYTE_CLOUD: u8 = 0x02;
pub const MODE_BYTE_SERPENTINE: u8 = 0x03;
pub const MODE_BYTE_SPECTRUM: u8 = 0x04;
pub const MODE_BYTE_BREATH: u8 = 0x05;
pub const MODE_BYTE_NORMAL: u8 = 0x06;
pub const MODE_BYTE_REACTION: u8 = 0x07;
pub const MODE_BYTE_RIPPLES: u8 = 0x08;
pub const MODE_BYTE_TRAVERSE: u8 = 0x09;
pub const MODE_BYTE_STARS: u8 = 0x0A;
pub const MODE_BYTE_FLOWERS: u8 = 0x0B;
pub const MODE_BYTE_ROLL: u8 = 0x0C;
pub const MODE_BYTE_WAVE: u8 = 0x0D;
pub const MODE_BYTE_CARTOON: u8 = 0x0E;
pub const MODE_BYTE_RAIN: u8 = 0x0F;
pub const MODE_BYTE_SCAN: u8 = 0x10;
pub const MODE_BYTE_SURMOUNT: u8 = 0x11;
pub const MODE_BYTE_SPEED: u8 = 0x12;
pub const MODE_BYTE_CUSTOM: u8 = 0x14;

// Legacy constants for backward compatibility
pub const MODE_BYTE_STATIC: u8 = MODE_BYTE_NORMAL;
pub const MODE_BYTE_BREATHING: u8 = MODE_BYTE_BREATH;
pub const MODE_BYTE_CYCLE: u8 = MODE_BYTE_SPECTRUM;
pub const MODE_BYTE_REACTIVE: u8 = MODE_BYTE_REACTION;
pub const MODE_BYTE_RIPPLE: u8 = MODE_BYTE_RIPPLES;
pub const MODE_BYTE_STARLIGHT: u8 = MODE_BYTE_STARS;
pub const MODE_BYTE_ACCUMULATE: u8 = MODE_BYTE_CARTOON;
pub const MODE_BYTE_RAINDROPS: u8 = MODE_BYTE_RAIN;
pub const MODE_CUSTOM: u8 = MODE_BYTE_CUSTOM;

// Surmount mode picks one of four fixed colors through the led-mode byte.
const SURMOUNT_RED: u8 = 0x00;
const SURMOUNT_YELLOW: u8 = 0x01;
const SURMOUNT_GREEN: u8 = 0x02;
const SURMOUNT_CYAN: u8 = 0x03;

/// The hardware lighting modes supported natively by the Kreo Hive MCU firmware.
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
#[serde(rename_all = "kebab-case")]
pub enum LightingMode {
    Corrugated,
    Cloud,
    Serpentine,
    #[default]
    #[value(alias = "cycle")]
    #[serde(alias = "cycle")]
    Spectrum,
    #[value(alias = "breathing")]
    #[serde(alias = "breathing")]
    Breath,
    #[value(alias = "static")]
    #[serde(alias = "static")]
    Normal,
    #[value(alias = "reactive")]
    #[serde(alias = "reactive")]
    Reaction,
    #[value(alias = "ripple")]
    #[serde(alias = "ripple")]
    Ripples,
    Traverse,
    #[value(alias = "starlight")]
    #[serde(alias = "starlight")]
    Stars,
    Flowers,
    Roll,
    Wave,
    #[value(alias = "accumulate")]
    #[serde(alias = "accumulate")]
    Cartoon,
    #[value(alias = "raindrops")]
    #[serde(alias = "raindrops")]
    Rain,
    #[value(alias = "visor")]
    #[serde(alias = "visor")]
    Scan,
    Surmount,
    Speed,
    Custom,
    Off,
}

/// Axis along which a mode's direction parameter moves the effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionAxis {
    Horizontal,
    Vertical,
}

#[allow(non_upper_case_globals)]
impl LightingMode {
    // Associated constants for seamless code backwards compatibility
    pub const Static: Self = Self::Normal;
    pub const Breathing: Self = Self::Breath;
    pub const Cycle: Self = Self::Spectrum;
    pub const Reactive: Self = Self::Reaction;
    pub const Ripple: Self = Self::Ripples;
    pub const Starlight: Self = Self::Stars;
    pub const Accumulate: Self = Self::Cartoon;
    pub const Raindrops: Self = Self::Rain;
    pub const Visor: Self = Self::Scan;

    /// Every selectable mode, in the order of the official Kreo software.
    pub const ALL: [Self; 20] = [
        Self::Corrugated,
        Self::Cloud,
        Self::Serpentine,
        Self::Spectrum,
        Self::Breath,
        Self::Normal,
        Self::Reaction,
        Self::Ripples,
        Self::Traverse,
        Self::Stars,
        Self::Flowers,
        Self::Roll,
        Self::Wave,
        Self::Cartoon,
        Self::Rain,
        Self::Scan,
        Self::Surmount,
        Self::Speed,
        Self::Custom,
        Self::Off,
    ];

    /// Returns the firmware mode identifier byte stored at `PARAM_MODE`.
    pub fn hardware_byte(&self) -> u8 {
        match self {
            Self::Corrugated => MODE_BYTE_CORRUGATED,
            Self::Cloud => MODE_BYTE_CLOUD,
            Self::Serpentine => MODE_BYTE_SERPENTINE,
            Self::Spectrum => MODE_BYTE_SPECTRUM,
            Self::Breath => MODE_BYTE_BREATH,
            Self::Normal | Self::Off => MODE_BYTE_NORMAL,
            Self::Reaction => MODE_BYTE_REACTION,
            Self::Ripples => MODE_BYTE_RIPPLES,
            Self::Traverse => MODE_BYTE_TRAVERSE,
            Self::Stars => MODE_BYTE_STARS,
            Self::Flowers => MODE_BYTE_FLOWERS,
            Self::Roll => MODE_BYTE_ROLL,
            Self::Wave => MODE_BYTE_WAVE,
            Self::Cartoon => MODE_BYTE_CARTOON,
            Self::Rain => MODE_BYTE_RAIN,
            Self::Scan => MODE_BYTE_SCAN,
            Self::Surmount => MODE_BYTE_SURMOUNT,
            Self::Speed => MODE_BYTE_SPEED,
            Self::Custom => MODE_BYTE_CUSTOM,
        }
    }

    /// Reconstructs a LightingMode from its raw hardware mode byte.
    pub fn from_hardware_byte(b: u8) -> Self {
        match b {
            MODE_BYTE_CORRUGATED => Self::Corrugated,
            MODE_BYTE_CLOUD => Self::Cloud,
            MODE_BYTE_SERPENTINE => Self::Serpentine,
            MODE_BYTE_SPECTRUM => Self::Spectrum,
            MODE_BYTE_BREATH => Self::Breath,
            MODE_BYTE_REACTION => Self::Reaction,
            MODE_BYTE_RIPPLES => Self::Ripples,
            MODE_BYTE_TRAVERSE => Self::Traverse,
            MODE_BYTE_STARS => Self::Stars,
            MODE_BYTE_FLOWERS => Self::Flowers,
            MODE_BYTE_ROLL => Self::Roll,
            MODE_BYTE_WAVE => Self::Wave,
            MODE_BYTE_CARTOON => Self::Cartoon,
            MODE_BYTE_RAIN => Self::Rain,
            MODE_BYTE_SCAN => Self::Scan,
            MODE_BYTE_SURMOUNT => Self::Surmount,
            MODE_BYTE_SPEED => Self::Speed,
            MODE_BYTE_CUSTOM => Self::Custom,
            _ => Self::Normal,
        }
    }

    /// User-friendly display name of the lighting mode matching official Kreo software.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Corrugated => "Corrugated",
            Self::Cloud => "Cloud",
            Self::Serpentine => "Serpentine",
            Self::Spectrum => "Spectrum",
            Self::Breath => "Breath",
            Self::Normal => "Normal",
            Self::Reaction => "Reaction",
            Self::Ripples => "Ripples",
            Self::Traverse => "Traverse",
            Self::Stars => "Stars",
            Self::Flowers => "Flowers",
            Self::Roll => "Roll",
            Self::Wave => "Wave",
            Self::Cartoon => "Cartoon",
            Self::Rain => "Rain",
            Self::Scan => "Scan",
            Self::Surmount => "Surmount",
            Self::Speed => "Speed",
            Self::Custom => "Custom",
            Self::Off => "Backlight Off",
        }
    }

    /// Whether the firmware animates this mode at a configurable speed.
    pub fn has_speed(&self) -> bool {
        !matches!(
            self,
            Self::Normal | Self::Surmount | Self::Speed | Self::Custom | Self::Off
        )
    }

    /// Whether this mode accepts a user-chosen color.
    pub fn has_color(&self) -> bool {
        !matches!(
            self,
            Self::Spectrum | Self::Flowers | Self::Roll | Self::Speed | Self::Custom | Self::Off
        )
    }

    /// Whether this mode can switch between the user color and random/rainbow colors.
    pub fn has_random(&self) -> bool {
        self.has_color() && *self != Self::Surmount
    }

    /// The axis of the direction parameter, if the mode has one.
    pub fn direction_axis(&self) -> Option<DirectionAxis> {
        match self {
            Self::Corrugated | Self::Cloud | Self::Serpentine | Self::Speed => {
                Some(DirectionAxis::Horizontal)
            }
            Self::Roll => Some(DirectionAxis::Vertical),
            _ => None,
        }
    }

    /// Returns whether this mode shows rainbow colors by default.
    pub fn is_inherently_multicolor(&self) -> bool {
        (!self.has_color() && !matches!(self, Self::Custom | Self::Off))
            || matches!(
                self,
                Self::Corrugated | Self::Cloud | Self::Serpentine | Self::Wave
            )
    }
}

/// Comprehensive configuration bundle for a lighting state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LightingConfig {
    pub mode: LightingMode,
    pub color: Rgb,
    /// Brightness level from 0 (off) to 4 (maximum).
    pub brightness: u8,
    /// Animation speed level from 1 (slowest) to 5 (fastest).
    pub speed: u8,
    /// Direction: 0 = right-to-left (or bottom-to-top), 1 = left-to-right (or top-to-bottom).
    pub direction: u8,
    /// Whether multi-color/rainbow cycling is active instead of solid user color.
    pub multicolor: bool,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            mode: LightingMode::Spectrum,
            color: Rgb::new(255, 240, 5),
            brightness: BRIGHTNESS_MAX,
            speed: 3,
            direction: 0,
            multicolor: true,
        }
    }
}

impl LightingConfig {
    pub fn new_static(color: Rgb) -> Self {
        Self {
            mode: LightingMode::Normal,
            color,
            brightness: BRIGHTNESS_MAX,
            speed: 3,
            direction: 0,
            multicolor: false,
        }
    }

    pub fn new_off() -> Self {
        Self {
            mode: LightingMode::Off,
            color: Rgb::new(0, 0, 0),
            brightness: 0,
            speed: 3,
            direction: 0,
            multicolor: false,
        }
    }
}

/// Converts the user direction (0 = right-to-left) to the firmware value.
pub fn direction_to_hw(direction: u8) -> u8 {
    if direction == 0 { 1 } else { 0 }
}

/// Converts a firmware direction value back to the user direction.
pub fn direction_from_hw(hw: u8) -> u8 {
    if hw == 0 { 1 } else { 0 }
}

/// Converts the user speed (1 = slow .. 5 = fast) to the firmware value (5 = slow .. 0 = fast).
pub fn speed_to_hw(speed: u8) -> u8 {
    SPEED_MAX - speed.clamp(SPEED_MIN, SPEED_MAX)
}

/// Converts a firmware speed value back to the user speed.
pub fn speed_from_hw(hw: u8) -> u8 {
    SPEED_MAX.saturating_sub(hw).clamp(SPEED_MIN, SPEED_MAX)
}

fn surmount_led_mode(color: Rgb) -> u8 {
    let (r, g, b) = (color.r as f32, color.g as f32, color.b as f32);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    if d < 1.0 {
        return SURMOUNT_RED;
    }
    let hue = if max == r {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    match hue {
        h if !(30.0..=300.0).contains(&h) => SURMOUNT_RED,
        h if h <= 90.0 => SURMOUNT_YELLOW,
        h if h <= 150.0 => SURMOUNT_GREEN,
        _ => SURMOUNT_CYAN,
    }
}

fn surmount_color(led_mode: u8) -> Rgb {
    match led_mode {
        SURMOUNT_YELLOW => Rgb::new(255, 255, 0),
        SURMOUNT_GREEN => Rgb::new(0, 255, 0),
        SURMOUNT_CYAN => Rgb::new(0, 255, 255),
        _ => Rgb::new(255, 0, 0),
    }
}

/// Serializes a LightingConfig into the 18-byte keyboard parameter block.
pub fn encode_mode_block(config: &LightingConfig) -> [u8; MODE_BLOCK_LEN] {
    let mut block = [0_u8; MODE_BLOCK_LEN];
    let off = config.mode == LightingMode::Off;

    block[PARAM_MODE] = config.mode.hardware_byte();
    block[PARAM_BRIGHTNESS] = if off {
        0
    } else {
        config.brightness.min(BRIGHTNESS_MAX)
    };
    block[PARAM_SPEED] = speed_to_hw(config.speed);
    block[PARAM_DIRECTION] = direction_to_hw(config.direction);
    block[PARAM_RANDOM_COLOR] = if config.multicolor && config.mode.has_random() {
        0xFF
    } else {
        0
    };
    let color = if off { Rgb::new(0, 0, 0) } else { config.color };
    block[PARAM_MODE_COLOR] = color.r;
    block[PARAM_MODE_COLOR + 1] = color.g;
    block[PARAM_MODE_COLOR + 2] = color.b;
    block[PARAM_COLOR_OFFSET] = 0;
    if config.mode == LightingMode::Surmount {
        block[PARAM_LED_MODE] = surmount_led_mode(config.color);
    }
    block
}

/// Parses an 18-byte keyboard parameter block back into a LightingConfig.
pub fn decode_mode_block(block: &[u8; MODE_BLOCK_LEN]) -> LightingConfig {
    let mut mode = LightingMode::from_hardware_byte(block[PARAM_MODE]);
    let brightness = block[PARAM_BRIGHTNESS].min(BRIGHTNESS_MAX);
    if mode == LightingMode::Normal && brightness == 0 {
        mode = LightingMode::Off;
    }
    let color = if mode == LightingMode::Surmount {
        surmount_color(block[PARAM_LED_MODE])
    } else {
        Rgb::new(
            block[PARAM_MODE_COLOR],
            block[PARAM_MODE_COLOR + 1],
            block[PARAM_MODE_COLOR + 2],
        )
    };
    let multicolor = if mode.has_random() {
        block[PARAM_RANDOM_COLOR] != 0
    } else {
        !mode.has_color() && mode != LightingMode::Custom && mode != LightingMode::Off
    };

    LightingConfig {
        mode,
        color,
        brightness,
        speed: speed_from_hw(block[PARAM_SPEED]),
        direction: direction_from_hw(block[PARAM_DIRECTION]),
        multicolor,
    }
}

/// Returns the config address of a profile's lighting parameter block.
pub fn profile_offset(profile: u8) -> u16 {
    OFFSET_FIRST_PROFILE + profile.min(MAX_PROFILE_INDEX) as u16 * PROFILE_STRIDE
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
    fn profile_blocks_skip_the_profile_index_byte() {
        assert_eq!(profile_offset(0), 0x01);
        assert_eq!(profile_offset(1), 0x41);
        assert_eq!(profile_offset(2), 0x81);
        assert_eq!(profile_offset(9), 0x81);
    }

    #[test]
    fn mode_block_layout() {
        let cfg = LightingConfig {
            mode: LightingMode::Roll,
            color: Rgb::new(255, 128, 64),
            brightness: 3,
            speed: 4,
            direction: 0,
            multicolor: false,
        };
        let block = encode_mode_block(&cfg);
        assert_eq!(block[PARAM_MODE], MODE_BYTE_ROLL);
        assert_eq!(block[PARAM_BRIGHTNESS], 3);
        assert_eq!(block[PARAM_SPEED], 1);
        assert_eq!(block[PARAM_DIRECTION], 1);
        assert_eq!(block[PARAM_RANDOM_COLOR], 0);
        assert_eq!(
            &block[PARAM_MODE_COLOR..PARAM_MODE_COLOR + 3],
            &[255, 128, 64]
        );
    }

    #[test]
    fn random_flag_is_full_byte_and_only_for_supported_modes() {
        let mut cfg = LightingConfig {
            mode: LightingMode::Breath,
            multicolor: true,
            ..Default::default()
        };
        assert_eq!(encode_mode_block(&cfg)[PARAM_RANDOM_COLOR], 0xFF);
        cfg.mode = LightingMode::Spectrum;
        assert_eq!(encode_mode_block(&cfg)[PARAM_RANDOM_COLOR], 0);
    }

    #[test]
    fn normal_mode_solid_color_roundtrip() {
        let cfg = LightingConfig::new_static(Rgb::new(10, 20, 30));
        let block = encode_mode_block(&cfg);
        assert_eq!(block[PARAM_MODE], MODE_BYTE_NORMAL);
        assert_eq!(block[PARAM_RANDOM_COLOR], 0);
        assert_eq!(decode_mode_block(&block), cfg);
    }

    #[test]
    fn mode_block_roundtrip_all_modes() {
        for mode in LightingMode::ALL {
            let cfg = LightingConfig {
                mode,
                color: if mode == LightingMode::Off {
                    Rgb::new(0, 0, 0)
                } else if mode == LightingMode::Surmount {
                    Rgb::new(0, 255, 255)
                } else {
                    Rgb::new(1, 2, 3)
                },
                brightness: if mode == LightingMode::Off { 0 } else { 2 },
                speed: 2,
                direction: 1,
                multicolor: if mode.has_random() {
                    false
                } else {
                    mode.is_inherently_multicolor()
                },
            };
            let decoded = decode_mode_block(&encode_mode_block(&cfg));
            assert_eq!(decoded, cfg, "roundtrip failed for {mode:?}");
        }
    }

    #[test]
    fn off_mode_blacks_out() {
        let block = encode_mode_block(&LightingConfig {
            mode: LightingMode::Off,
            color: Rgb::new(255, 255, 255),
            brightness: 4,
            ..Default::default()
        });
        assert_eq!(block[PARAM_BRIGHTNESS], 0);
        assert_eq!(&block[PARAM_MODE_COLOR..PARAM_MODE_COLOR + 3], &[0, 0, 0]);
    }

    #[test]
    fn speed_and_direction_conversions() {
        assert_eq!(speed_to_hw(5), 0);
        assert_eq!(speed_to_hw(1), 4);
        assert_eq!(speed_from_hw(0), 5);
        assert_eq!(speed_from_hw(5), 1);
        for s in 1..=5 {
            assert_eq!(speed_from_hw(speed_to_hw(s)), s);
        }
        assert_eq!(direction_from_hw(direction_to_hw(0)), 0);
        assert_eq!(direction_from_hw(direction_to_hw(1)), 1);
    }

    #[test]
    fn mode_byte_mapping_and_roundtrip() {
        let expected = [
            (LightingMode::Corrugated, 0x01),
            (LightingMode::Cloud, 0x02),
            (LightingMode::Serpentine, 0x03),
            (LightingMode::Spectrum, 0x04),
            (LightingMode::Breath, 0x05),
            (LightingMode::Normal, 0x06),
            (LightingMode::Reaction, 0x07),
            (LightingMode::Ripples, 0x08),
            (LightingMode::Traverse, 0x09),
            (LightingMode::Stars, 0x0A),
            (LightingMode::Flowers, 0x0B),
            (LightingMode::Roll, 0x0C),
            (LightingMode::Wave, 0x0D),
            (LightingMode::Cartoon, 0x0E),
            (LightingMode::Rain, 0x0F),
            (LightingMode::Scan, 0x10),
            (LightingMode::Surmount, 0x11),
            (LightingMode::Speed, 0x12),
            (LightingMode::Custom, 0x14),
        ];

        for (mode, expected_byte) in expected {
            assert_eq!(mode.hardware_byte(), expected_byte);
            assert_eq!(LightingMode::from_hardware_byte(expected_byte), mode);
        }
    }

    #[test]
    fn reply_validation() {
        let req = build_query(CMD_READ_CONFIG, 0x01, 18, &[]);
        let mut reply = req;
        reply[8] = MODE_BYTE_BREATH;
        assert_eq!(reply_payload(&req, &reply).unwrap()[0], MODE_BYTE_BREATH);

        reply[7] = 0x03;
        assert!(reply_payload(&req, &reply).is_err());

        let other = build_query(CMD_READ_CONFIG, 0x41, 18, &[]);
        assert!(!is_reply_to(&req, &other));
    }
}
