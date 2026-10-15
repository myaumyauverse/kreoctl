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
    fn numbers_selection_works_on_65() {
        let mut model = KeyboardModel::with_variant(KeyboardVariant::Hive65);
        model.select_numbers();
        assert!(model.is_selected("1"));
        assert!(model.is_selected("Esc"));
        assert!(model.is_selected("Backspace"));
    }
}
