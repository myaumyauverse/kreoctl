use kreoctl::{
    DeviceBackend, MockBackend, Rgb,
    animation::stream_animation,
    color_report,
    error::KreoError,
    parse_color,
    protocol::{
        LightingConfig, LightingMode, MODE_BYTE_BREATHING, MODE_BYTE_CYCLE, REPORT_ID, REPORT_LEN,
        build_report,
    },
};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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

#[test]
fn test_library_mode_and_brightness_report_layout() {
    let cfg = LightingConfig {
        mode: LightingMode::Breathing,
        color: Rgb::new(255, 50, 25),
        brightness: 5,
        speed: 4,
    };
    let report = build_report(&cfg);

    assert_eq!(report[0], REPORT_ID);
    assert_eq!(report[4], MODE_BYTE_BREATHING);
    assert_eq!(report[5], 4); // speed
    assert_eq!(&report[14..17], &[255, 50, 25]);
    assert_eq!(report[17], 10); // brightness level 5 * 2
}

#[test]
fn test_mock_backend_transaction_recording() {
    let mut mock = MockBackend::new();
    let cfg = LightingConfig {
        mode: LightingMode::Cycle,
        color: Rgb::new(100, 200, 50),
        brightness: 3,
        speed: 2,
    };
    let report = build_report(&cfg);

    mock.write_report(&report).expect("write should succeed");
    assert_eq!(mock.report_count(), 1);
    let captured = mock.last_report().unwrap();
    assert_eq!(captured[4], MODE_BYTE_CYCLE);
    assert_eq!(captured[5], 2);
}

#[test]
fn test_streaming_animation_into_mock_backend() {
    let mut mock = MockBackend::new();
    let stop_signal = Arc::new(AtomicBool::new(false));

    stream_animation(
        &mut mock,
        LightingMode::Breathing,
        Rgb::new(0, 0, 255),
        5,
        stop_signal,
        Some(15),
    )
    .expect("streaming should succeed");

    assert_eq!(mock.report_count(), 15);
}

#[test]
fn test_config_serialization_roundtrip() {
    let s = kreoctl::config::Settings {
        last_color: Some("#2a54a8".into()),
        mode: Some(LightingMode::Wave),
        brightness: Some(4),
        speed: Some(5),
    };

    let serialized = toml::to_string(&s).expect("serializing settings should succeed");
    let deserialized: kreoctl::config::Settings =
        toml::from_str(&serialized).expect("deserializing settings should succeed");

    assert_eq!(deserialized.last_color.as_deref(), Some("#2a54a8"));
    assert_eq!(deserialized.mode, Some(LightingMode::Wave));
    assert_eq!(deserialized.brightness, Some(4));
    assert_eq!(deserialized.speed, Some(5));
}
