# Changelog

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
