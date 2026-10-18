# Hardware Protocol & Reverse Engineering: Kreo Hive RGB

This document captures verified hardware facts, packet layouts, reverse-engineered byte fields, and protocol architecture for the **Kreo Hive RGB** mechanical keyboard.

---

## 1. Hardware Identifiers

- **Manufacturer:** Evision / Telink OEM
- **Device Label:** `320f:5055 Evision Kreo Hive RGB`
- **USB Vendor ID (VID):** `0x320f`
- **USB Product ID (PID):** `0x5055` (Hive 75, verified). The Hive 65 is assumed to be `0x5064`; this is unverified.
- **Linux Kernel Driver:** `hid-generic`
- **Exposed HID Interfaces:**
  - USB interface 0: standard keyboard input reports
  - USB interface 1 (Usage Page `0xFF1C`): vendor-specific control and lighting reports
  - The `/dev/hidrawN` numbers depend on what else is connected.

---

## 2. Report Format (EVision V2)

The Hive uses the EVision V2 controller protocol (the same one OpenRGB's
`EVisionV2KeyboardController` implements). Every transaction is a 64-byte
report on Report ID `0x04`, sent to the vendor interface (Usage Page `0xFF1C`).
The keyboard answers each request by echoing it back with a status byte and,
for reads, the requested data.

| Offset | Field |
| :---: | :--- |
| 0 | Report ID `0x04` |
| 1–2 | Checksum: 16-bit little-endian sum of bytes 3..64 |
| 3 | Command |
| 4 | Payload size (max 56) |
| 5–6 | Address / offset, 16-bit little-endian |
| 7 | Status in replies (0 = OK) |
| 8–63 | Payload |

### Commands

| Byte | Command |
| :---: | :--- |
| `0x01` / `0x02` | Begin / end configuration transaction |
| `0x03` | Read capabilities (`AA 55 .. .. .. <led slots>`) |
| `0x05` / `0x06` | Read / write config memory |
| `0x0A` / `0x0B` | Read / write custom (per-key) color memory, 512 bytes per colorset |
| `0x12` / `0x13` | Stream direct (software) colors / leave direct mode |

## 3. Config Memory

Byte `0x00` holds the active onboard profile (0–2). Each profile has a
`0x40`-byte block starting at `0x01 + profile * 0x40`. The lighting block:

| Offset | Field |
| :---: | :--- |
| `0x00` | Mode (table below) |
| `0x01` | Brightness, 0 (off) .. 4 (max) |
| `0x02` | Speed, 0 (fastest) .. 5 (slowest) |
| `0x03` | Direction, 1 = right-to-left / upwards, 0 = left-to-right / downwards |
| `0x04` | Random/rainbow color flag (`0xFF` on, `0x00` off) |
| `0x05–0x07` | Mode color R, G, B |
| `0x11` | LED mode (Surmount color index) |
| `0x19` | Active custom colorset for Custom mode |

Writing the block at offset `0x00` (instead of the profile block) corrupts the
profile index and shifts every parameter by one byte; `kreoctl` repairs an
out-of-range profile index before writing.

### Modes

| Byte | Kreo name | Byte | Kreo name |
| :---: | :--- | :---: | :--- |
| `0x01` | Corrugated | `0x0B` | Flowers |
| `0x02` | Cloud | `0x0C` | Roll (vertical) |
| `0x03` | Serpentine | `0x0D` | Wave |
| `0x04` | Spectrum | `0x0E` | Cartoon |
| `0x05` | Breath | `0x0F` | Rain |
| `0x06` | Normal (static) | `0x10` | Scan |
| `0x07` | Reaction | `0x11` | Surmount |
| `0x08` | Ripples | `0x12` | Speed |
| `0x09` | Traverse | `0x14` | Custom (per-key) |
| `0x0A` | Stars | | |

"Off" is Normal mode at brightness 0.

## 4. Per-Key LED Map

Capabilities report 128 LED slots. Custom colors are 3 bytes (R, G, B) per
slot, numbered **down each matrix column, 8 slots per column**:
`slot = column * 8 + row`. Mapped on a Hive 75 by lighting single slots:

| Row | Keys (matrix column) |
| :---: | :--- |
| 0 | Esc 0, F1–F12 1–12, Del 13 |
| 1 | \` 0, 1–0 1–10, - 11, = 12, Backspace 13 |
| 2 | Tab 0, Q–P 1–10, [ 11, ] 12, \\ 13, Ins 14, PgDn 15 |
| 3 | Caps 0, A–; 1–10, ' 11, Enter 13, End 14, PgUp 15 |
| 4 | LShift 0, Z–/ 2–11, RShift 13, Up 14 |
| 5 | Ctrl 0, Win 1, Alt 2, Space 3–8, RAlt 9, Fn 10, RCtrl 12, Left 13, Down 14, Right 15 |

The knob has no LED. The Hive 65 map is assumed to match for shared keys and
has not been verified.

## 5. Live Sync

The keyboard can change mode, brightness and speed itself (Fn+Ins,
Fn+arrows) without sending any report. The GUI's device worker re-reads the
active profile block every 400 ms and mirrors changes into the app.

Software animations (`--stream`) use the direct-color commands, so the onboard
effect resumes when streaming stops.

## 6. Security & Permissions

Because `/dev/hidraw*` character devices default to `root:root 0600`, `kreoctl` includes a standard udev rule:

```udev
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="320f", ATTRS{idProduct}=="5055", TAG+="uaccess"
```

The `TAG+="uaccess"` rule tells `systemd-logind` to dynamically grant read/write access to the currently active desktop console session, eliminating the need for `sudo` or insecure setuid binaries.
