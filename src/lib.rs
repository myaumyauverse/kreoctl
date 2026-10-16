pub mod animation;
pub mod backend;
pub mod color;
pub mod config;
pub mod device;
pub mod error;
pub mod keyboard;
pub mod preview;
pub mod profile;
pub mod protocol;
pub mod sync;

#[cfg(feature = "gui")]
pub mod gui;

pub use backend::{DeviceBackend, HidBackend, MockBackend};
pub use color::{PRESETS, Rgb, parse_color};
pub use error::KreoError;
pub use keyboard::{KeyDefinition, KeyboardModel, KeyboardVariant};
pub use profile::{Profile, ProfileStore};
pub use protocol::{LightingConfig, LightingMode};
