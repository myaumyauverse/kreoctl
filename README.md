# kreoctl

A lightweight Linux CLI and GUI tool to control RGB lighting on the **Kreo Hive RGB** keyboard (`0x320f:0x5055`).

I built this because the official software is Windows-only, and I wanted a simple, native way to control the keyboard's backlighting on Linux without dealing with custom kernel drivers, wine, or running Python scripts as root. It talks directly to the keyboard via Linux `hidraw` using Rust.

> **Current status:** Works for static colors (presets and hex codes). Tested and verified on physical hardware. Brightness control, animations, and per-key lighting are not reverse-engineered yet.

---

## What works right now

- **CLI:** Quick commands to set colors (`kreoctl red`, `kreoctl cyan`, `kreoctl '#00ff88'`, `kreoctl off`).
- **GUI:** Optional graphical color picker and palette (`kreoctl --gui`).
- **Remember color:** Saves your choice to `~/.config/kreoctl/config.toml` so `kreoctl --restore` re-applies it.
- **Autostart:** Can write an XDG autostart file to restore your color when you log in.
- **Safe:** Runs in userspace via `hidraw` with a simple udev rule. No kernel modules, no root daemon.

---

## Supported hardware

- **Keyboard:** Kreo Hive RGB mechanical keyboard
- **USB IDs:** `320f:5055` (Evision / Telink controller)
- **Protocol:** 64-byte HID output report on interface `0003` (`/dev/hidraw2`).

Other keyboards with this vendor ID might share the protocol, but this tool is specifically tested on the Kreo Hive.

---

## How to build and run

### 1. Install prerequisites (Arch Linux)

```bash
sudo pacman -S --needed rust cargo pkgconf hidapi
```

### 2. Build

```bash
git clone https://github.com/myaumyauverse/kreoctl.git
cd kreoctl

# Build the fast CLI binary
cargo build --release

# Or build with the GUI included
cargo build --release --features gui
```

The compiled binary will be in `target/release/kreoctl`.

---

## Usage

```bash
# See all options
./target/release/kreoctl --help

# Set built-in presets
./target/release/kreoctl red
./target/release/kreoctl blue
./target/release/kreoctl cyan
./target/release/kreoctl off

# Set any hex color
./target/release/kreoctl '#00ff88'
./target/release/kreoctl '#ff5500'

# List all available presets
./target/release/kreoctl --list-presets

# Restore the last saved color
./target/release/kreoctl --restore

# Open the graphical color picker (if built with --features gui)
./target/release/kreoctl --gui

# Enable/disable restoring color automatically at login
./target/release/kreoctl --enable-autostart
./target/release/kreoctl --disable-autostart
```

---

## Running without `sudo` (udev rule)

By default, Linux protects raw HID devices (`/dev/hidraw*`) so only root can write to them. To use `kreoctl` as your normal user, add a udev rule:

```bash
echo 'KERNEL=="hidraw*", ATTRS{idVendor}=="320f", ATTRS{idProduct}=="5055", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/60-kreo-hive.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Then unplug and plug the keyboard back in.

---

## Notes & Architecture

If you're interested in the code or reverse engineering notes:
- [Hardware Protocol](docs/HARDWARE_PROTOCOL.md) — The 64-byte packet layout and what each byte does.
- [Architecture](docs/ARCHITECTURE.md) — How the code is organized.
- [Project Rationale](docs/PROJECT_RATIONALE.md) — Why I chose userspace Rust instead of a kernel module or Python.
- [Alternatives Considered](docs/ALTERNATIVES.md) — Comparison with existing tools and OpenRGB.

---

## License

[MIT](LICENSE)
