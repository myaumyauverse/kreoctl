use crate::color::Rgb;
use crate::protocol::LightingMode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Keystroke press or release event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyActionType {
    Press,
    Release,
}

/// Mouse movement direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseDirection {
    Left,
    Right,
    Top,
    Down,
}

/// A discrete recorded step in a hardware macro sequence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MacroStep {
    Key {
        key: String,
        action: KeyActionType,
        delay_ms: u32,
    },
    MouseMove {
        direction: MouseDirection,
        distance: i32,
        delay_ms: u32,
    },
}

impl MacroStep {
    pub fn delay_ms(&self) -> u32 {
        match self {
            Self::Key { delay_ms, .. } | Self::MouseMove { delay_ms, .. } => *delay_ms,
        }
    }

    pub fn set_delay_ms(&mut self, delay: u32) {
        match self {
            Self::Key { delay_ms, .. } | Self::MouseMove { delay_ms, .. } => *delay_ms = delay,
        }
    }
}

/// A named macro containing a recorded sequence of keystrokes and mouse movements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroDefinition {
    pub id: String,
    pub name: String,
    pub steps: Vec<MacroStep>,
}

/// Key remapping assignment options.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum KeyRemap {
    #[default]
    Default,
    Key(String),
    Media(String),
    Macro(String),
    Disabled,
}

/// A user profile containing lighting modes, colors, button remaps, and macros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub mode: LightingMode,
    pub color: Rgb,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8, // 0 = From right to left, 1 = From left to right
    #[serde(default)]
    pub multicolor: bool,
    pub per_key_colors: HashMap<String, String>, // Key code -> Hex string
    #[serde(default)]
    pub key_remaps: HashMap<String, KeyRemap>,
    #[serde(default)]
    pub macros: Vec<MacroDefinition>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Profile 1".into(),
            mode: LightingMode::Spectrum,
            color: Rgb::new(255, 240, 5),
            brightness: 4,
            speed: 3,
            direction: 1,
            multicolor: true,
            per_key_colors: HashMap::new(),
            key_remaps: HashMap::new(),
            macros: vec![MacroDefinition {
                id: "macro_1".into(),
                name: "macro_1".into(),
                steps: vec![
                    MacroStep::Key {
                        key: "F".into(),
                        action: KeyActionType::Press,
                        delay_ms: 16,
                    },
                    MacroStep::Key {
                        key: "K".into(),
                        action: KeyActionType::Press,
                        delay_ms: 16,
                    },
                    MacroStep::Key {
                        key: "S".into(),
                        action: KeyActionType::Press,
                        delay_ms: 31,
                    },
                    MacroStep::Key {
                        key: "S".into(),
                        action: KeyActionType::Release,
                        delay_ms: 16,
                    },
                    MacroStep::Key {
                        key: "F".into(),
                        action: KeyActionType::Release,
                        delay_ms: 15,
                    },
                    MacroStep::Key {
                        key: "J".into(),
                        action: KeyActionType::Release,
                        delay_ms: 16,
                    },
                    MacroStep::Key {
                        key: "D".into(),
                        action: KeyActionType::Release,
                        delay_ms: 31,
                    },
                ],
            }],
        }
    }
}

/// Profile management system allowing switching, adding, and editing profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStore {
    pub active_index: usize,
    pub profiles: Vec<Profile>,
}

impl Default for ProfileStore {
    fn default() -> Self {
        let default_profile = Profile {
            name: "Profile 1".into(),
            ..Default::default()
        };

        let gaming_profile = Profile {
            name: "Gaming WASD".into(),
            mode: LightingMode::Static,
            color: Rgb::new(0, 255, 136),
            brightness: 4,
            speed: 3,
            direction: 1,
            multicolor: false,
            per_key_colors: {
                let mut map = HashMap::new();
                map.insert("W".into(), "#ff0000".into());
                map.insert("A".into(), "#ff0000".into());
                map.insert("S".into(), "#ff0000".into());
                map.insert("D".into(), "#ff0000".into());
                map
            },
            key_remaps: HashMap::new(),
            macros: Vec::new(),
        };

        Self {
            active_index: 0,
            profiles: vec![default_profile, gaming_profile],
        }
    }
}

impl ProfileStore {
    pub fn active_profile(&self) -> &Profile {
        self.profiles
            .get(self.active_index)
            .unwrap_or(&self.profiles[0])
    }

    pub fn active_profile_mut(&mut self) -> &mut Profile {
        if self.active_index >= self.profiles.len() {
            self.active_index = 0;
        }
        &mut self.profiles[self.active_index]
    }

    pub fn add_profile(&mut self, name: &str) {
        let mut new_p = self.active_profile().clone();
        new_p.name = name.to_string();
        self.profiles.push(new_p);
        self.active_index = self.profiles.len() - 1;
    }

    pub fn delete_profile(&mut self, index: usize) {
        if self.profiles.len() > 1 && index < self.profiles.len() {
            self.profiles.remove(index);
            if self.active_index >= self.profiles.len() {
                self.active_index = self.profiles.len() - 1;
            }
        }
    }

    pub fn rename_profile(&mut self, index: usize, new_name: &str) {
        if let Some(p) = self.profiles.get_mut(index) {
            p.name = new_name.to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_store_lifecycle() {
        let mut store = ProfileStore::default();
        assert_eq!(store.profiles.len(), 2);

        store.add_profile("Cyberpunk");
        assert_eq!(store.profiles.len(), 3);
        assert_eq!(store.active_profile().name, "Cyberpunk");

        store.delete_profile(2);
        assert_eq!(store.profiles.len(), 2);
    }

    #[test]
    fn test_remap_and_macro_serialization() {
        let mut prof = Profile::default();
        prof.key_remaps
            .insert("Caps Lock".into(), KeyRemap::Key("Left Ctrl".into()));
        prof.key_remaps
            .insert("F1".into(), KeyRemap::Media("Mute".into()));
        prof.key_remaps
            .insert("G".into(), KeyRemap::Macro("burst_fire".into()));

        let macro_def = MacroDefinition {
            id: "burst_fire".into(),
            name: "Burst Fire".into(),
            steps: vec![
                MacroStep::Key {
                    key: "MouseLeft".into(),
                    action: KeyActionType::Press,
                    delay_ms: 12,
                },
                MacroStep::Key {
                    key: "MouseLeft".into(),
                    action: KeyActionType::Release,
                    delay_ms: 50,
                },
                MacroStep::MouseMove {
                    direction: MouseDirection::Down,
                    distance: 15,
                    delay_ms: 10,
                },
            ],
        };
        prof.macros.clear();
        prof.macros.push(macro_def);

        let toml_str = toml::to_string(&prof).expect("serialization failed");
        let deserialized: Profile = toml::from_str(&toml_str).expect("deserialization failed");

        assert_eq!(
            deserialized.key_remaps.get("Caps Lock"),
            Some(&KeyRemap::Key("Left Ctrl".into()))
        );
        assert_eq!(
            deserialized.key_remaps.get("F1"),
            Some(&KeyRemap::Media("Mute".into()))
        );
        assert_eq!(
            deserialized.key_remaps.get("G"),
            Some(&KeyRemap::Macro("burst_fire".into()))
        );
        assert_eq!(deserialized.macros.len(), 1);
        assert_eq!(deserialized.macros[0].name, "Burst Fire");
        assert_eq!(deserialized.macros[0].steps.len(), 3);
    }

    #[test]
    fn test_macro_step_delay() {
        let mut key_step = MacroStep::Key {
            key: "W".into(),
            action: KeyActionType::Press,
            delay_ms: 40,
        };
        assert_eq!(key_step.delay_ms(), 40);
        key_step.set_delay_ms(120);
        assert_eq!(key_step.delay_ms(), 120);

        let mut mouse_step = MacroStep::MouseMove {
            direction: MouseDirection::Top,
            distance: 25,
            delay_ms: 30,
        };
        assert_eq!(mouse_step.delay_ms(), 30);
        mouse_step.set_delay_ms(80);
        assert_eq!(mouse_step.delay_ms(), 80);
    }
}
