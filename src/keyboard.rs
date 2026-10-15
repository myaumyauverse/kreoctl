use crate::color::Rgb;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The physical keyboard model variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyboardVariant {
    Hive75,
    Hive65,
}

impl KeyboardVariant {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hive75 => "Kreo Hive 75%",
            Self::Hive65 => "Kreo Hive 65%",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Hive75 => "HIVE 75%",
            Self::Hive65 => "HIVE 65%",
        }
    }

    pub fn total_rows(&self) -> usize {
        match self {
            Self::Hive75 => 6,
            Self::Hive65 => 5,
        }
    }

    /// Whether the per-key LED map and layout were verified on real hardware.
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::Hive75)
    }

    pub fn key_count_label(&self) -> &'static str {
        match self {
            Self::Hive75 => "83 Keys + Knob",
            Self::Hive65 => "66 Keys + Knob (experimental)",
        }
    }
}

/// Representation of a single physical switch on a Kreo Hive keyboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDefinition {
    pub id: usize,
    pub code: &'static str,
    pub label: &'static str,
    pub row: usize,
    pub width: f32, // in standard key units (1.0 = 1u)
}

/// A complete mechanical keyboard layout with rotary knob.
#[derive(Debug, Clone)]
pub struct KeyboardModel {
    pub variant: KeyboardVariant,
    pub keys: Vec<KeyDefinition>,
    pub per_key_colors: HashMap<String, Rgb>,
    pub selected_keys: Vec<String>,
}

impl Default for KeyboardModel {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardModel {
    /// Creates a default KeyboardModel (Hive 75%).
    pub fn new() -> Self {
        Self::with_variant(KeyboardVariant::Hive75)
    }

    /// Creates a KeyboardModel for the specified physical variant.
    pub fn with_variant(variant: KeyboardVariant) -> Self {
        let keys = match variant {
            KeyboardVariant::Hive75 => build_75_layout(),
            KeyboardVariant::Hive65 => build_65_layout(),
        };
        // Keys without an explicit color follow the main lighting color.
        Self {
            variant,
            keys,
            per_key_colors: HashMap::new(),
            selected_keys: Vec::new(),
        }
    }

    /// Switches the active layout variant, retaining existing key color configurations.
    pub fn set_variant(&mut self, variant: KeyboardVariant) {
        if self.variant != variant {
            self.variant = variant;
            self.keys = match variant {
                KeyboardVariant::Hive75 => build_75_layout(),
                KeyboardVariant::Hive65 => build_65_layout(),
            };
            self.selected_keys.clear();
        }
    }

    /// Replaces all per-key colors with the given key code -> color pairs.
    pub fn load_key_colors(&mut self, colors: impl IntoIterator<Item = (String, Rgb)>) {
        self.per_key_colors = colors.into_iter().collect();
    }

    /// Removes every per-key color so all keys follow the main color again.
    pub fn clear_key_colors(&mut self) {
        self.per_key_colors.clear();
    }

    /// Checks whether a key is currently selected without allocating memory.
    pub fn is_selected(&self, code: &str) -> bool {
        self.selected_keys.iter().any(|k| k == code)
    }

    /// Selects or deselects a key by its code.
    pub fn toggle_selection(&mut self, code: &str) {
        if let Some(pos) = self.selected_keys.iter().position(|k| k == code) {
            self.selected_keys.remove(pos);
        } else {
            self.selected_keys.push(code.to_string());
        }
    }

    /// Selects all keys in the layout.
    pub fn select_all(&mut self) {
        self.selected_keys = self.keys.iter().map(|k| k.code.to_string()).collect();
    }

    /// Clears any selected keys.
    pub fn clear_selection(&mut self) {
        self.selected_keys.clear();
    }

    /// Selects only WASD gaming keys.
    pub fn select_wasd(&mut self) {
        self.selected_keys = vec!["W".into(), "A".into(), "S".into(), "D".into()];
    }

    /// Selects arrow navigation keys.
    pub fn select_arrows(&mut self) {
        self.selected_keys = vec!["Up".into(), "Left".into(), "Down".into(), "Right".into()];
    }

    /// Selects top Function keys.
    pub fn select_f_keys(&mut self) {
        self.selected_keys = (1..=12).map(|i| format!("F{i}")).collect();
    }

    /// Selects all letter keys (A-Z).
    pub fn select_letters(&mut self) {
        self.selected_keys = (b'A'..=b'Z').map(|c| (c as char).to_string()).collect();
    }

    /// Selects number row keys (1-0, -, =, Esc).
    pub fn select_numbers(&mut self) {
        let mut keys: Vec<String> = (0..=9).map(|i| i.to_string()).collect();
        keys.push("Esc".into());
        keys.push("Minus".into());
        keys.push("Equal".into());
        keys.push("Backspace".into());
        self.selected_keys = keys;
    }

    /// Sets the color of all currently selected keys, or all keys if none selected.
    pub fn apply_color_to_selection(&mut self, color: Rgb) {
        if self.selected_keys.is_empty() {
            for k in &self.keys {
                self.per_key_colors.insert(k.code.to_string(), color);
            }
        } else {
            for code in &self.selected_keys {
                self.per_key_colors.insert(code.clone(), color);
            }
        }
    }

    /// Returns the active color of a key.
    pub fn get_key_color(&self, code: &str, fallback: Rgb) -> Rgb {
        self.per_key_colors.get(code).copied().unwrap_or(fallback)
    }

    /// Builds the per-key color buffer (3 bytes per LED slot) for custom hardware upload.
    pub fn build_led_buffer(&self, fallback: Rgb) -> [u8; crate::protocol::TOTAL_COLOR_BYTES] {
        let mut buf = [0_u8; crate::protocol::TOTAL_COLOR_BYTES];
        // Initialize all LED slots with the base / fallback color
        for led in buf.as_chunks_mut::<3>().0 {
            *led = [fallback.r, fallback.g, fallback.b];
        }
        // Map individual configured keys onto their LED slots
        for key in &self.keys {
            let col = self.get_key_color(key.code, fallback);
            for &slot in key_led_slots(self.variant, key.code) {
                if slot < crate::protocol::TOTAL_LEDS {
                    buf[slot * 3..slot * 3 + 3].copy_from_slice(&[col.r, col.g, col.b]);
                }
            }
        }
        buf
    }
}

/// Number of LED slots per matrix column. The firmware numbers LEDs down each
/// column: slot = column * LEDS_PER_COLUMN + row.
const LEDS_PER_COLUMN: usize = 8;

const fn slot(col: usize, row: usize) -> usize {
    col * LEDS_PER_COLUMN + row
}

/// The space bar spans several matrix columns; light all of them.
const SPACE_SLOTS: [usize; 6] = [
    slot(3, 5),
    slot(4, 5),
    slot(5, 5),
    slot(6, 5),
    slot(7, 5),
    slot(8, 5),
];

/// Returns the LED slots lit by a key, verified on a Hive 75 by probing each slot.
///
/// The Hive 65 is assumed to share the 75's matrix for keys both boards have.
pub fn key_led_slots(_variant: KeyboardVariant, code: &str) -> &'static [usize] {
    let (col, row) = match code {
        "Space" => return &SPACE_SLOTS,
        "Knob" => return &[],
        // Row 0
        "Esc" => (0, 0),
        "F1" => (1, 0),
        "F2" => (2, 0),
        "F3" => (3, 0),
        "F4" => (4, 0),
        "F5" => (5, 0),
        "F6" => (6, 0),
        "F7" => (7, 0),
        "F8" => (8, 0),
        "F9" => (9, 0),
        "F10" => (10, 0),
        "F11" => (11, 0),
        "F12" => (12, 0),
        "Del" => (13, 0),
        // Row 1
        "Grave" => (0, 1),
        "1" => (1, 1),
        "2" => (2, 1),
        "3" => (3, 1),
        "4" => (4, 1),
        "5" => (5, 1),
        "6" => (6, 1),
        "7" => (7, 1),
        "8" => (8, 1),
        "9" => (9, 1),
        "0" => (10, 1),
        "Minus" => (11, 1),
        "Equal" => (12, 1),
        "Backspace" => (13, 1),
        // Row 2
        "Tab" => (0, 2),
        "Q" => (1, 2),
        "W" => (2, 2),
        "E" => (3, 2),
        "R" => (4, 2),
        "T" => (5, 2),
        "Y" => (6, 2),
        "U" => (7, 2),
        "I" => (8, 2),
        "O" => (9, 2),
        "P" => (10, 2),
        "LBracket" => (11, 2),
        "RBracket" => (12, 2),
        "Backslash" => (13, 2),
        "Ins" => (14, 2),
        "PgDn" => (15, 2),
        // Row 3
        "CapsLock" => (0, 3),
        "A" => (1, 3),
        "S" => (2, 3),
        "D" => (3, 3),
        "F" => (4, 3),
        "G" => (5, 3),
        "H" => (6, 3),
        "J" => (7, 3),
        "K" => (8, 3),
        "L" => (9, 3),
        "Semicolon" => (10, 3),
        "Quote" => (11, 3),
        "Enter" => (13, 3),
        "End" => (14, 3),
        "PgUp" => (15, 3),
        // Row 4
        "LShift" => (0, 4),
        "Z" => (2, 4),
        "X" => (3, 4),
        "C" => (4, 4),
        "V" => (5, 4),
        "B" => (6, 4),
        "N" => (7, 4),
        "M" => (8, 4),
        "Comma" => (9, 4),
        "Period" => (10, 4),
        "Slash" => (11, 4),
        "RShift" => (13, 4),
        "Up" => (14, 4),
        // Row 5
        "LCtrl" => (0, 5),
        "LWin" => (1, 5),
        "LAlt" => (2, 5),
        "RAlt" => (9, 5),
        "Fn" => (10, 5),
        "RCtrl" => (12, 5),
        "Left" => (13, 5),
        "Down" => (14, 5),
        "Right" => (15, 5),
        _ => return &[],
    };
    let s = slot(col, row);
    &SLOT_TABLE[s..s + 1]
}

/// Every slot index, so single-slot lookups can return a `&'static [usize]`.
const SLOT_TABLE: [usize; crate::protocol::TOTAL_LEDS] = {
    let mut t = [0; crate::protocol::TOTAL_LEDS];
    let mut i = 0;
    while i < t.len() {
        t[i] = i;
        i += 1;
    }
    t
};

/// Returns the first LED slot for a given key code.
pub fn key_code_to_led_index(variant: KeyboardVariant, code: &str) -> Option<usize> {
    key_led_slots(variant, code).first().copied()
}

/// Generates the standard 82-key + knob layout of the Kreo Hive 75%.
fn build_75_layout() -> Vec<KeyDefinition> {
    let mut keys = Vec::with_capacity(84);
    let mut id = 0;

    macro_rules! key {
        ($code:expr, $label:expr, $row:expr, $width:expr) => {{
            keys.push(KeyDefinition {
                id,
                code: $code,
                label: $label,
                row: $row,
                width: $width,
            });
            id += 1;
        }};
    }

    // Row 0: Function row + Delete + Rotary Knob
    key!("Esc", "ESC", 0, 1.0);
    key!("F1", "F1", 0, 1.0);
    key!("F2", "F2", 0, 1.0);
    key!("F3", "F3", 0, 1.0);
    key!("F4", "F4", 0, 1.0);
    key!("F5", "F5", 0, 1.0);
    key!("F6", "F6", 0, 1.0);
    key!("F7", "F7", 0, 1.0);
    key!("F8", "F8", 0, 1.0);
    key!("F9", "F9", 0, 1.0);
    key!("F10", "F10", 0, 1.0);
    key!("F11", "F11", 0, 1.0);
    key!("F12", "F12", 0, 1.0);
    key!("Del", "DEL", 0, 1.0);
    key!("Knob", "KNOB", 0, 1.25); // Rotary volume knob

    // Row 1: Number row + Backspace + Ins
    key!("Grave", "~", 1, 1.0);
    key!("1", "1", 1, 1.0);
    key!("2", "2", 1, 1.0);
    key!("3", "3", 1, 1.0);
    key!("4", "4", 1, 1.0);
    key!("5", "5", 1, 1.0);
    key!("6", "6", 1, 1.0);
    key!("7", "7", 1, 1.0);
    key!("8", "8", 1, 1.0);
    key!("9", "9", 1, 1.0);
    key!("0", "0", 1, 1.0);
    key!("Minus", "-", 1, 1.0);
    key!("Equal", "=", 1, 1.0);
    key!("Backspace", "BACK", 1, 2.0);
    key!("Ins", "INS", 1, 1.0);

    // Row 2: Tab + QWERTY + End
    key!("Tab", "TAB", 2, 1.5);
    key!("Q", "Q", 2, 1.0);
    key!("W", "W", 2, 1.0);
    key!("E", "E", 2, 1.0);
    key!("R", "R", 2, 1.0);
    key!("T", "T", 2, 1.0);
    key!("Y", "Y", 2, 1.0);
    key!("U", "U", 2, 1.0);
    key!("I", "I", 2, 1.0);
    key!("O", "O", 2, 1.0);
    key!("P", "P", 2, 1.0);
    key!("LBracket", "[", 2, 1.0);
    key!("RBracket", "]", 2, 1.0);
    key!("Backslash", "\\", 2, 1.5);
    key!("End", "END", 2, 1.0);

    // Row 3: Caps + Home row + Enter + PgUp
    key!("CapsLock", "CAPS", 3, 1.75);
    key!("A", "A", 3, 1.0);
    key!("S", "S", 3, 1.0);
    key!("D", "D", 3, 1.0);
    key!("F", "F", 3, 1.0);
    key!("G", "G", 3, 1.0);
    key!("H", "H", 3, 1.0);
    key!("J", "J", 3, 1.0);
    key!("K", "K", 3, 1.0);
    key!("L", "L", 3, 1.0);
    key!("Semicolon", ";", 3, 1.0);
    key!("Quote", "'", 3, 1.0);
    key!("Enter", "ENTER", 3, 2.25);
    key!("PgUp", "PGUP", 3, 1.0);

    // Row 4: Shift + Bottom letter row + RShift + Up + PgDn
    key!("LShift", "SHIFT", 4, 2.25);
    key!("Z", "Z", 4, 1.0);
    key!("X", "X", 4, 1.0);
    key!("C", "C", 4, 1.0);
    key!("V", "V", 4, 1.0);
    key!("B", "B", 4, 1.0);
    key!("N", "N", 4, 1.0);
    key!("M", "M", 4, 1.0);
    key!("Comma", ",", 4, 1.0);
    key!("Period", ".", 4, 1.0);
    key!("Slash", "/", 4, 1.0);
    key!("RShift", "SHIFT", 4, 1.75);
    key!("Up", "UP", 4, 1.0);
    key!("PgDn", "PGDN", 4, 1.0);

    // Row 5: Modifier row + Spacebar + Arrows
    key!("LCtrl", "CTRL", 5, 1.25);
    key!("LWin", "WIN", 5, 1.25);
    key!("LAlt", "ALT", 5, 1.25);
    key!("Space", "SPACE", 5, 6.25);
    key!("RAlt", "ALT", 5, 1.0);
    key!("Fn", "FN", 5, 1.0);
    key!("RCtrl", "CTRL", 5, 1.0);
    key!("Left", "LEFT", 5, 1.0);
    key!("Down", "DOWN", 5, 1.0);
    key!("Right", "RIGHT", 5, 1.0);

    let _ = id;
    keys
}

/// Generates the standard 66-key + knob layout of the Kreo Hive 65%.
fn build_65_layout() -> Vec<KeyDefinition> {
    let mut keys = Vec::with_capacity(68);
    let mut id = 0;

    macro_rules! key {
        ($code:expr, $label:expr, $row:expr, $width:expr) => {{
            keys.push(KeyDefinition {
                id,
                code: $code,
                label: $label,
                row: $row,
                width: $width,
            });
            id += 1;
        }};
    }

    // Row 0: Esc + Numbers 1..0, -, =, Backspace + Rotary Knob
    key!("Esc", "ESC", 0, 1.0);
    key!("1", "1", 0, 1.0);
    key!("2", "2", 0, 1.0);
    key!("3", "3", 0, 1.0);
    key!("4", "4", 0, 1.0);
    key!("5", "5", 0, 1.0);
    key!("6", "6", 0, 1.0);
    key!("7", "7", 0, 1.0);
    key!("8", "8", 0, 1.0);
    key!("9", "9", 0, 1.0);
    key!("0", "0", 0, 1.0);
    key!("Minus", "-", 0, 1.0);
    key!("Equal", "=", 0, 1.0);
    key!("Backspace", "BACK", 0, 2.0);
    key!("Knob", "KNOB", 0, 1.25); // Top-right rotary volume knob

    // Row 1: Tab + QWERTY + Delete
    key!("Tab", "TAB", 1, 1.5);
    key!("Q", "Q", 1, 1.0);
    key!("W", "W", 1, 1.0);
    key!("E", "E", 1, 1.0);
    key!("R", "R", 1, 1.0);
    key!("T", "T", 1, 1.0);
    key!("Y", "Y", 1, 1.0);
    key!("U", "U", 1, 1.0);
    key!("I", "I", 1, 1.0);
    key!("O", "O", 1, 1.0);
    key!("P", "P", 1, 1.0);
    key!("LBracket", "[", 1, 1.0);
    key!("RBracket", "]", 1, 1.0);
    key!("Backslash", "\\", 1, 1.5);
    key!("Del", "DEL", 1, 1.0);

    // Row 2: Caps + Home row + Enter + PgUp
    key!("CapsLock", "CAPS", 2, 1.75);
    key!("A", "A", 2, 1.0);
    key!("S", "S", 2, 1.0);
    key!("D", "D", 2, 1.0);
    key!("F", "F", 2, 1.0);
    key!("G", "G", 2, 1.0);
    key!("H", "H", 2, 1.0);
    key!("J", "J", 2, 1.0);
    key!("K", "K", 2, 1.0);
    key!("L", "L", 2, 1.0);
    key!("Semicolon", ";", 2, 1.0);
    key!("Quote", "'", 2, 1.0);
    key!("Enter", "ENTER", 2, 2.25);
    key!("PgUp", "PGUP", 2, 1.0);

    // Row 3: Shift + Bottom alpha + RShift + Up + PgDn
    key!("LShift", "SHIFT", 3, 2.25);
    key!("Z", "Z", 3, 1.0);
    key!("X", "X", 3, 1.0);
    key!("C", "C", 3, 1.0);
    key!("V", "V", 3, 1.0);
    key!("B", "B", 3, 1.0);
    key!("N", "N", 3, 1.0);
    key!("M", "M", 3, 1.0);
    key!("Comma", ",", 3, 1.0);
    key!("Period", ".", 3, 1.0);
    key!("Slash", "/", 3, 1.0);
    key!("RShift", "SHIFT", 3, 1.75);
    key!("Up", "UP", 3, 1.0);
    key!("PgDn", "PGDN", 3, 1.0);

    // Row 4: Modifiers + Space + Left, Down, Right
    key!("LCtrl", "CTRL", 4, 1.25);
    key!("LWin", "WIN", 4, 1.25);
    key!("LAlt", "ALT", 4, 1.25);
    key!("Space", "SPACE", 4, 6.25);
    key!("RAlt", "ALT", 4, 1.0);
    key!("Fn", "FN", 4, 1.0);
    key!("Left", "LEFT", 4, 1.0);
    key!("Down", "DOWN", 4, 1.0);
    key!("Right", "RIGHT", 4, 1.0);

    let _ = id;
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_has_expected_key_count() {
        let model_75 = KeyboardModel::with_variant(KeyboardVariant::Hive75);
        // 75% layout has 82 keys + 1 knob = 83 items
        assert_eq!(model_75.keys.len(), 83);
        assert_eq!(model_75.variant.total_rows(), 6);

        let model_65 = KeyboardModel::with_variant(KeyboardVariant::Hive65);
        // 65% exploded layout has 66 keys + 1 knob = 67 items
        assert_eq!(model_65.keys.len(), 67);
        assert_eq!(model_65.variant.total_rows(), 5);
    }

    #[test]
    fn wasd_selection_works() {
        let mut model = KeyboardModel::new();
        model.select_wasd();
        assert_eq!(model.selected_keys.len(), 4);
        assert!(model.selected_keys.contains(&"W".to_string()));
    }

    #[test]
    fn variant_switching_preserves_colors() {
        let mut model = KeyboardModel::with_variant(KeyboardVariant::Hive75);
        model
            .per_key_colors
            .insert("Q".into(), Rgb::new(255, 0, 128));
        model.set_variant(KeyboardVariant::Hive65);
        assert_eq!(model.variant, KeyboardVariant::Hive65);
        assert_eq!(
            model.get_key_color("Q", Rgb::new(0, 0, 0)),
            Rgb::new(255, 0, 128)
        );
        assert_eq!(model.keys.len(), 67);
    }

    #[test]
    fn unpainted_keys_use_fallback_color() {
        let mut model = KeyboardModel::new();
        model.select_wasd();
        model.apply_color_to_selection(Rgb::new(255, 0, 0));
        let base = Rgb::new(0, 0, 255);
        let buf = model.build_led_buffer(base);
        let w = key_code_to_led_index(model.variant, "W").unwrap() * 3;
        let q = key_code_to_led_index(model.variant, "Q").unwrap() * 3;
        assert_eq!(&buf[w..w + 3], &[255, 0, 0]);
        assert_eq!(&buf[q..q + 3], &[0, 0, 255]);
    }

    #[test]
    fn hive75_map_matches_probed_hardware() {
        let v = KeyboardVariant::Hive75;
        // Verified by lighting single slots on a real Hive 75
        assert_eq!(key_code_to_led_index(v, "Grave"), Some(1));
        assert_eq!(key_code_to_led_index(v, "Q"), Some(10));
        assert_eq!(key_code_to_led_index(v, "F10"), Some(80));
        assert_eq!(key_code_to_led_index(v, "Ins"), Some(114));
        assert_eq!(key_code_to_led_index(v, "PgUp"), Some(123));
        assert_eq!(key_code_to_led_index(v, "RCtrl"), Some(101));
        assert_eq!(key_led_slots(v, "Space").len(), 6);

        // Every key except the knob has a unique LED slot
        let model = KeyboardModel::with_variant(v);
        let mut seen = std::collections::HashSet::new();
        for k in model.keys.iter().filter(|k| k.code != "Knob") {
            for &s in key_led_slots(v, k.code) {
                assert!(seen.insert(s), "slot {s} reused by {}", k.code);
            }
            assert!(
                !key_led_slots(v, k.code).is_empty(),
                "{} has no LED",
                k.code
            );
        }
    }

    #[test]
    fn numbers_selection_works_on_65() {
        let mut model = KeyboardModel::with_variant(KeyboardVariant::Hive65);
        model.select_numbers();
        assert!(model.is_selected("1"));
        assert!(model.is_selected("Esc"));
        assert!(model.is_selected("Backspace"));
    }
}
