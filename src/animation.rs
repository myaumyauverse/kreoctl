use crate::backend::DeviceBackend;
use crate::color::Rgb;
use crate::error::KreoError;
use crate::protocol::{LightingConfig, LightingMode, build_report};
use std::f32::consts::PI;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

/// Converts a hue value [0.0, 1.0] to an RGB color using full saturation and brightness.
pub fn rainbow_hue(progress: f32) -> Rgb {
    let hue = (progress.fract() * 6.0).clamp(0.0, 6.0);
    let x = (1.0 - (hue % 2.0 - 1.0).abs()).clamp(0.0, 1.0);

    let (r, g, b) = match hue as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };

    Rgb::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Computes a breathing intensity factor [0.05, 1.0] using a smooth sine wave.
pub fn breathing_factor(progress: f32) -> f32 {
    let factor = (f32::sin(progress * 2.0 * PI - (PI / 2.0)) + 1.0) / 2.0;
    // Keep a 5% floor so LEDs do not abruptly switch off completely
    0.05 + 0.95 * factor
}

/// Applies breathing intensity to a base color.
pub fn breathing_color(base: Rgb, progress: f32) -> Rgb {
    let factor = breathing_factor(progress);
    Rgb::new(
        (base.r as f32 * factor).round() as u8,
        (base.g as f32 * factor).round() as u8,
        (base.b as f32 * factor).round() as u8,
    )
}

/// Computes the frame delay Duration based on speed level 1 (slow) to 5 (fast).
pub fn speed_to_delay(speed: u8) -> Duration {
    let clamped = speed.clamp(1, 5);
    match clamped {
        1 => Duration::from_millis(50),
        2 => Duration::from_millis(35),
        3 => Duration::from_millis(25),
        4 => Duration::from_millis(16),
        5 => Duration::from_millis(10),
        _ => Duration::from_millis(25),
    }
}

/// Streams software animation frames to the provided DeviceBackend until stopped.
pub fn stream_animation<B: DeviceBackend>(
    backend: &mut B,
    mode: LightingMode,
    base_color: Rgb,
    speed: u8,
    stop_signal: Arc<AtomicBool>,
    max_frames: Option<usize>,
) -> Result<(), KreoError> {
    let delay = speed_to_delay(speed);
    let mut step: usize = 0;
    let total_cycle_steps = 100;

    while !stop_signal.load(Ordering::Relaxed) {
        if max_frames.is_some_and(|limit| step >= limit) {
            break;
        }

        let progress = (step % total_cycle_steps) as f32 / total_cycle_steps as f32;
        let color = match mode {
            LightingMode::Cycle | LightingMode::Wave => rainbow_hue(progress),
            LightingMode::Breathing => breathing_color(base_color, progress),
            LightingMode::Static => base_color,
            LightingMode::Off => Rgb::new(0, 0, 0),
            LightingMode::Reactive => base_color,
        };

        let cfg = LightingConfig {
            mode: LightingMode::Static,
            color,
            brightness: 4,
            speed,
        };
        let report = build_report(&cfg);
        backend.write_report(&report)?;

        step += 1;
        sleep(delay);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockBackend;

    #[test]
    fn rainbow_hue_produces_primary_colors() {
        let red = rainbow_hue(0.0);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);

        let green = rainbow_hue(1.0 / 3.0);
        assert_eq!(green.g, 255);

        let blue = rainbow_hue(2.0 / 3.0);
        assert_eq!(blue.b, 255);
    }

    #[test]
    fn breathing_modulates_color() {
        let base = Rgb::new(200, 100, 50);
        let dim = breathing_color(base, 0.0);
        let bright = breathing_color(base, 0.5);

        assert!(dim.r < bright.r);
        assert!(dim.g < bright.g);
        assert!(dim.b < bright.b);
    }

    #[test]
    fn stream_animation_respects_frame_limit_on_mock() {
        let mut mock = MockBackend::new();
        let stop = Arc::new(AtomicBool::new(false));

        stream_animation(
            &mut mock,
            LightingMode::Cycle,
            Rgb::new(255, 0, 0),
            5,
            stop,
            Some(10),
        )
        .unwrap();

        assert_eq!(mock.report_count(), 10);
    }
}
