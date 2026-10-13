#!/usr/bin/env bash
set -euo pipefail

echo "=== Building kreoctl (release profile) ==="
cargo build --release --features gui

PREFIX="${PREFIX:-/usr/local}"
BIN_DIR="$PREFIX/bin"

echo "=== Installing binary to $BIN_DIR ==="
mkdir -p "$BIN_DIR"
install -m 755 target/release/kreoctl "$BIN_DIR/kreoctl"

echo "=== Installing udev rules ==="
if [ -d /etc/udev/rules.d ]; then
    if [ "$EUID" -eq 0 ]; then
        install -m 644 udev/99-kreoctl.rules /etc/udev/rules.d/99-kreoctl.rules
        udevadm control --reload-rules && udevadm trigger
        echo "udev rule installed and triggered."
    else
        echo "Note: Run the following to install udev rules for passwordless control:"
        echo "  sudo install -m 644 udev/99-kreoctl.rules /etc/udev/rules.d/99-kreoctl.rules"
        echo "  sudo udevadm control --reload-rules && sudo udevadm trigger"
    fi
fi

echo "=== Installation complete ==="
echo "Run 'kreoctl --help' or 'kreoctl --gui' to get started."
