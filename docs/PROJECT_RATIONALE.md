# Why I Built `kreoctl`

When I plugged my Kreo Hive RGB keyboard into my Linux machine, I ran into a familiar problem: the official software is Windows-only. There was no native Linux utility to set a static color, and every time the keyboard lost power or the system rebooted, the backlighting would reset.

---

## 1. Why not the existing Linux driver project?

Someone previously created a project for this keyboard (`adarshg98765/Kreo_Hive_keyboard_Rgb_driver_linux`). That project was a helpful reference because it proved that the hardware accepts raw HID reports. However, it had several practical issues:

1. **It used a custom Linux kernel module (`rgb_driver.ko`):** Writing a kernel driver just to send a lighting packet is overkill. Custom kernel modules break whenever the Linux kernel updates unless you set up DKMS.
2. **Secure Boot issues:** Unsigned out-of-tree kernel modules won't load on modern Linux installations with Secure Boot enabled without enrolling MOK keys.
3. **Incomplete code:** The public repository's C source file was literally cut off at line 55 mid-function.
4. **Python dependencies:** The companion GUI needed a Python environment and root privileges to write to sysfs.

---

## 2. Why a userspace Rust tool?

Linux already exposes raw HID devices through `/dev/hidraw*`. You don't need a kernel module to send a vendor output report—any userspace program with access to the hidraw device node can do it.

By writing this in Rust:
- **No kernel module needed:** It talks directly to `hidraw` via `hidapi`. It runs on any Linux kernel and doesn't care about Secure Boot.
- **Fast and self-contained:** Compiles to a single binary with no Python or runtime dependencies. It starts in milliseconds, making it great for autostart scripts at login.
- **Safe permissions:** A simple 1-line udev rule gives your user account access to the keyboard's hidraw node so you never have to run `sudo` to change colors.

---

## 3. Why not OpenRGB?

OpenRGB is the ideal home for peripheral lighting on Linux. However:
- `320f:5055` is a generic white-label OEM ID used by several different keyboards (Kreo Hive, EvoFox Ronin, Rainy 75, etc.). OpenRGB currently has multiple open issues for this ID because different boards have slight protocol quirks.
- Building a standalone, focused tool solved my immediate problem and allowed me to test and document the packet structure thoroughly.
- Having verified packet layouts and working Rust code makes it much easier to contribute back to OpenRGB or another universal project down the road.
