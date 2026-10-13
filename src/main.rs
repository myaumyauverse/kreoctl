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

    /// Lighting mode (static, breathing, cycle, wave, reactive, off).
    #[arg(short, long, value_enum)]
    mode: Option<LightingMode>,

    /// Brightness level from 1 (dimmest) to 5 (maximum).
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=5))]
    brightness: Option<u8>,

    /// Animation speed level from 1 (slowest) to 5 (fastest).
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=5))]
    speed: Option<u8>,

    /// Stream dynamic software lighting animation in the terminal until Ctrl+C.
    #[arg(long)]
    stream: bool,

    /// Open the graphical color picker.
    #[arg(long, short)]
    gui: bool,

    /// Re-apply the last saved lighting configuration.
    #[arg(long)]
    restore: bool,

    /// List built-in color presets.
    #[arg(long)]
    list_presets: bool,

    /// Add this application to XDG autostart for the current user.
    #[arg(long)]
    enable_autostart: bool,

    /// Remove this application from XDG autostart.
    #[arg(long)]
    disable_autostart: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let result = run_command(args);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("kreoctl: {error}");
            if let KreoError::PermissionDenied(_) = &error {
                eprintln!("\nTip: To control lighting without sudo, add the udev rule:");
                eprintln!(
                    "  echo 'KERNEL==\"hidraw*\", ATTRS{{idVendor}}==\"320f\", ATTRS{{idProduct}}==\"5055\", TAG+=\"uaccess\"' | sudo tee /etc/udev/rules.d/60-kreo-hive.rules"
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
        || args.color.is_some()
    {
        let mut cfg = config::load_last_lighting().unwrap_or_default();

        if let Some(c) = args.color {
            cfg.color = parse_color(&c)?;
        }
        if let Some(m) = args.mode {
            cfg.mode = m;
        }
        if let Some(b) = args.brightness {
            cfg.brightness = b;
        }
        if let Some(s) = args.speed {
            cfg.speed = s;
        }

        device::set_lighting(&cfg)?;
        config::save_last_lighting(&cfg)?;
        println!(
            "Applied {} (Mode: {}, Brightness: {}, Speed: {})",
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
