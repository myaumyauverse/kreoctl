use std::io;
use thiserror::Error;

/// Central error type for all hardware, configuration, and protocol operations in `kreoctl`.
#[derive(Debug, Error)]
pub enum KreoError {
    #[error("Kreo Hive (VID:PID 320f:5055) was not found. Check the USB connection.")]
    DeviceNotFound,

    #[error(
        "Could not open the Kreo Hive RGB interface: {0}. Check device permissions / udev rules."
    )]
    PermissionDenied(String),

    #[error(
        "Found Kreo Hive, but could not identify its RGB interface (expected USB interface {interface}). Refusing to send a report to an unverified interface."
    )]
    InterfaceNotFound { interface: i32 },

    #[error("HID communication failed: {0}")]
    Hid(String),

    #[error("Short HID write: sent {sent} of {expected} bytes")]
    ShortWrite { sent: usize, expected: usize },

    #[error("Unknown color '{input}'. Use a preset or #RRGGBB.")]
    InvalidColor { input: String },

    #[error("No saved color yet. Set a color first.")]
    NoSavedColor,

    #[error("Could not determine the user config directory")]
    ConfigDirNotFound,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Autostart configuration error: {0}")]
    Autostart(String),

    #[error(
        "kreoctl was compiled without GUI support. Rebuild with `--features gui` to enable the graphical interface."
    )]
    GuiNotCompiled,

    #[error("GUI error: {0}")]
    Gui(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_formatting() {
        let err = KreoError::InvalidColor {
            input: "bluish".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Unknown color 'bluish'. Use a preset or #RRGGBB."
        );

        let err2 = KreoError::ShortWrite {
            sent: 32,
            expected: 64,
        };
        assert_eq!(err2.to_string(), "Short HID write: sent 32 of 64 bytes");
    }
}
