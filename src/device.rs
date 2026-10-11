use crate::color::Rgb;
use crate::error::KreoError;
use crate::protocol::color_report;
use hidapi::{HidApi, HidDevice};

pub const VENDOR_ID: u16 = 0x320f;
pub const PRODUCT_ID: u16 = 0x5055;
pub const RGB_INTERFACE: i32 = 1; // verified on the user's Kreo Hive: USB interface 1 -> hidraw2

fn open_rgb_interface() -> Result<HidDevice, KreoError> {
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

    api.open_path(info.path()).map_err(|e| {
        let err_msg = e.to_string();
        if err_msg.to_lowercase().contains("permission")
            || err_msg.to_lowercase().contains("access")
        {
            KreoError::PermissionDenied(err_msg)
        } else {
            KreoError::Hid(err_msg)
        }
    })
}

pub fn set_color(color: Rgb) -> Result<(), KreoError> {
    let device = open_rgb_interface()?;
    let report = color_report(color);
    let written = device
        .write(&report)
        .map_err(|e| KreoError::Hid(e.to_string()))?;
    if written != report.len() {
        return Err(KreoError::ShortWrite {
            sent: written,
            expected: report.len(),
        });
    }
    Ok(())
}
