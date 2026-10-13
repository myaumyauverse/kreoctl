# kreoctl

A lightweight Linux CLI and GUI tool to control RGB lighting on the **Kreo Hive RGB** mechanical keyboard (`0x320f:0x5055`).

I built this because the official software is Windows-only, and I wanted a simple, native way to control the keyboard's backlighting on Linux without dealing with custom kernel drivers, wine, or running Python scripts as root. It talks directly to the keyboard via Linux `hidraw` using 100% safe Rust.

---

## What works

- **CLI:** Instant lighting changes with presets (`red`, `cyan`, `off`), hex codes (`#00ff88`), modes, brightness, and speed.
- **Lighting Modes:** Support for Static, Breathing, Spectrum Cycle, Rainbow Wave, Reactive, and Off.
- **Brightness & Speed:** Granular 5-level brightness and animation speed control (`--brightness 1..5`, `--speed 1..5`).
- **Real-Time Streaming:** Live terminal software animation loop (`kreoctl --stream --mode cycle`).
- **GUI:** Optional graphical interface (`kreoctl --gui`) with live color preview, mode selectors, and sliders.
- **Persistent State:** Automatically remembers your color, mode, brightness, and speed in `~/.config/kreoctl/config.toml`.
- **System Integration:** Includes standard udev rules, systemd user restore service, and an XDG desktop entry.
- **Testing & Mocking:** Full test suite (34 tests) with an in-memory `MockBackend` to verify protocols without hardware attached.

---

## Supported hardware

- **Keyboard:** Kreo Hive RGB mechanical keyboard
- **USB IDs:** `320f:5055` (Evision / Telink controller)
- **Protocol:** 64-byte HID output report on interface `0003` (`/dev/hidraw2`).

---

## Installation & Build

### 1. Quick Install from Source

```bash
git clone https://github.com/myaumyauverse/kreoctl.git
cd kreoctl
./install.sh
```

### 2. Manual Cargo Build

```bash
# Fast, lightweight CLI-only binary (compiles in seconds)
cargo build --release

# Or compile with GUI support (eframe / egui)
cargo build --release --features gui
```

The compiled binary will be located at `target/release/kreoctl`.

---

## Usage

```bash
# View all flags and options
kreoctl --help

# Set built-in color presets
kreoctl red
kreoctl cyan
kreoctl '#00ff88'

# Set lighting modes
kreoctl --mode breathing red
kreoctl --mode wave --speed 5
kreoctl --mode cycle --brightness 3
kreoctl --mode off

# Stream real-time software rainbow animation (Ctrl+C or Enter to stop)
kreoctl --stream --mode cycle --speed 4

# Open the graphical interface
kreoctl --gui

# Restore saved settings
kreoctl --restore

# List color presets
kreoctl --list-presets

# Manage desktop autostart
kreoctl --enable-autostart
kreoctl --disable-autostart
```

---

## Running without `sudo` (udev rule)

To access the keyboard's lighting interface as a regular user, install the included udev rule:

```bash
sudo cp udev/99-kreoctl.rules /etc/udev/rules.d/99-kreoctl.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Then unplug and reconnect your keyboard once.

---

## Automatic Restore on Login (systemd)

You can automatically re-apply your last saved lighting whenever you log in via systemd user services:

```bash
mkdir -p ~/.config/systemd/user/
cp systemd/kreoctl-restore.service ~/.config/systemd/user/
systemctl --user enable --now kreoctl-restore.service
```

---

## Architecture & Documentation

- [Hardware Protocol](docs/HARDWARE_PROTOCOL.md) — 64-byte packet layout, mode bytes, and intensity parameters.
- [System Architecture](docs/ARCHITECTURE.md) — Component responsibilities, `DeviceBackend` trait, and runtime data flow.
- [Project Rationale](docs/PROJECT_RATIONALE.md) — Why safe Rust userspace was chosen over kernel modules or Python.
- [Alternatives Considered](docs/ALTERNATIVES.md) — Technical comparison with OpenRGB and other solutions.

---

## License

[MIT](LICENSE)
