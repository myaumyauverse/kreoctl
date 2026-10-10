use crate::protocol::{Rgb, color_report};
use hidapi::{HidApi, HidDevice};

const VENDOR_ID: u16 = 0x320f;
const PRODUCT_ID: u16 = 0x5055;
const RGB_INTERFACE: i32 = 1; // verified on the user's Kreo Hive: USB interface 1 -> hidraw2

fn open_rgb_interface() -> Result<HidDevice, String> {
    let api = HidApi::new().map_err(|e| format!("Could not initialize HIDAPI: {e}"))?;
    let candidates: Vec<_> = api
        .device_list()
        .filter(|info| info.vendor_id() == VENDOR_ID && info.product_id() == PRODUCT_ID)
        .collect();

    if candidates.is_empty() {
        return Err(
            "Kreo Hive (VID:PID 320f:5055) was not found. Check the USB connection.".into(),
        );
    }

    let info = candidates.iter()
        .find(|info| info.interface_number() == RGB_INTERFACE)
        .ok_or_else(|| format!(
            "Found Kreo Hive, but could not identify its RGB interface (expected USB interface {RGB_INTERFACE}). Refusing to send a report to an unverified interface."
        ))?;

    api.open_path(info.path())
        .map_err(|e| format!("Could not open the Kreo Hive RGB interface: {e}. Check device permissions / udev rules."))
}

pub fn set_color(color: Rgb) -> Result<(), String> {
    let device = open_rgb_interface()?;
    let report = color_report(color);
    let written = device
        .write(&report)
        .map_err(|e| format!("HID report write failed: {e}"))?;
    if written != report.len() {
        return Err(format!(
            "Short HID write: sent {written} of {} bytes",
            report.len()
        ));
    }
    Ok(())
}
