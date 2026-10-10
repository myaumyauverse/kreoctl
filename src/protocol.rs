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
    fn parses_hex_colors_properly() {
        assert_eq!(parse_color("#00ff88"), Some(Rgb::new(0, 255, 136)));
        assert_eq!(parse_color("00ff88"), Some(Rgb::new(0, 255, 136)));
        assert_eq!(parse_color("#ffffff"), Some(Rgb::new(255, 255, 255)));
        assert_eq!(parse_color("#000000"), Some(Rgb::new(0, 0, 0)));
        assert_eq!(parse_color("#AABBCC"), Some(Rgb::new(170, 187, 204)));
        assert_eq!(parse_color("#123456"), Some(Rgb::new(18, 52, 86)));
        assert_eq!(parse_color("  #00ff88  "), Some(Rgb::new(0, 255, 136)));
    }

    #[test]
    fn parses_all_builtin_presets() {
        for (name, expected_color) in PRESETS {
            assert_eq!(
                parse_color(name),
                Some(expected_color),
                "Preset '{name}' failed to parse"
            );
        }
        // Test case-insensitivity on presets
        assert_eq!(parse_color("RED"), Some(Rgb::new(255, 0, 0)));
        assert_eq!(parse_color("Cyan"), Some(Rgb::new(0, 255, 255)));
        assert_eq!(parse_color("oFf"), Some(Rgb::new(0, 0, 0)));
    }

    #[test]
    fn rejects_invalid_color_inputs() {
        assert_eq!(parse_color(""), None);
        assert_eq!(parse_color("   "), None);
        assert_eq!(parse_color("not-a-color"), None);
        assert_eq!(parse_color("bluish"), None);
        assert_eq!(parse_color("#fff"), None); // 3-digit shorthand is not supported
        assert_eq!(parse_color("#00ff88ff"), None); // 8-digit RGBA is not supported
        assert_eq!(parse_color("#GGGGGG"), None); // Non-hex characters
        assert_eq!(parse_color("#12345Z"), None);
        assert_eq!(parse_color("#12345"), None); // 5 digits
        assert_eq!(parse_color("#1234567"), None); // 7 digits
    }

    #[test]
    fn formats_hex_correctly() {
        assert_eq!(Rgb::new(0, 0, 0).hex(), "#000000");
        assert_eq!(Rgb::new(255, 255, 255).hex(), "#ffffff");
        assert_eq!(Rgb::new(1, 2, 3).hex(), "#010203");
        assert_eq!(Rgb::new(0, 255, 136).hex(), "#00ff88");
    }

    #[cfg(feature = "gui")]
    #[test]
    fn egui_float_conversions_roundtrip() {
        let original = Rgb::new(255, 128, 64);
        let floats = original.egui();
        let recovered = Rgb::from_egui(floats);
        assert_eq!(original, recovered);

        // Test clamping on out-of-bounds float inputs
        assert_eq!(Rgb::from_egui([-1.0, 2.0, 0.5]), Rgb::new(0, 255, 128));
    }

    #[test]
    fn report_matches_verified_hardware_layout() {
        let report = color_report(Rgb::new(10, 20, 30));

        // Exact report dimensions
        assert_eq!(report.len(), 64);

        // Report ID
        assert_eq!(report[0], 0x04);

        // Command header
        assert_eq!(&report[1..5], &[0x3b, 0x02, 0x06, 0x22]);

        // Padding before param header
        assert_eq!(&report[5..9], &[0x00, 0x00, 0x00, 0x00]);

        // Param header
        assert_eq!(&report[9..12], &[0x06, 0x04, 0x02]);

        // Padding before RGB
        assert_eq!(&report[12..14], &[0x00, 0x00]);

        // RGB payload
        assert_eq!(&report[14..17], &[10, 20, 30]);

        // Mode parameter
        assert_eq!(report[17], 0x08);

        // Padding before commit byte
        assert_eq!(&report[18..28], &[0_u8; 10]);

        // Commit / activation flag
        assert_eq!(report[28], 0x01);

        // Trailing padding must be all zeros
        assert_eq!(&report[29..64], &[0_u8; 35]);
    }

    #[test]
    fn report_handles_edge_cases() {
        // Pure black / LEDs off
        let off_report = color_report(Rgb::new(0, 0, 0));
        assert_eq!(&off_report[14..17], &[0, 0, 0]);
        assert_eq!(off_report[0], 0x04);
        assert_eq!(off_report[28], 0x01);

        // Pure white
        let white_report = color_report(Rgb::new(255, 255, 255));
        assert_eq!(&white_report[14..17], &[255, 255, 255]);
    }
}
