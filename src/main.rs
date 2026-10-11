use clap::Parser;
use kreoctl::{
    color::{PRESETS, Rgb, parse_color},
    config, device,
    error::KreoError,
};
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(
    name = "kreoctl",
    version,
    about = "Control Kreo Hive RGB lighting on Linux"
)]
struct Args {
    /// Named preset or hexadecimal RGB color, e.g. red or '#00ff88'.
    color: Option<String>,

    /// Open the graphical color picker.
    #[arg(long, short)]
    gui: bool,

    /// Re-apply the last saved color.
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
        let color = config::load_last_color()?;
        apply_and_save(color)
    } else if let Some(value) = args.color {
        let color = parse_color(&value)?;
        apply_and_save(color)
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

fn apply_and_save(color: Rgb) -> Result<(), KreoError> {
    device::set_color(color)?;
    config::save_last_color(color)?;
    println!("Applied {}", color.hex());
    Ok(())
}

fn print_presets() {
    for (name, color) in PRESETS {
        println!("{name:12} {}", color.hex());
    }
}
