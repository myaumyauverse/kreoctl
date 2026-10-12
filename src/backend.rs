use crate::error::KreoError;
use crate::protocol::REPORT_LEN;

/// Abstraction for communicating with the physical or emulated keyboard.
///
/// This trait allows testing packet generation, lighting sequence timing,
/// and error recovery without requiring physical hardware plugged in.
pub trait DeviceBackend {
    /// Send a single 64-byte Output Report to the device.
    fn write_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), KreoError>;
}

/// Real USB HID transport backed by the Linux `hidraw` subsystem via `hidapi`.
pub struct HidBackend {
    device: hidapi::HidDevice,
}

impl HidBackend {
    pub fn new(device: hidapi::HidDevice) -> Self {
        Self { device }
    }
}

impl DeviceBackend for HidBackend {
    fn write_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), KreoError> {
        let written = self
            .device
            .write(report)
            .map_err(|e| KreoError::Hid(e.to_string()))?;
        if written != report.len() {
            return Err(KreoError::ShortWrite {
                sent: written,
                expected: report.len(),
            });
        }
        Ok(())
    }
}

/// In-memory mock backend for unit and integration testing without hardware.
#[derive(Debug, Clone, Default)]
pub struct MockBackend {
    pub written_reports: Vec<[u8; REPORT_LEN]>,
    pub simulate_failure: bool,
}

impl MockBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn last_report(&self) -> Option<&[u8; REPORT_LEN]> {
        self.written_reports.last()
    }

    pub fn report_count(&self) -> usize {
        self.written_reports.len()
    }

    pub fn clear(&mut self) {
        self.written_reports.clear();
    }
}

impl DeviceBackend for MockBackend {
    fn write_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), KreoError> {
        if self.simulate_failure {
            return Err(KreoError::ShortWrite {
                sent: 0,
                expected: REPORT_LEN,
            });
        }
        self.written_reports.push(*report);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Rgb;
    use crate::protocol::color_report;

    #[test]
    fn mock_backend_records_written_reports() {
        let mut mock = MockBackend::new();
        let report = color_report(Rgb::new(255, 0, 128));

        mock.write_report(&report).unwrap();
        assert_eq!(mock.report_count(), 1);
        assert_eq!(mock.last_report().unwrap(), &report);
    }

    #[test]
    fn mock_backend_simulates_write_failures() {
        let mut mock = MockBackend::new();
        mock.simulate_failure = true;
        let report = color_report(Rgb::new(0, 255, 0));

        let result = mock.write_report(&report);
        assert!(matches!(result, Err(KreoError::ShortWrite { .. })));
    }
}
