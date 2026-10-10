# Contributing to kreoctl

Thanks for checking out `kreoctl`! Contributions, bug reports, and especially protocol packet captures on similar hardware are very welcome.

Here are a few practical guidelines to keep things working and clean.

---

## Building and Testing Locally

1. Install dependencies:
   ```bash
   sudo pacman -S --needed rust cargo pkgconf hidapi
   ```
2. Run standard checks before opening a PR:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   ```

---

## Hardware Safety Guidelines

- **Don't guess packet bytes:** Sending random raw HID reports to microcontrollers can corrupt onboard flash memory or brick the keyboard. If you're adding support for a new mode, please document how you captured the packet (e.g. Wireshark with USBPcap on Windows).
- **Keep unit tests hardware-independent:** Tests should verify packet byte arrays in memory without requiring a physical keyboard to be plugged in.

---

## Commits and PRs

- Use clear, descriptive commit messages (e.g. `feat: add breathing effect mode`, `fix: check interface number`).
- Open a draft PR or issue first if you're planning a big architectural change.

All contributions are licensed under the [MIT License](LICENSE).
