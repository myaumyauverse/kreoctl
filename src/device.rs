use crate::backend::{DeviceBackend, HidBackend};
use crate::color::Rgb;
use crate::error::KreoError;
use crate::protocol::{LightingConfig, build_report, color_report};
use hidapi::HidApi;

pub const VENDOR_ID: u16 = 0x320f;
pub const PRODUCT_ID: u16 = 0x5055;
pub const RGB_INTERFACE: i32 = 1; // verified on Kreo Hive: USB interface 1 -> hidraw2

/// Discover and open the keyboard's vendor lighting interface.
pub fn open_device() -> Result<HidBackend, KreoError> {
    let api = HidApi::new().map_err(|e| KreoError::Hid(e.to_string()))?;
    let candidates: Vec<_> = api
        .device_list()
        .filter(|info| info.vendor_id() == VENDOR_ID && info.product_id() == PRODUCT_ID)
        .collect();

    if candidates.is_empty() {
        return Err(KreoError::DeviceNotFound);
    }

    let info = candidates
        .iter()
        .find(|info| info.interface_number() == RGB_INTERFACE)
        .ok_or(KreoError::InterfaceNotFound {
            interface: RGB_INTERFACE,
        })?;

    let device = api.open_path(info.path()).map_err(|e| {
        let err_msg = e.to_string();
        if err_msg.to_lowercase().contains("permission")
            || err_msg.to_lowercase().contains("access")
        {
            KreoError::PermissionDenied(err_msg)
        } else {
            KreoError::Hid(err_msg)
        }
    })?;

    Ok(HidBackend::new(device))
}

/// Apply a static color to the physical keyboard using the auto-discovered HID backend.
pub fn set_color(color: Rgb) -> Result<(), KreoError> {
    let mut backend = open_device()?;
    set_color_with_backend(&mut backend, color)
}

/// Apply a static color using an explicit backend (useful for testing and dependency injection).
pub fn set_color_with_backend(
    backend: &mut dyn DeviceBackend,
    color: Rgb,
) -> Result<(), KreoError> {
    let report = color_report(color);
    backend.write_report(&report)
}

/// Apply an arbitrary LightingConfig (mode, speed, brightness, color) to the keyboard.
pub fn set_lighting(config: &LightingConfig) -> Result<(), KreoError> {
    let mut backend = open_device()?;
    set_lighting_with_backend(&mut backend, config)
}

/// Apply a LightingConfig using an explicit backend.
pub fn set_lighting_with_backend(
    backend: &mut dyn DeviceBackend,
    config: &LightingConfig,
) -> Result<(), KreoError> {
    let report = build_report(config);
    backend.write_report(&report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;
    use crate::protocol::LightingMode;

    #[test]
    fn set_color_with_mock_backend_succeeds() {
        let mut mock = MockBackend::new();
        let color = Rgb::new(0, 128, 255);

        assert!(set_color_with_backend(&mut mock, color).is_ok());
        assert_eq!(mock.report_count(), 1);
        let report = mock.last_report().unwrap();
        assert_eq!(report[14], 0);
        assert_eq!(report[15], 128);
        assert_eq!(report[16], 255);
    }

    #[test]
    fn set_lighting_with_mock_backend_succeeds() {
        let mut mock = MockBackend::new();
        let cfg = LightingConfig {
            mode: LightingMode::Breathing,
            color: Rgb::new(255, 100, 0),
            brightness: 5,
            speed: 4,
        };

        assert!(set_lighting_with_backend(&mut mock, &cfg).is_ok());
        assert_eq!(mock.report_count(), 1);
        let report = mock.last_report().unwrap();
        assert_eq!(report[4], crate::protocol::MODE_BYTE_BREATHING);
        assert_eq!(report[5], 4);
    }
}
