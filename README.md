# kreoctl

A lightweight Linux CLI and GUI tool to control RGB lighting on the **Kreo Hive RGB** mechanical keyboard (`0x320f:0x5055`).

I built this because the official software is Windows-only, and I wanted a simple, native way to control the keyboard's backlighting on Linux without dealing with custom kernel drivers, wine, or running Python scripts as root. It talks directly to the keyboard via Linux `hidraw` using 100% safe Rust.

---

## What works

- **CLI:** Instant lighting changes with presets (`red`, `cyan`, `off`), hex codes (`#00ff88`), modes, brightness, and speed.
- **Lighting Modes:** Support for Static, Breathing, Spectrum Cycle, Rainbow Wave, Reactive, and Off.
- **Brightness, Speed & Direction:** Firmware brightness levels (`--brightness 0..4`, 0 = off), animation speed (`--speed 1..5`) and wave direction (`--direction rtl|ltr`).
- **Real-Time Streaming:** Live terminal software animation loop (`kreoctl --stream --mode cycle`).
- **GUI:** Graphical interface (`kreoctl --gui`) with a live keyboard visualizer, per-key colors, and two-way sync with the keyboard.
- **Persistent State:** Automatically remembers your color, mode, brightness, and speed in `~/.config/kreoctl/config.toml`.
- **System Integration:** Includes standard udev rules, systemd user restore service, and an XDG desktop entry.
- **Testing & Mocking:** Full test suite (34 tests) with an in-memory `MockBackend` to verify protocols without hardware attached.

---

## Supported hardware

| Keyboard | USB ID | Status |
| :--- | :--- | :--- |
| Kreo Hive 75 | `320f:5055` | Supported and verified, including the per-key LED map |
| Kreo Hive 65 | expected `320f:5064` (or a Bytech `258a:010c` variant) | **Experimental.** Lighting effects should work; the per-key LED map, on-screen layout and USB ID are unverified |

- **Protocol:** EVision V2, 64-byte HID reports on USB interface 1 (Usage Page `0xFF1C`). The `/dev/hidrawN` number varies between machines.

**Hive 65 owners:** please open an issue with your `lsusb` output and which key lights up
for a few `kreoctl --probe-led N` values (e.g. 0, 1, 8, 10). Then run `kreoctl --restore`;
the probe overwrites the keyboard's first custom colorset.

---

## Installation & Build

### 1. Quick Install from Source

```bash
git clone https://github.com/myaumyauverse/kreoctl.git
cd kreoctl
./install.sh
```

Run as a normal user, `install.sh` installs into `~/.local/bin` (no sudo) and adds a
"kreoctl" entry to your app launcher. Run it with `sudo` to install into
`/usr/local/bin` instead. Re-run it after pulling or changing the code.

### 2. Manual Cargo Build

```bash
# Full build: CLI + GUI (default)
cargo build --release

# Headless CLI-only build (no GUI dependencies)
cargo build --release --no-default-features
```

The compiled binary will be located at `target/release/kreoctl`. Note that
`--features` / `--no-default-features` are **cargo** flags; `kreoctl` itself
does not accept them.

---

## Usage

```bash
# View all flags and options
kreoctl --help

# Open the graphical interface
kreoctl --gui

# Show what the keyboard is doing right now (also reflects Fn+Ins changes)
kreoctl --status

# Solid colors (Normal mode)
kreoctl red
kreoctl '#00ff88' --mode normal

# Effects: speed 1 (slow) .. 5 (fast), brightness 0 (off) .. 4 (max)
kreoctl --mode breath --speed 5 --brightness 3 red
kreoctl --mode spectrum
kreoctl --mode corrugated --direction ltr   # rtl or ltr
kreoctl --mode off

# Stream real-time software rainbow animation (Enter to stop)
kreoctl --stream --mode spectrum --speed 4

# Restore the last applied lighting
kreoctl --restore

# List color presets
kreoctl --list-presets

# Light a single LED slot to check the per-key map
# (overwrites the keyboard's first custom colorset; run --restore after)
kreoctl --probe-led 44

# Manage desktop autostart
kreoctl --enable-autostart
kreoctl --disable-autostart
```

Modes: corrugated, cloud, serpentine, spectrum, breath, normal, reaction, ripples,
traverse, stars, flowers, roll, wave, cartoon, rain, scan, surmount, speed, custom, off.

### Using the GUI

- **Effects:** click a mode in the grid. Brightness, speed and direction are only
  enabled when the selected effect supports them.
- **Color:** click a preset swatch, type RGB values, or drag on the spectrum.
  "Rainbow" switches supported effects to random colors.
- **Per-key colors:** click keys on the visualizer or use a quick select (WASD,
  Arrows, …), then pick a color. The keyboard switches to Custom mode with those
  colors. "Reset Key Colors" clears them.
- **Live sync:** changes made on the keyboard itself (Fn+Ins, Fn+arrows) appear in
  the app within about half a second. "Re-read" forces an immediate refresh.

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
