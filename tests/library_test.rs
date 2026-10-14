use kreoctl::{
    MockBackend, Rgb, device,
    error::KreoError,
    parse_color,
    protocol::{CMD_WRITE_CONFIG, MODE_BYTE_NORMAL, REPORT_ID, REPORT_LEN},
};

#[test]
fn test_library_color_parsing_and_report_generation() {
    let rgb = parse_color("#00ff88").expect("valid hex color should parse");
    assert_eq!(rgb, Rgb::new(0, 255, 136));

    let mut mock = MockBackend::new();
    device::set_color_with_backend(&mut mock, rgb).expect("write should succeed");
    let write = mock
        .written_reports
        .iter()
        .find(|r| r[3] == CMD_WRITE_CONFIG)
        .expect("a config write");
    assert_eq!(write.len(), REPORT_LEN);
    assert_eq!(write[0], REPORT_ID);
    assert_eq!(write[8], MODE_BYTE_NORMAL);
    assert_eq!(&write[13..16], &[0, 255, 136]);
}

#[test]
fn test_library_error_matching() {
    let err = parse_color("invalid-color-str").unwrap_err();
    match err {
        KreoError::InvalidColor { input } => {
            assert_eq!(input, "invalid-color-str");
        }
        other => panic!("expected KreoError::InvalidColor, got {other:?}"),
    }
}
