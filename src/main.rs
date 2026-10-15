use clap::Parser;
use kreoctl::{
    animation,
    color::{PRESETS, Rgb, parse_color},
    config, device,
    error::KreoError,
    protocol::LightingMode,
};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Parser)]
#[command(
    name = "kreoctl",
    version,
    about = "Control Kreo Hive RGB lighting on Linux"
)]
struct Args {
    /// Named preset or hexadecimal RGB color, e.g. red or '#00ff88'.
    color: Option<String>,

    /// Lighting effect, e.g. normal, breath, spectrum, wave, corrugated or off.
    #[arg(short, long, value_enum)]
    mode: Option<LightingMode>,

    /// Brightness level from 0 (off) to 4 (maximum).
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(0..=4))]
    brightness: Option<u8>,

    /// Animation speed level from 1 (slowest) to 5 (fastest).
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=5))]
    speed: Option<u8>,

    /// Effect direction for wave-style modes.
    #[arg(short, long, value_parser = ["rtl", "ltr"])]
    direction: Option<String>,

    /// Stream dynamic software lighting animation in the terminal until Ctrl+C.
    #[arg(long)]
    stream: bool,

    /// Open the graphical color picker.
    #[arg(long, short)]
    gui: bool,

    /// Re-apply the last saved lighting configuration.
    #[arg(long)]
    restore: bool,

    /// Read and print the lighting state currently stored on the keyboard.
    #[arg(long)]
    status: bool,

    /// Light a single LED slot (0-127) white and all others off, to verify the per-key map.
    #[arg(long, value_name = "INDEX", value_parser = clap::value_parser!(u16).range(0..128))]
    probe_led: Option<u16>,

    /// List built-in color presets.
    #[arg(long)]
    list_presets: bool,

    /// Add this application to XDG autostart for the current user.
    #[arg(long)]
    enable_autostart: bool,

    /// Remove this application from XDG autostart.
    #[arg(long)]
    disable_autostart: bool,

    /// Target keyboard model (75 or 65). Auto-detected if omitted.
    #[arg(long, value_parser = ["75", "65"])]
    model: Option<String>,
}

#[inline]
fn tune_memory_allocator() {
    #[cfg(target_os = "linux")]
    unsafe {
        unsafe extern "C" {
            fn mallopt(
                param: std::os::raw::c_int,
                value: std::os::raw::c_int,
            ) -> std::os::raw::c_int;
        }
        // Limit max arenas to 2 to prevent excessive per-thread virtual/resident memory
        mallopt(-8, 2);
        // Trim threshold: 64 KB (default is up to 32MB)
        mallopt(-1, 64 * 1024);
        // Mmap threshold: 128 KB
        mallopt(-3, 128 * 1024);
    }
}

fn main() -> ExitCode {
    tune_memory_allocator();
    let args = Args::parse();

    let result = run_command(args);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("kreoctl: {error}");
            if let KreoError::PermissionDenied(_) = &error {
                eprintln!("\nTip: To control lighting without sudo, add the udev rule:");
                eprintln!(
                    "  echo 'KERNEL==\"hidraw*\", ATTRS{{idVendor}}==\"320f\", TAG+=\"uaccess\"' | sudo tee /etc/udev/rules.d/60-kreo-hive.rules"
                );
                eprintln!(
                    "  echo 'KERNEL==\"hidraw*\", ATTRS{{idVendor}}==\"258a\", ATTRS{{idProduct}}==\"010c\", TAG+=\"uaccess\"' | sudo tee -a /etc/udev/rules.d/60-kreo-hive.rules"
                );
                eprintln!("  sudo udevadm control --reload-rules && sudo udevadm trigger");
                eprintln!("Then unplug and replug the keyboard.");
            }
            ExitCode::FAILURE
        }
    }
}

fn run_command(args: Args) -> Result<(), KreoError> {
    if args.list_presets {
        print_presets();
        Ok(())
    } else if args.enable_autostart {
        config::set_autostart(true)
    } else if args.disable_autostart {
        config::set_autostart(false)
    } else if args.gui {
        #[cfg(feature = "gui")]
        {
            kreoctl::gui::run()
        }
        #[cfg(not(feature = "gui"))]
        {
            Err(KreoError::GuiNotCompiled)
        }
    } else if args.status {
        let cfg = device::read_current_lighting()?;
        let color = if cfg.multicolor {
            "rainbow".to_string()
        } else {
            cfg.color.hex()
        };
        println!(
            "Mode: {}\nColor: {color}\nBrightness: {}/4\nSpeed: {}/5\nDirection: {}",
            cfg.mode.name(),
            cfg.brightness,
            cfg.speed,
            if cfg.direction == 0 {
                "right to left"
            } else {
                "left to right"
            }
        );
        Ok(())
    } else if let Some(index) = args.probe_led {
        let mut buf = [0_u8; kreoctl::protocol::TOTAL_COLOR_BYTES];
        let i = index as usize * 3;
        buf[i..i + 3].copy_from_slice(&[255, 255, 255]);
        device::set_per_key_colors(&buf, kreoctl::protocol::BRIGHTNESS_MAX)?;
        println!("Lit LED slot {index}. Run `kreoctl --restore` to return to your saved lighting.");
        Ok(())
    } else if args.restore {
        let cfg = config::load_last_lighting()?;
        device::set_lighting(&cfg)?;
        println!(
            "Restored {} mode at {} (brightness {}, speed {})",
            cfg.mode.name(),
            cfg.color.hex(),
            cfg.brightness,
            cfg.speed
        );
        Ok(())
    } else if args.stream {
        let mode = args.mode.unwrap_or(LightingMode::Cycle);
        let color = match args.color {
            Some(c) => parse_color(&c)?,
            None => Rgb::new(255, 0, 0),
        };
        let speed = args.speed.unwrap_or(3);
        println!(
            "Streaming {} animation at speed {} (Press Enter or Ctrl+C to stop)...",
            mode.name(),
            speed
        );

        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = std::io::stdin().read_line(&mut line);
            stop_clone.store(true, Ordering::SeqCst);
        });

        let mut backend = device::open_device()?;
        animation::stream_animation(&mut backend, mode, color, speed, stop, None)
    } else if args.mode.is_some()
        || args.brightness.is_some()
        || args.speed.is_some()
        || args.direction.is_some()
        || args.color.is_some()
    {
        let mut cfg = config::load_last_lighting().unwrap_or_default();

        let has_explicit_color = args.color.is_some();
        if let Some(ref c) = args.color {
            cfg.color = parse_color(c)?;
            cfg.multicolor = false;
            if args.mode.is_none() {
                cfg.mode = LightingMode::Static;
            }
        }
        if let Some(m) = args.mode {
            cfg.mode = m;
            if m.is_inherently_multicolor() && !has_explicit_color {
                cfg.multicolor = true;
            }
        }
        if let Some(b) = args.brightness {
            cfg.brightness = b;
        }
        if let Some(s) = args.speed {
            cfg.speed = s;
        }
        if let Some(d) = args.direction.as_deref() {
            cfg.direction = u8::from(d == "ltr");
        }

        device::set_lighting(&cfg)?;
        config::save_last_lighting(&cfg)?;
        let model_msg = if let Some(d) = device::detect_connected_device() {
            format!(" on {}", d.variant.name())
        } else {
            String::new()
        };
        println!(
            "Applied {}{model_msg} (Mode: {}, Brightness: {}, Speed: {})",
            cfg.color.hex(),
            cfg.mode.name(),
            cfg.brightness,
            cfg.speed
        );
        Ok(())
    } else {
        #[cfg(feature = "gui")]
        {
            kreoctl::gui::run()
        }
        #[cfg(not(feature = "gui"))]
        {
            use clap::CommandFactory;
            let mut cmd = Args::command();
            cmd.print_help()?;
            println!();
            Ok(())
        }
    }
}

fn print_presets() {
    for (name, color) in PRESETS {
        println!("{name:12} {}", color.hex());
    }
}
