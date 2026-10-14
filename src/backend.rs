use crate::error::KreoError;
use crate::protocol::REPORT_LEN;

/// Abstraction for communicating with the physical or emulated keyboard.
///
/// This trait allows testing packet generation, lighting sequence timing,
/// and error recovery without requiring physical hardware plugged in.
pub trait DeviceBackend {
    /// Send a single 64-byte Output Report to the device.
    fn write_report(&mut self, report: &[u8; REPORT_LEN]) -> Result<(), KreoError>;

    /// Read a single 64-byte Input Report from the device with a timeout.
    fn read_report(&mut self) -> Result<[u8; REPORT_LEN], KreoError>;

    /// Send a single 64-byte Output Report to the device and synchronously read the 64-byte ACK reply.
    fn write_and_read_ack(
        &mut self,
        report: &[u8; REPORT_LEN],
    ) -> Result<[u8; REPORT_LEN], KreoError> {
        self.write_report(report)?;
        self.read_report()
    }
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

    fn read_report(&mut self) -> Result<[u8; REPORT_LEN], KreoError> {
        let mut buf = [0_u8; REPORT_LEN];
        let bytes_read = self
            .device
            .read_timeout(&mut buf, 200)
            .map_err(|e| KreoError::Hid(e.to_string()))?;
        if bytes_read == 0 {
            return Err(KreoError::Hid("Read timeout waiting for response".into()));
        }
        Ok(buf)
    }
}

/// In-memory mock backend for unit and integration testing without hardware.
#[derive(Debug, Clone, Default)]
pub struct MockBackend {
    pub written_reports: Vec<[u8; REPORT_LEN]>,
    pub read_queue: Vec<[u8; REPORT_LEN]>,
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

    pub fn queue_read_report(&mut self, report: [u8; REPORT_LEN]) {
        self.read_queue.push(report);
    }

    pub fn clear(&mut self) {
        self.written_reports.clear();
        self.read_queue.clear();
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

    fn read_report(&mut self) -> Result<[u8; REPORT_LEN], KreoError> {
        if self.simulate_failure {
            return Err(KreoError::Hid("Simulated read failure".into()));
        }
        if !self.read_queue.is_empty() {
            Ok(self.read_queue.remove(0))
        } else if let Some(last) = self.written_reports.last() {
            Ok(*last)
        } else {
            Ok([0_u8; REPORT_LEN])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{CMD_WRITE_CONFIG, build_write};

    fn color_report(r: u8, g: u8, b: u8) -> [u8; REPORT_LEN] {
        build_write(CMD_WRITE_CONFIG, 0x06, &[r, g, b])
    }

    #[test]
    fn mock_backend_records_written_reports() {
        let mut mock = MockBackend::new();
        let report = color_report(255, 0, 128);

        mock.write_report(&report).unwrap();
        assert_eq!(mock.report_count(), 1);
        assert_eq!(mock.last_report().unwrap(), &report);
    }

    #[test]
    fn mock_backend_simulates_write_failures() {
        let mut mock = MockBackend::new();
        mock.simulate_failure = true;
        let report = color_report(0, 255, 0);

        let result = mock.write_report(&report);
        assert!(matches!(result, Err(KreoError::ShortWrite { .. })));
    }

    #[test]
    fn mock_backend_reads_queued_or_echo_reports() {
        let mut mock = MockBackend::new();
        let report = color_report(10, 20, 30);
        mock.write_report(&report).unwrap();

        let read = mock.read_report().unwrap();
        assert_eq!(read, report);
    }
}
