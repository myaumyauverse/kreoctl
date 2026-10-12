pub mod backend;
pub mod color;
pub mod config;
pub mod device;
pub mod error;
pub mod protocol;

#[cfg(feature = "gui")]
pub mod gui;

pub use backend::{DeviceBackend, HidBackend, MockBackend};
pub use color::{PRESETS, Rgb, parse_color};
pub use error::KreoError;
pub use protocol::color_report;
