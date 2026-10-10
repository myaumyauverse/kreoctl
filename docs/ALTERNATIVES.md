# Existing Tools & Alternatives

When looking for ways to control the Kreo Hive keyboard on Linux, I evaluated three main options before writing `kreoctl`.

---

## 1. Quick Comparison

| Approach | Works on Linux? | Needs Kernel Driver? | Works with Secure Boot? | Supports VID `320f:5055`? |
| :--- | :---: | :---: | :---: | :---: |
| **Official Kreo Software** | ❌ (Windows only) | N/A | N/A | ✅ |
| **`adarshg98765/` Linux Driver** | ✅ | ⚠️ Yes (`.ko`) | ❌ (Unsigned) | ✅ |
| **OpenRGB** | ✅ | ❌ No | ✅ | ❌ (Open issues) |
| **`kreoctl` (This Project)** | ✅ | ❌ No (Userspace `hidraw`) | ✅ | ✅ |

---

## 2. Detailed Breakdown

### Option 1: The Custom Kernel Driver (`adarshg98765`)
- **How it works:** A C kernel module binds to the keyboard's HID interface, sends the raw report, and exposes a sysfs node at `/sys/bus/hid/devices/.../rgb_color`.
- **Why I didn't use it:**
  - It breaks every time the kernel updates unless you maintain DKMS scripts.
  - It doesn't load under UEFI Secure Boot without custom key signing.
  - The repo's C file is truncated mid-function.
  - You don't actually need kernel space to send HID output reports on Linux.

### Option 2: Upstreaming directly to OpenRGB
- **How it works:** Submitting a C++ driver into OpenRGB's codebase.
- **Why I built standalone first:**
  - `320f:5055` is used across many white-label Telink-based mechanical keyboards (Kreo Hive, EvoFox Ronin, Rainy 75, etc.). OpenRGB has unresolved issues for this ID because different OEM revisions behave differently.
  - Building a small, dedicated tool solved my immediate problem right away and allowed me to document the exact packet format cleanly.
  - Documenting the protocol here makes it much easier to contribute upstream when ready.

### Option 3: Python script with `hidapi` or `pyusb`
- **How it works:** A Python script sending the same 64-byte packet.
- **Why Rust instead:**
  - Rust produces a single compiled binary with no Python interpreter startup overhead or pip virtual environment headaches.
  - The CLI starts up instantly, which is ideal for a login script restoring lighting in the background.
