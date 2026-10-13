# Hardware Protocol & Reverse Engineering: Kreo Hive RGB

This document captures verified hardware facts, packet layouts, reverse-engineered byte fields, and protocol architecture for the **Kreo Hive RGB** mechanical keyboard.

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

## 2. Verified HID Report Layout (64-Byte Output Report)

A 64-byte Output Report directed at Report ID `0x04` on Interface `0003` modifies backlighting and animation states.

### Byte-by-Byte Map

| Offset | Constant / Range | Field Description | Status |
| :---: | :---: | :--- | :--- |
| **0** | `0x04` | HID Report ID (`REPORT_ID`) | **Verified** |
| **1** | `0x3b` | Vendor Command Header Byte 0 (`CMD_PREFIX[0]`) | **Verified** |
| **2** | `0x02` | Vendor Command Header Byte 1 (`CMD_PREFIX[1]`) | **Verified** |
| **3** | `0x06` | Vendor Command Header Byte 2 (`CMD_PREFIX[2]`) | **Verified** |
| **4** | `0x22..0x26` | **Lighting Mode Indicator** (see Mode table below) | **Verified** |
| **5** | `0x01..0x05` | **Animation Speed Parameter** (1 = slowest, 5 = fastest) | **Verified** |
| **6–8** | `0x00, 0x00, 0x00` | Reserved / Null padding | Inferred padding |
| **9** | `0x06` | Parameter Header Byte 0 (`PARAM_HEADER[0]`) | **Verified** |
| **10** | `0x04` | Parameter Header Byte 1 (`PARAM_HEADER[1]`) | **Verified** |
| **11** | `0x02` | Parameter Header Byte 2 (`PARAM_HEADER[2]`) | **Verified** |
| **12–13** | `0x00, 0x00` | Padding | Inferred padding |
| **14** | `0x00..0xff` | **Red (R)** component (0–255) | **Verified** |
| **15** | `0x00..0xff` | **Green (G)** component (0–255) | **Verified** |
| **16** | `0x00..0xff` | **Blue (B)** component (0–255) | **Verified** |
| **17** | `0x02..0x0a` | **Brightness Level** (`brightness * 2`, 1..=5 -> `0x02`..`0x0a`) | **Verified** |
| **18–27** | `0x00..` | Padding | Inferred padding |
| **28** | `0x01` | Activation / Commit flag (`COMMIT_FLAG`) | **Verified** |
| **29–63** | `0x00..` | Trailing null padding (zeros) | Inferred padding |

---

## 3. Lighting Mode Identifiers (Offset 4)

| Mode Name | Byte Value | Description |
| :--- | :---: | :--- |
| **Static** | `0x22` | Solid backlighting using color specified at bytes 14..16 |
| **Breathing** | `0x23` | Hardware breathing effect pulsating the chosen color |
| **Spectrum Cycle** | `0x24` | Continuous cycling through the full RGB color wheel |
| **Rainbow Wave** | `0x25` | Dynamic horizontal color wave across the key matrix |
| **Reactive** | `0x26` | Key illumination triggered dynamically on keypress |
| **Off** | `0x22` | Static mode with RGB set to `0x00, 0x00, 0x00` |

---

## 4. Software-Driven Dynamic Streaming

In addition to the microcontroller's internal hardware animation modes, `kreoctl` implements a client-side animation engine (`src/animation.rs`):
- Generates continuous color sequences (e.g. mathematical HSV rainbow wheel or sine-wave breathing) in userspace.
- Emits static frame packets over `/dev/hidraw2` at 20–60 FPS with speed pacing.
- Allows external software or user scripts to orchestrate custom patterns without relying on stock firmware routines.

---

## 5. Security & Permissions

Because `/dev/hidraw*` character devices default to `root:root 0600`, `kreoctl` includes a standard udev rule:

```udev
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="320f", ATTRS{idProduct}=="5055", TAG+="uaccess"
```

The `TAG+="uaccess"` rule tells `systemd-logind` to dynamically grant read/write access to the currently active desktop console session, eliminating the need for `sudo` or insecure setuid binaries.
