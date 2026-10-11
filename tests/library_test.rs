use kreoctl::{
    Rgb, color_report,
    error::KreoError,
    parse_color,
    protocol::{REPORT_ID, REPORT_LEN},
};

#[test]
fn test_library_color_parsing_and_report_generation() {
    let rgb = parse_color("#00ff88").expect("valid hex color should parse");
    assert_eq!(rgb, Rgb::new(0, 255, 136));

    let report = color_report(rgb);
    assert_eq!(report.len(), REPORT_LEN);
    assert_eq!(report[0], REPORT_ID);
    assert_eq!(&report[14..17], &[0, 255, 136]);
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
