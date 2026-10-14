use crate::color::{Rgb, parse_color};
use crate::error::KreoError;
use crate::protocol::{BRIGHTNESS_MAX, LightingConfig, LightingMode};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub last_color: Option<String>,
    pub mode: Option<LightingMode>,
    pub brightness: Option<u8>,
    pub speed: Option<u8>,
}

fn config_path() -> Result<PathBuf, KreoError> {
    let dirs = ProjectDirs::from("io", "kreoctl", "kreoctl").ok_or(KreoError::ConfigDirNotFound)?;
    Ok(dirs.config_dir().join("config.toml"))
}

pub fn load_settings() -> Settings {
    let Ok(path) = config_path() else {
        return Settings::default();
    };
    if !path.exists() {
        return Settings::default();
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) -> Result<(), KreoError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(settings)
        .map_err(|e| KreoError::Config(format!("Could not serialize config: {e}")))?;
    fs::write(path, contents)?;
    Ok(())
}

/// Saves the complete lighting configuration into the user's persistent config file.
pub fn save_last_lighting(cfg: &LightingConfig) -> Result<(), KreoError> {
    let mut settings = load_settings();
    settings.last_color = Some(cfg.color.hex());
    settings.mode = Some(cfg.mode);
    settings.brightness = Some(cfg.brightness);
    settings.speed = Some(cfg.speed);
    save_settings(&settings)
}

/// Loads the complete lighting configuration from the user's persistent config file.
pub fn load_last_lighting() -> Result<LightingConfig, KreoError> {
    let settings = load_settings();
    let color_str = settings.last_color.ok_or(KreoError::NoSavedColor)?;
    let color = parse_color(&color_str)?;
    let mode = settings.mode.unwrap_or(LightingMode::Static);
    let brightness = settings
        .brightness
        .unwrap_or(BRIGHTNESS_MAX)
        .min(BRIGHTNESS_MAX);
    let speed = settings.speed.unwrap_or(3);

    Ok(LightingConfig {
        mode,
        color,
        brightness,
        speed,
        direction: 0,
        multicolor: mode.is_inherently_multicolor(),
    })
}

/// Saves the currently applied color into the user's persistent config file.
pub fn save_last_color(color: Rgb) -> Result<(), KreoError> {
    save_last_lighting(&LightingConfig::new_static(color))
}

/// Loads the last applied color from the user's persistent config file.
pub fn load_last_color() -> Result<Rgb, KreoError> {
    let cfg = load_last_lighting()?;
    Ok(cfg.color)
}

/// Enables or disables restoring lighting automatically on desktop login via XDG Autostart.
pub fn set_autostart(enabled: bool) -> Result<(), KreoError> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| KreoError::Autostart("HOME environment variable is not set".into()))?;
    let path = PathBuf::from(home).join(".config/autostart/kreoctl.desktop");

    if enabled {
        let exe = std::env::current_exe()?;
        let parent = path
            .parent()
            .ok_or_else(|| KreoError::Autostart("Invalid autostart destination path".into()))?;
        fs::create_dir_all(parent)?;
        let contents = format!(
            "[Desktop Entry]\nType=Application\nName=kreoctl\nComment=Restore Kreo Hive keyboard lighting\nExec={} --restore\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
            exe.display()
        );
        fs::write(&path, contents)?;
        println!("Enabled login restore: {}", path.display());
    } else if path.exists() {
        fs::remove_file(&path)?;
        println!("Disabled login restore.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_toml_settings() {
        let text = r##"last_color = "#ff8000"
mode = "breathing"
brightness = 5
speed = 4
"##;
        let s: Settings = toml::from_str(text).unwrap();
        assert_eq!(s.last_color.as_deref(), Some("#ff8000"));
        assert_eq!(s.mode, Some(LightingMode::Breathing));
        assert_eq!(s.brightness, Some(5));
        assert_eq!(s.speed, Some(4));
    }

    #[test]
    fn parses_empty_toml_as_default() {
        let s: Settings = toml::from_str("").unwrap();
        assert!(s.last_color.is_none());
        assert!(s.mode.is_none());
    }

    #[test]
    fn serializes_settings_to_valid_toml() {
        let s = Settings {
            last_color: Some("#00ff88".into()),
            mode: Some(LightingMode::Cycle),
            brightness: Some(3),
            speed: Some(2),
        };
        let text = toml::to_string(&s).unwrap();
        assert!(text.contains(r##"last_color = "#00ff88""##));
        assert!(text.contains(r#"mode = "spectrum""#));

        // Verify backwards compatible deserialization of legacy "cycle"
        let legacy: Settings = toml::from_str(r#"mode = "cycle""#).unwrap();
        assert_eq!(legacy.mode, Some(LightingMode::Spectrum));
    }

    #[test]
    fn settings_default_has_no_color() {
        let s = Settings::default();
        assert!(s.last_color.is_none());
        assert!(s.mode.is_none());
    }
}
