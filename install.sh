#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

echo "=== Building kreoctl (release profile) ==="
cargo build --release

# Root installs go system-wide; a normal user installs into ~/.local without sudo.
if [ "$EUID" -eq 0 ]; then
    PREFIX="${PREFIX:-/usr/local}"
else
    PREFIX="${PREFIX:-$HOME/.local}"
fi
BIN_DIR="$PREFIX/bin"
APP_DIR="$PREFIX/share/applications"

echo "=== Installing binary to $BIN_DIR ==="
mkdir -p "$BIN_DIR"
install -m 755 target/release/kreoctl "$BIN_DIR/kreoctl"

echo "=== Installing desktop entry to $APP_DIR ==="
mkdir -p "$APP_DIR"
# Absolute path so app launchers find it even when $BIN_DIR is not on their PATH.
sed "s|^Exec=kreoctl|Exec=$BIN_DIR/kreoctl|" extra/kreoctl.desktop > "$APP_DIR/kreoctl.desktop"
chmod 644 "$APP_DIR/kreoctl.desktop"

echo "=== Installing udev rules ==="
if [ -d /etc/udev/rules.d ]; then
    if [ "$EUID" -eq 0 ]; then
        install -m 644 udev/99-kreoctl.rules /etc/udev/rules.d/99-kreoctl.rules
        udevadm control --reload-rules && udevadm trigger
        echo "udev rule installed and triggered."
    elif ls /etc/udev/rules.d/*kreo* >/dev/null 2>&1; then
        echo "udev rule already present."
    else
        echo "Note: Run the following to install udev rules for passwordless control:"
        echo "  sudo install -m 644 udev/99-kreoctl.rules /etc/udev/rules.d/99-kreoctl.rules"
        echo "  sudo udevadm control --reload-rules && sudo udevadm trigger"
    fi
fi

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) echo "Warning: $BIN_DIR is not on your PATH. Add: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac

echo "=== Installation complete ==="
echo "Run 'kreoctl --help' or 'kreoctl --gui' to get started."
