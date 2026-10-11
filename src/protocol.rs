use crate::color::Rgb;

/// The exact size in bytes of the vendor-specific HID Output Report.
pub const REPORT_LEN: usize = 64;

/// The HID Output Report ID used for lighting control on interface 0003 (/dev/hidraw2).
pub const REPORT_ID: u8 = 0x04;

/// Vendor command header sequence verified on the Kreo Hive controller.
pub const CMD_HEADER: [u8; 4] = [0x3b, 0x02, 0x06, 0x22];

/// Parameter header sequence preceding the RGB payload.
pub const PARAM_HEADER: [u8; 3] = [0x06, 0x04, 0x02];

/// Lighting mode parameter (static color backlighting).
pub const MODE_STATIC: u8 = 0x08;

/// Activation / commit flag indicating the controller should apply the payload.
pub const COMMIT_FLAG: u8 = 0x01;

// Verified byte offsets within the 64-byte report payload
pub const OFFSET_REPORT_ID: usize = 0;
pub const OFFSET_CMD_START: usize = 1;
pub const OFFSET_PARAM_START: usize = 9;
pub const OFFSET_RED: usize = 14;
pub const OFFSET_GREEN: usize = 15;
pub const OFFSET_BLUE: usize = 16;
pub const OFFSET_MODE: usize = 17;
pub const OFFSET_COMMIT: usize = 28;

/// Constructs a 64-byte HID Output Report to set static backlighting on the Kreo Hive RGB keyboard.
///
/// This report layout was reverse-engineered specifically for VID:PID `320f:5055`.
/// It should not be sent to other devices without verifying their report descriptors.
pub fn color_report(color: Rgb) -> [u8; REPORT_LEN] {
    let mut report = [0_u8; REPORT_LEN];

    report[OFFSET_REPORT_ID] = REPORT_ID;
    report[OFFSET_CMD_START..OFFSET_CMD_START + CMD_HEADER.len()].copy_from_slice(&CMD_HEADER);
    report[OFFSET_PARAM_START..OFFSET_PARAM_START + PARAM_HEADER.len()]
        .copy_from_slice(&PARAM_HEADER);

    report[OFFSET_RED] = color.r;
    report[OFFSET_GREEN] = color.g;
    report[OFFSET_BLUE] = color.b;

    report[OFFSET_MODE] = MODE_STATIC;
    report[OFFSET_COMMIT] = COMMIT_FLAG;

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_matches_verified_hardware_layout() {
        let report = color_report(Rgb::new(10, 20, 30));

        // Exact report dimensions
        assert_eq!(report.len(), REPORT_LEN);

        // Report ID
        assert_eq!(report[OFFSET_REPORT_ID], REPORT_ID);

        // Headers
        assert_eq!(
            &report[OFFSET_CMD_START..OFFSET_CMD_START + CMD_HEADER.len()],
            &CMD_HEADER
        );
        assert_eq!(
            &report[OFFSET_PARAM_START..OFFSET_PARAM_START + PARAM_HEADER.len()],
            &PARAM_HEADER
        );

        // Padding regions are zeroed
        assert_eq!(&report[5..9], &[0_u8; 4]);
        assert_eq!(&report[12..14], &[0_u8; 2]);

        // RGB payload
        assert_eq!(report[OFFSET_RED], 10);
        assert_eq!(report[OFFSET_GREEN], 20);
        assert_eq!(report[OFFSET_BLUE], 30);

        // Mode parameter and commit flag
        assert_eq!(report[OFFSET_MODE], MODE_STATIC);
        assert_eq!(&report[18..28], &[0_u8; 10]);
        assert_eq!(report[OFFSET_COMMIT], COMMIT_FLAG);

        // Trailing padding must be all zeros
        assert_eq!(&report[29..REPORT_LEN], &[0_u8; 35]);
    }

    #[test]
    fn report_handles_edge_cases() {
        // Pure black / LEDs off
        let off_report = color_report(Rgb::new(0, 0, 0));
        assert_eq!(off_report[OFFSET_RED], 0);
        assert_eq!(off_report[OFFSET_GREEN], 0);
        assert_eq!(off_report[OFFSET_BLUE], 0);
        assert_eq!(off_report[OFFSET_REPORT_ID], REPORT_ID);
        assert_eq!(off_report[OFFSET_COMMIT], COMMIT_FLAG);

        // Pure white
        let white_report = color_report(Rgb::new(255, 255, 255));
        assert_eq!(white_report[OFFSET_RED], 255);
        assert_eq!(white_report[OFFSET_GREEN], 255);
        assert_eq!(white_report[OFFSET_BLUE], 255);
    }
}
