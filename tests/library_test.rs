use kreoctl::{
    MockBackend, Rgb,
    animation::stream_animation,
    device,
    error::KreoError,
    parse_color,
    protocol::{
        self, CMD_WRITE_CONFIG, LightingConfig, LightingMode, MODE_BYTE_NORMAL, REPORT_ID,
        REPORT_LEN,
    },
};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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

#[test]
fn test_library_mode_and_brightness_block_layout() {
    let cfg = LightingConfig {
        mode: LightingMode::Breathing,
        color: Rgb::new(255, 50, 25),
        brightness: 4,
        speed: 4,
        direction: 0,
        multicolor: false,
    };
    let block = protocol::encode_mode_block(&cfg);

    assert_eq!(block[protocol::PARAM_MODE], protocol::MODE_BYTE_BREATH);
    assert_eq!(block[protocol::PARAM_BRIGHTNESS], 4);
    assert_eq!(block[protocol::PARAM_SPEED], 1);
    assert_eq!(&block[5..8], &[255, 50, 25]);
    assert_eq!(protocol::decode_mode_block(&block), cfg);
}

#[test]
fn test_mock_backend_read_back_after_write() {
    let mut mock = MockBackend::new();
    let cfg = LightingConfig {
        mode: LightingMode::Cycle,
        color: Rgb::new(100, 200, 50),
        brightness: 3,
        speed: 2,
        direction: 0,
        multicolor: true,
    };
    device::set_lighting_with_backend(&mut mock, &cfg).expect("write should succeed");
    // The mock echoes requests, so a read returns zeroed data but must parse cleanly.
    assert!(device::read_current_lighting_with_backend(&mut mock).is_ok());
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

    let per_frame = protocol::TOTAL_COLOR_BYTES.div_ceil(protocol::MAX_PAYLOAD);
    assert_eq!(mock.report_count(), 1 + 15 * per_frame + 1);
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

#[test]
fn print_connected_devices() {
    if let Ok(api) = hidapi::HidApi::new() {
        for dev in api.device_list() {
            if dev.vendor_id() == 0x320f {
                println!(
                    "VID: 0x{:04x}, PID: 0x{:04x}, path: {:?}, interface: {}, usage_page: 0x{:04x}, usage: 0x{:04x}",
                    dev.vendor_id(),
                    dev.product_id(),
                    dev.path(),
                    dev.interface_number(),
                    dev.usage_page(),
                    dev.usage(),
                );
            }
        }
    }
}
