mod config;
mod device;
mod gui;
mod protocol;

use clap::Parser;
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

    let result = if args.list_presets {
        print_presets();
        Ok(())
    } else if args.enable_autostart {
        config::set_autostart(true)
    } else if args.disable_autostart {
        config::set_autostart(false)
    } else if args.gui {
        gui::run()
    } else if args.restore {
        match config::load_last_color() {
            Some(color) => apply_and_save(color),
            None => Err("No saved color yet. Set a color first.".into()),
        }
    } else if let Some(value) = args.color {
        match protocol::parse_color(&value) {
            Some(color) => apply_and_save(color),
            None => Err(format!("Unknown color '{value}'. Use a preset or #RRGGBB.")),
        }
    } else {
        #[cfg(feature = "gui")]
        {
            gui::run()
        }
        #[cfg(not(feature = "gui"))]
        {
            use clap::CommandFactory;
            let mut cmd = Args::command();
            match cmd.print_help() {
                Ok(()) => {
                    println!();
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("kreoctl: {error}");
            ExitCode::FAILURE
        }
    }
}

fn apply_and_save(color: protocol::Rgb) -> Result<(), String> {
    device::set_color(color)?;
    config::save_last_color(color)?;
    println!("Applied {}", color.hex());
    Ok(())
}

fn print_presets() {
    for (name, color) in protocol::PRESETS {
        println!("{name:12} {}", color.hex());
    }
}
