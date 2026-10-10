#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    #[cfg(feature = "gui")]
    pub fn egui(self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    #[cfg(feature = "gui")]
    pub fn from_egui(value: [f32; 3]) -> Self {
        Self::new(
            (value[0].clamp(0.0, 1.0) * 255.0).round() as u8,
            (value[1].clamp(0.0, 1.0) * 255.0).round() as u8,
            (value[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        )
    }
}

pub const PRESETS: [(&str, Rgb); 10] = [
    ("red", Rgb::new(255, 0, 0)),
    ("green", Rgb::new(0, 255, 0)),
    ("blue", Rgb::new(0, 0, 255)),
    ("white", Rgb::new(255, 255, 255)),
    ("purple", Rgb::new(160, 32, 240)),
    ("cyan", Rgb::new(0, 255, 255)),
    ("magenta", Rgb::new(255, 0, 255)),
    ("orange", Rgb::new(255, 128, 0)),
    ("yellow", Rgb::new(255, 255, 0)),
    ("off", Rgb::new(0, 0, 0)),
];

pub fn parse_color(input: &str) -> Option<Rgb> {
    let normalized = input.trim().to_ascii_lowercase();
    if let Some((_, color)) = PRESETS.iter().find(|(name, _)| *name == normalized) {
        return Some(*color);
    }
    let hex = normalized.strip_prefix('#').unwrap_or(&normalized);
    if hex.len() != 6 {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some(Rgb::new(
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ))
}

/// Packet layout reverse-engineered and verified on a Kreo Hive RGB keyboard.
/// Keep this device-specific: do not reuse it for other VID/PID pairs.
pub fn color_report(color: Rgb) -> [u8; 64] {
    let mut report = [0_u8; 64];
    report[0] = 0x04; // HID report ID
    report[1] = 0x3b;
    report[2] = 0x02;
    report[3] = 0x06;
    report[4] = 0x22;
    report[9] = 0x06;
    report[10] = 0x04;
    report[11] = 0x02;
    report[14] = color.r;
    report[15] = color.g;
    report[16] = color.b;
    report[17] = 0x08;
    report[28] = 0x01;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_and_presets() {
        assert_eq!(parse_color("#00ff88"), Some(Rgb::new(0, 255, 136)));
        assert_eq!(parse_color("red"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(parse_color("not-a-color"), None);
    }

    #[test]
    fn report_contains_rgb_at_verified_offsets() {
        let report = color_report(Rgb::new(1, 2, 3));
        assert_eq!(&report[14..17], &[1, 2, 3]);
        assert_eq!(report[0], 4);
    }
}
