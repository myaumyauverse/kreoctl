use crate::error::KreoError;

/// An 8-bit per channel RGB color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Formats the color as a standard lowercase hexadecimal string (e.g. `#00ff88`).
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

/// The 10 built-in color presets verified for the Kreo Hive keyboard.
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

/// Parses a color string from either a named preset (`red`, `cyan`) or a 6-digit hex code (`#00ff88`, `00ff88`).
pub fn parse_color(input: &str) -> Result<Rgb, KreoError> {
    let normalized = input.trim().to_ascii_lowercase();

    // Check named presets
    if let Some((_, color)) = PRESETS.iter().find(|(name, _)| *name == normalized) {
        return Ok(*color);
    }

    // Strip leading '#' if present
    let hex = normalized.strip_prefix('#').unwrap_or(&normalized);
    if hex.len() != 6 {
        return Err(KreoError::InvalidColor {
            input: input.to_string(),
        });
    }

    let value = u32::from_str_radix(hex, 16).map_err(|_| KreoError::InvalidColor {
        input: input.to_string(),
    })?;

    Ok(Rgb::new(
        (value >> 16) as u8,
        (value >> 8) as u8,
        value as u8,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors_properly() {
        assert_eq!(parse_color("#00ff88").unwrap(), Rgb::new(0, 255, 136));
        assert_eq!(parse_color("00ff88").unwrap(), Rgb::new(0, 255, 136));
        assert_eq!(parse_color("#ffffff").unwrap(), Rgb::new(255, 255, 255));
        assert_eq!(parse_color("#000000").unwrap(), Rgb::new(0, 0, 0));
        assert_eq!(parse_color("#AABBCC").unwrap(), Rgb::new(170, 187, 204));
        assert_eq!(parse_color("#123456").unwrap(), Rgb::new(18, 52, 86));
        assert_eq!(parse_color("  #00ff88  ").unwrap(), Rgb::new(0, 255, 136));
    }

    #[test]
    fn parses_all_builtin_presets() {
        for (name, expected_color) in PRESETS {
            assert_eq!(
                parse_color(name).unwrap(),
                expected_color,
                "Preset '{name}' failed to parse"
            );
        }
        assert_eq!(parse_color("RED").unwrap(), Rgb::new(255, 0, 0));
        assert_eq!(parse_color("Cyan").unwrap(), Rgb::new(0, 255, 255));
        assert_eq!(parse_color("oFf").unwrap(), Rgb::new(0, 0, 0));
    }

    #[test]
    fn rejects_invalid_color_inputs() {
        assert!(parse_color("").is_err());
        assert!(parse_color("   ").is_err());
        assert!(parse_color("not-a-color").is_err());
        assert!(parse_color("bluish").is_err());
        assert!(parse_color("#fff").is_err());
        assert!(parse_color("#00ff88ff").is_err());
        assert!(parse_color("#GGGGGG").is_err());
        assert!(parse_color("#12345Z").is_err());
        assert!(parse_color("#12345").is_err());
        assert!(parse_color("#1234567").is_err());
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
        assert_eq!(Rgb::from_egui([-1.0, 2.0, 0.5]), Rgb::new(0, 255, 128));
    }
}
