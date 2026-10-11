use crate::color::{Rgb, parse_color};
use crate::error::KreoError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
struct Settings {
    last_color: Option<String>,
}

fn config_path() -> Result<PathBuf, KreoError> {
    let dirs = ProjectDirs::from("io", "kreoctl", "kreoctl").ok_or(KreoError::ConfigDirNotFound)?;
    Ok(dirs.config_dir().join("config.toml"))
}

fn load_settings() -> Settings {
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

fn save_settings(settings: &Settings) -> Result<(), KreoError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(settings)
        .map_err(|e| KreoError::Config(format!("Could not serialize config: {e}")))?;
    fs::write(path, contents)?;
    Ok(())
}

/// Saves the currently applied color into the user's persistent config file.
pub fn save_last_color(color: Rgb) -> Result<(), KreoError> {
    let mut settings = load_settings();
    settings.last_color = Some(color.hex());
    save_settings(&settings)
}

/// Loads the last applied color from the user's persistent config file.
pub fn load_last_color() -> Result<Rgb, KreoError> {
    let settings = load_settings();
    let value = settings.last_color.ok_or(KreoError::NoSavedColor)?;
    parse_color(&value)
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
    } else {
        println!("Autostart was not enabled; nothing to remove.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_has_no_color() {
        let settings = Settings::default();
        assert_eq!(settings.last_color, None);
    }

    #[test]
    fn serializes_settings_to_valid_toml() {
        let settings = Settings {
            last_color: Some("#00ff88".to_string()),
        };
        let toml_str = toml::to_string(&settings).expect("serialization failed");
        assert!(toml_str.contains("last_color = \"#00ff88\""));
    }

    #[test]
    fn parses_valid_toml_settings() {
        let raw = "last_color = \"#ff0000\"\n";
        let parsed: Settings = toml::from_str(raw).expect("deserialization failed");
        assert_eq!(parsed.last_color, Some("#ff0000".to_string()));
    }

    #[test]
    fn parses_empty_toml_as_default() {
        let raw = "";
        let parsed: Settings = toml::from_str(raw).expect("empty toml should parse");
        assert_eq!(parsed.last_color, None);
    }
}
