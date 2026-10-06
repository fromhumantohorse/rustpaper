# Changelog

## v0.3.0 — Resource diet

Measured on a real 2-monitor setup playing a 4K video (same settings before/after):

| | v0.2 | v0.3 |
|---|---|---|
| GPU 3D | 31 % | **2.9 %** |
| GPU video decoder | busy | **0 %** |
| VRAM | 331 MB | **115 MB** |
| RAM (working set) | 650 MB | **174 MB** |
| CPU | heavy 4K software decoding | **0.6 %** |

- **Automatic video optimization**: each video is re-encoded once, in a low-priority background process, into a lightweight copy at the exact resolution of every screen (H.264 tuned for fast decoding, FPS limit baked in). The original plays meanwhile; the engine switches over seamlessly. Playlist items are prepared ahead of time. Cache capped at 4 GB (least recently used evicted).
- Optimized copies are decoded on a single CPU thread: no GPU decoder load and minimal VRAM.
- Lighter rendering presets (no linear/correct downscaling or dithering unless *Quality* is selected), single-frame swap chain, no HDR peak detection.
- Crashes are now written to the log with their exact location.
- Installer: upgrading from a previous version no longer leaves `Program Files` empty (files are now always replaced and the old version is removed first). If you installed v0.2.1, simply run the v0.3.0 installer.
- Executables now carry proper version information (English description, file version).

## v0.2.1

- Windows installer (`.msi`): installs to Program Files with Desktop and Start menu shortcuts, in-place upgrades and clean uninstall.
- Reproducible packaging script (`scripts/package-windows.sh`) producing the portable zip, the installer and SHA-256 checksums.

## v0.2.0

- Brand-new Studio design: sidebar navigation, live "Now playing" card with engine memory graph, thumbnail grid, side inspector, toast notifications.
- 7 languages: English, French, Spanish, Russian, Arabic, Chinese, Hindi.
- Notification-area icon (Windows) with pause, mute and quit.
- Per-monitor pausing for maximized / fullscreen apps, battery policy, render-quality presets.
- Memory released while every screen is paused; engine idles at ~15 MB.
- Authenticated local control channel with a per-user single-instance lock.
- Crash-safe storage (fsync'd atomic writes, corrupt files quarantined instead of blocking startup).
- Per-monitor DPI awareness for scaled displays.
- Dozens of robustness fixes from a 12-pass audit.

## v0.1.0

- First release: video, audio and image wallpapers rendered with libmpv behind the desktop icons on Windows and X11.
