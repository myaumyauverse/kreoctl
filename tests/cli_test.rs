use std::process::Command;

fn kreoctl_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kreoctl"))
}

#[test]
fn test_cli_help() {
    let output = kreoctl_bin()
        .arg("--help")
        .output()
        .expect("failed to run kreoctl --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Control Kreo Hive RGB lighting on Linux"));
    assert!(stdout.contains("Usage: kreoctl"));
    assert!(stdout.contains("--list-presets"));
}

#[test]
fn test_cli_version() {
    let output = kreoctl_bin()
        .arg("--version")
        .output()
        .expect("failed to run kreoctl --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kreoctl 0.1.0"));
}

#[test]
fn test_cli_list_presets() {
    let output = kreoctl_bin()
        .arg("--list-presets")
        .output()
        .expect("failed to run kreoctl --list-presets");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("red          #ff0000"));
    assert!(stdout.contains("cyan         #00ffff"));
    assert!(stdout.contains("off          #000000"));
}

#[test]
fn test_cli_rejects_unknown_color() {
    let output = kreoctl_bin()
        .arg("not-a-real-color-12345")
        .output()
        .expect("failed to run kreoctl");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("kreoctl: Unknown color 'not-a-real-color-12345'"));
}

#[test]
fn test_cli_rejects_invalid_hex() {
    let output = kreoctl_bin()
        .arg("#XYZ123")
        .output()
        .expect("failed to run kreoctl");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("kreoctl: Unknown color '#XYZ123'"));
}

#[test]
fn test_cli_rejects_out_of_range_brightness() {
    let output = kreoctl_bin()
        .args(["--brightness", "10"])
        .output()
        .expect("failed to run kreoctl");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not in 1..=5"));
}

#[test]
fn test_cli_rejects_invalid_mode() {
    let output = kreoctl_bin()
        .args(["--mode", "hyperspace"])
        .output()
        .expect("failed to run kreoctl");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value 'hyperspace'"));
}
