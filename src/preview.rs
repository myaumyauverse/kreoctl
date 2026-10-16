//! On-screen approximation of the firmware lighting effects.
//!
//! Shapes and timings were matched by eye against a Hive 75 at speed 3.
//! The preview cannot phase-lock to the keyboard (the firmware does not report
//! its animation clock), so it matches look and pace, not the exact frame.
//!
//! Coordinates are in key units: `x` in 0..16 across the board, `y` in
//! 0..rows down it, measured at key centres.

use crate::animation::{breathing_factor, rainbow_hue};
use crate::color::Rgb;
use crate::protocol::{LightingConfig, LightingMode, SPEED_MAX};
use std::f32::consts::TAU;

/// Board width in key units.
pub const BOARD_UNITS: f32 = 16.0;

/// A key's position on the board.
#[derive(Debug, Clone, Copy)]
pub struct KeyPos {
    pub x: f32,
    pub y: f32,
    /// Stable per-key value in 0..1, used for "random" effects.
    pub seed: f32,
}

/// A recent physical key press, for the reactive effects.
#[derive(Debug, Clone, Copy)]
pub struct Press {
    pub x: f32,
    pub y: f32,
    /// Seconds since the press.
    pub age: f32,
    /// Stable per-press value in 0..1 (picks the colour in rainbow mode).
    pub seed: f32,
}

/// How long a press keeps influencing the preview.
pub const PRESS_LIFETIME: f32 = 1.2;

/// Firmware time per Spectrum cycle, measured: ~15 s at speed 1, ~6 s at speed 5,
/// linear in between. Every effect slows down by the same ratio.
fn spectrum_period(speed: u8) -> f32 {
    6.0 + 2.25 * (SPEED_MAX - speed.clamp(1, SPEED_MAX)) as f32
}

/// Converts wall-clock seconds into "speed 3 seconds", so effect timings below
/// can be written once for speed 3.
pub fn effect_time(elapsed: f32, speed: u8) -> f32 {
    elapsed * spectrum_period(3) / spectrum_period(speed)
}

fn scale(c: Rgb, f: f32) -> Rgb {
    let f = f.clamp(0.0, 1.0);
    Rgb::new(
        (c.r as f32 * f).round() as u8,
        (c.g as f32 * f).round() as u8,
        (c.b as f32 * f).round() as u8,
    )
}

/// Deterministic pseudo-random value in 0..1 from two inputs.
fn hash2(a: f32, b: f32) -> f32 {
    let v = (a * 127.1 + b * 311.7).sin() * 43_758.547;
    v - v.floor()
}

/// 0 → 1 → 0 over one period.
fn triangle(phase: f32) -> f32 {
    let p = phase.rem_euclid(1.0);
    1.0 - (2.0 * p - 1.0).abs()
}

/// Colour for a moving pattern: rainbow, or the base colour's brightness rippling.
fn pattern(cfg: &LightingConfig, phase: f32) -> Rgb {
    if cfg.multicolor {
        rainbow_hue(phase.rem_euclid(1.0))
    } else {
        let f = (f32::sin(phase * TAU) + 1.0) / 2.0;
        scale(cfg.color, 0.15 + 0.85 * f)
    }
}

/// Colour of a lit key: the base colour, or a random hue in rainbow mode.
fn lit(cfg: &LightingConfig, seed: f32) -> Rgb {
    if cfg.multicolor {
        rainbow_hue(seed)
    } else {
        cfg.color
    }
}

/// The colour a key shows for `cfg` at `t` (from [`effect_time`]), before brightness.
/// `per_key` is the key's own colour for Custom mode.
pub fn key_color(
    cfg: &LightingConfig,
    key: KeyPos,
    rows: f32,
    t: f32,
    presses: &[Press],
    per_key: Rgb,
) -> Rgb {
    // +1 = right-to-left / upwards (direction 0), -1 = the other way.
    let flow = if cfg.direction == 0 { 1.0 } else { -1.0 };
    let (cx, cy) = (BOARD_UNITS / 2.0, rows / 2.0);
    // Colour bands travel across the 16-unit board in ~1.5 s.
    let band_speed = BOARD_UNITS / 1.5;

    match cfg.mode {
        LightingMode::Normal | LightingMode::Surmount => cfg.color,
        LightingMode::Custom => per_key,
        LightingMode::Off => Rgb::new(0, 0, 0),

        LightingMode::Spectrum => rainbow_hue((t / spectrum_period(3)).rem_euclid(1.0)),

        LightingMode::Breath => {
            let period = 2.5;
            let base = if cfg.multicolor {
                rainbow_hue((t / period).floor() * 0.17)
            } else {
                cfg.color
            };
            scale(base, breathing_factor(t / period))
        }

        // Narrow rainbow stripes sliding sideways (about two rainbows per board).
        LightingMode::Corrugated => pattern(cfg, (key.x + flow * band_speed * t) / 8.0),

        // One wide rainbow sliding sideways.
        LightingMode::Cloud => pattern(cfg, (key.x + flow * band_speed * t) / BOARD_UNITS),

        // Rainbow snaking along the rows, alternating direction each row.
        LightingMode::Serpentine => {
            let along = if (key.y.floor() as i32) % 2 == 0 {
                key.x
            } else {
                BOARD_UNITS - key.x
            };
            let path = key.y.floor() * BOARD_UNITS + along;
            pattern(cfg, (path + flow * band_speed * t) / 24.0)
        }

        // Rainbow rolling vertically, one rainbow over the board height.
        LightingMode::Roll => pattern(cfg, (key.y + flow * (rows / 1.5) * t) / rows),

        // Rainbow rings around the centre. With "left to right" (direction 1)
        // the keyboard spreads them outwards, so direction 0 moves them inwards.
        LightingMode::Speed => {
            let r = ((key.x - cx).powi(2) + ((key.y - cy) * 1.4).powi(2)).sqrt();
            pattern(cfg, (r + flow * band_speed * t) / 8.0)
        }

        // One lit key per column bouncing top -> bottom -> top; columns lag behind the middle.
        LightingMode::Wave => {
            let lag = (key.x - cx).abs() / cx * 0.6;
            let pos = 0.5 + triangle((t - lag) / 1.5) * (rows - 1.0);
            let f = 1.0 - (key.y - pos).abs();
            let base = if cfg.multicolor {
                rainbow_hue(key.x / BOARD_UNITS)
            } else {
                cfg.color
            };
            scale(base, f)
        }

        // A bar bouncing left <-> right, ~0.8 s per sweep.
        LightingMode::Scan => {
            let pos = triangle(t / 1.6) * BOARD_UNITS;
            scale(lit(cfg, pos / BOARD_UNITS), 1.0 - (key.x - pos).abs() / 1.5)
        }

        // Random single keys twinkle briefly on a dark board.
        LightingMode::Stars => {
            let period = 1.6;
            let local = t / period + key.seed;
            let cycle = local.floor();
            if hash2(key.seed, cycle) > 0.3 {
                return Rgb::new(0, 0, 0);
            }
            let twinkle = (local.fract() * period / 0.4).min(1.0);
            scale(
                lit(cfg, hash2(cycle, key.seed)),
                (twinkle * TAU / 2.0).sin(),
            )
        }

        // Random keys bloom slowly, each in its own colour.
        LightingMode::Flowers => {
            let period = 3.0;
            let local = t / period + key.seed;
            let cycle = local.floor();
            if hash2(key.seed, cycle) > 0.5 {
                return Rgb::new(0, 0, 0);
            }
            let bloom = (local.fract() * TAU / 2.0).sin();
            scale(rainbow_hue(hash2(cycle, key.seed + 0.5)), bloom)
        }

        // Drops falling down the columns with a short trail.
        LightingMode::Rain => {
            let column = key.x.floor();
            let period = 1.2;
            let local = t / period + hash2(column, 7.0);
            let head = local.fract() * (rows + 2.0) - 1.0;
            let behind = head - key.y;
            if !(0.0..2.0).contains(&behind) {
                return Rgb::new(0, 0, 0);
            }
            scale(lit(cfg, hash2(column, local.floor())), 1.0 - behind / 2.0)
        }

        // Light runs from opposite corners to the centre, spins, and runs back.
        LightingMode::Cartoon => {
            let p = (t / 4.0).rem_euclid(1.0);
            let corner = (0.0_f32, 0.0_f32);
            let (ax, ay) = if p < 0.35 {
                let k = p / 0.35;
                (
                    corner.0 + (cx - corner.0) * k,
                    corner.1 + (cy - corner.1) * k,
                )
            } else if p < 0.65 {
                let angle = (p - 0.35) / 0.3 * TAU * 1.5;
                (cx + 3.0 * angle.cos(), cy + 1.5 * angle.sin())
            } else {
                let k = (1.0 - p) / 0.35;
                (
                    corner.0 + (cx - corner.0) * k,
                    corner.1 + (cy - corner.1) * k,
                )
            };
            // The second light mirrors the first through the centre.
            let (bx, by) = (2.0 * cx - ax, 2.0 * cy - ay);
            let d = ((key.x - ax).powi(2) + (key.y - ay).powi(2))
                .sqrt()
                .min(((key.x - bx).powi(2) + (key.y - by).powi(2)).sqrt());
            scale(lit(cfg, p), 1.0 - d / 2.5)
        }

        // Pressed key lights up, then fades.
        LightingMode::Reaction => react(cfg, presses, |p| {
            let same_key = (key.x - p.x).abs() < 0.5 && (key.y - p.y).abs() < 0.5;
            if same_key { 1.0 - p.age / 0.8 } else { 0.0 }
        }),

        // A ring expands from the pressed key.
        LightingMode::Ripples => react(cfg, presses, |p| {
            let d = ((key.x - p.x).powi(2) + (key.y - p.y).powi(2)).sqrt();
            let ring = 1.0 - (d - p.age * 12.0).abs() / 1.0;
            ring * (1.0 - p.age / PRESS_LIFETIME)
        }),

        // Light shoots left and right along the pressed key's row.
        LightingMode::Traverse => react(cfg, presses, |p| {
            if (key.y - p.y).abs() >= 0.5 {
                return 0.0;
            }
            let streak = 1.0 - ((key.x - p.x).abs() - p.age * 14.0).abs() / 1.2;
            streak * (1.0 - p.age / PRESS_LIFETIME)
        }),
    }
}

/// Combines the strongest press contribution for a key.
fn react(cfg: &LightingConfig, presses: &[Press], strength: impl Fn(&Press) -> f32) -> Rgb {
    presses
        .iter()
        .map(|p| (strength(p).clamp(0.0, 1.0), p.seed))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .filter(|(s, _)| *s > 0.0)
        .map(|(s, seed)| scale(lit(cfg, seed), s))
        .unwrap_or(Rgb::new(0, 0, 0))
}

/// Whether the effect animates on its own (needs continuous repaints).
pub fn is_animated(mode: LightingMode) -> bool {
    !matches!(
        mode,
        LightingMode::Normal | LightingMode::Custom | LightingMode::Off | LightingMode::Surmount
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(mode: LightingMode, speed: u8) -> LightingConfig {
        LightingConfig {
            mode,
            speed,
            multicolor: true,
            ..Default::default()
        }
    }

    const KEY: KeyPos = KeyPos {
        x: 4.5,
        y: 2.5,
        seed: 0.3,
    };

    #[test]
    fn speed_curve_matches_measured_spectrum() {
        // One speed-3 Spectrum cycle (10.5 s of effect time) takes ~15 s at
        // speed 1 and ~6 s at speed 5 of wall-clock time.
        assert!((effect_time(15.0, 1) - 10.5).abs() < 1e-3);
        assert!((effect_time(6.0, 5) - 10.5).abs() < 1e-3);
        assert!((effect_time(10.5, 3) - 10.5).abs() < 1e-3);
    }

    #[test]
    fn spectrum_is_uniform_across_keys() {
        let c = cfg(LightingMode::Spectrum, 3);
        let other = KeyPos {
            x: 14.0,
            y: 5.0,
            seed: 0.9,
        };
        assert_eq!(
            key_color(&c, KEY, 6.0, 3.0, &[], Rgb::new(0, 0, 0)),
            key_color(&c, other, 6.0, 3.0, &[], Rgb::new(0, 0, 0))
        );
    }

    #[test]
    fn reactive_effects_are_dark_without_presses() {
        for mode in [
            LightingMode::Reaction,
            LightingMode::Ripples,
            LightingMode::Traverse,
        ] {
            assert_eq!(
                key_color(&cfg(mode, 3), KEY, 6.0, 1.0, &[], Rgb::new(0, 0, 0)),
                Rgb::new(0, 0, 0)
            );
        }
    }

    #[test]
    fn reaction_lights_pressed_key_then_fades() {
        let mut c = cfg(LightingMode::Reaction, 3);
        c.multicolor = false;
        c.color = Rgb::new(255, 0, 0);
        let press = |age| Press {
            x: KEY.x,
            y: KEY.y,
            age,
            seed: 0.0,
        };
        let fresh = key_color(&c, KEY, 6.0, 0.0, &[press(0.0)], Rgb::new(0, 0, 0));
        let old = key_color(&c, KEY, 6.0, 0.0, &[press(0.6)], Rgb::new(0, 0, 0));
        assert_eq!(fresh, Rgb::new(255, 0, 0));
        assert!(old.r < fresh.r && old.r > 0);
    }

    #[test]
    fn traverse_stays_on_the_pressed_row() {
        let c = cfg(LightingMode::Traverse, 3);
        let press = [Press {
            x: 8.0,
            y: 2.5,
            age: 0.25,
            seed: 0.0,
        }];
        let other_row = KeyPos {
            x: 4.5,
            y: 4.5,
            seed: 0.1,
        };
        assert_eq!(
            key_color(&c, other_row, 6.0, 0.0, &press, Rgb::new(0, 0, 0)),
            Rgb::new(0, 0, 0)
        );
    }

    #[test]
    fn speed_rings_move_outwards_left_to_right() {
        let mut c = cfg(LightingMode::Speed, 3);
        c.direction = 1;
        // The colour at radius 2 now is found further out a moment later.
        let at = |x: f32, t: f32| {
            key_color(
                &c,
                KeyPos {
                    x,
                    y: 3.0,
                    seed: 0.0,
                },
                6.0,
                t,
                &[],
                Rgb::new(0, 0, 0),
            )
        };
        let later = 0.1;
        assert_eq!(
            at(8.0 + 2.0, 0.0),
            at(8.0 + 2.0 + BOARD_UNITS / 1.5 * later, later)
        );
    }

    #[test]
    fn wave_lights_one_key_per_column() {
        let mut c = cfg(LightingMode::Wave, 3);
        c.multicolor = false;
        c.color = Rgb::new(0, 0, 255);
        for step in 0..30 {
            let t = step as f32 * 0.05;
            let lit = (0..6)
                .filter(|row| {
                    let k = KeyPos {
                        x: 8.0,
                        y: *row as f32 + 0.5,
                        seed: 0.0,
                    };
                    key_color(&c, k, 6.0, t, &[], Rgb::new(0, 0, 0)).b > 128
                })
                .count();
            assert!(lit <= 1, "{lit} keys brightly lit at t={t}");
        }
    }

    #[test]
    fn wave_starts_at_the_top() {
        let mut c = cfg(LightingMode::Wave, 3);
        c.multicolor = false;
        c.color = Rgb::new(0, 0, 255);
        let middle = |y| KeyPos {
            x: 8.0,
            y,
            seed: 0.0,
        };
        // At the start of a cycle the middle column's light is on the top row.
        let t = 0.0;
        let top = key_color(&c, middle(0.5), 6.0, t, &[], Rgb::new(0, 0, 0));
        let bottom = key_color(&c, middle(5.5), 6.0, t, &[], Rgb::new(0, 0, 0));
        assert!(top.b > bottom.b);
    }
}
