# Hardware Protocol & Reverse Engineering: Kreo Hive RGB

This document captures verified hardware facts, packet layouts, reverse-engineered byte fields, and unverified assumptions for the **Kreo Hive RGB** mechanical keyboard.

---

## 1. Hardware Identifiers

- **Manufacturer:** Evision / Telink OEM
- **Device Label:** `320f:5055 Evision Kreo Hive RGB`
- **USB Vendor ID (VID):** `0x320f`
- **USB Product ID (PID):** `0x5055`
- **Linux Kernel Driver:** `hid-generic`
- **Exposed HID Interfaces:**
  - Interface `0002` -> `/dev/hidraw1` (standard keyboard input reports)
  - Interface `0003` -> `/dev/hidraw2` (vendor-specific control and lighting reports)

---

## 2. Verified HID Report Layout (Static RGB)

A 64-byte Output Report directed at Report ID `0x04` on Interface `0003` successfully modifies the static backlighting of the keyboard.

### Byte-by-Byte Map

| Offset | Verified Value | Field Description | Status |
| :---: | :---: | :--- | :--- |
| **0** | `0x04` | HID Report ID | **Verified** |
| **1** | `0x3b` | Vendor Command Header Byte 0 | **Verified** |
| **2** | `0x02` | Vendor Command Header Byte 1 | **Verified** |
| **3** | `0x06` | Vendor Command Header Byte 2 | **Verified** |
| **4** | `0x22` | Lighting Mode Indicator (Static / Custom RGB) | **Verified** |
| **5–8** | `0x00, 0x00, 0x00, 0x00` | Reserved / Null padding | Inferred padding |
| **9** | `0x06` | Parameter Header Byte 0 | **Verified** |
| **10** | `0x04` | Parameter Header Byte 1 | **Verified** |
| **11** | `0x02` | Parameter Header Byte 2 | **Verified** |
| **12–13** | `0x00, 0x00` | Padding | Inferred padding |
| **14** | `0x00..0xff` | **Red (R)** component (0–255) | **Verified** |
| **15** | `0x00..0xff` | **Green (G)** component (0–255) | **Verified** |
| **16** | `0x00..0xff` | **Blue (B)** component (0–255) | **Verified** |
| **17** | `0x08` | Lighting Parameter (Brightness or zone selector) | Inferred |
| **18–27** | `0x00..` | Padding | Inferred padding |
| **28** | `0x01` | Activation / Commit flag | Inferred |
| **29–63** | `0x00..` | Trailing null padding (zeros) | Inferred padding |

---

## 3. Verified Empirical Testing

- **Experiment 1 (Pure Red):**
  - Packet: Byte 14 = `0xff`, Byte 15 = `0x00`, Byte 16 = `0x00`.
  - Result: Keyboard illuminated in solid red.
- **Experiment 2 (Custom Hex):**
  - Packet with `#00ff88`: Byte 14 = `0x00`, Byte 15 = `0xff`, Byte 16 = `0x88`.
  - Result: Keyboard illuminated in seafoam green.

---

## 4. Unverified Fields & Future Research

1. **Brightness Steps:**
   - Byte 17 (`0x08`) or Byte 11 may control intensity levels. Needs packet capture from the official OEM software.
2. **Animation Modes:**
   - Byte 4 (`0x22`) likely selects the active lighting mode (e.g. breathing, wave, reactive, static).
3. **Speed & Direction:**
   - Bytes 5–8 or 18–27 may encode animation speed and wave direction for dynamic presets.
4. **Per-Key Matrix Addressing:**
   - White-label Telink controllers often feature multi-packet transfers for individual switch LEDs.
   - Per-key lighting is currently **unsupported** in `kreoctl` until USB traffic captures verify multi-report commands.

