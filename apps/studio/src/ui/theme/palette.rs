use eframe::egui::Color32;
use rustpaper_domain::wallpaper::MediaKind;

pub const BACKGROUND: Color32 = Color32::from_rgb(11, 14, 26);
pub const SIDEBAR: Color32 = Color32::from_rgb(13, 17, 33);
pub const SURFACE: Color32 = Color32::from_rgb(20, 26, 46);
pub const SURFACE_HOVER: Color32 = Color32::from_rgb(28, 35, 61);
pub const INPUT: Color32 = Color32::from_rgb(17, 22, 39);
pub const BORDER: Color32 = Color32::from_rgb(34, 42, 69);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(52, 62, 100);
pub const TEXT: Color32 = Color32::from_rgb(232, 235, 247);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(140, 147, 173);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(93, 101, 132);
pub const ACCENT_START: Color32 = Color32::from_rgb(79, 139, 255);
pub const ACCENT_END: Color32 = Color32::from_rgb(160, 92, 255);
pub const ACCENT: Color32 = Color32::from_rgb(120, 116, 255);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(35, 36, 76);
pub const HERO_START: Color32 = Color32::from_rgb(24, 31, 64);
pub const HERO_END: Color32 = Color32::from_rgb(36, 27, 66);
pub const SUCCESS: Color32 = Color32::from_rgb(52, 211, 153);
pub const SUCCESS_SOFT: Color32 = Color32::from_rgb(16, 53, 46);
pub const WARNING: Color32 = Color32::from_rgb(245, 180, 84);
pub const WARNING_SOFT: Color32 = Color32::from_rgb(58, 43, 20);
pub const DANGER: Color32 = Color32::from_rgb(247, 118, 142);
pub const DANGER_SOFT: Color32 = Color32::from_rgb(60, 24, 36);
pub const PLAYLIST_TINT: Color32 = WARNING;
pub const METRIC_RAM: Color32 = Color32::from_rgb(79, 139, 255);
pub const METRIC_GPU: Color32 = Color32::from_rgb(244, 63, 94);
pub const METRIC_VRAM: Color32 = Color32::from_rgb(167, 139, 250);
pub const SCRIM: Color32 = Color32::from_rgba_premultiplied(5, 7, 14, 200);
pub const ON_ACCENT: Color32 = Color32::WHITE;

pub fn media_tint(kind: MediaKind) -> Color32 {
    match kind {
        MediaKind::Video => ACCENT_START,
        MediaKind::Audio => ACCENT_END,
        MediaKind::Image => Color32::from_rgb(45, 212, 191),
    }
}

pub fn mix(from: Color32, to: Color32, ratio: f32) -> Color32 {
    let ratio = ratio.clamp(0.0, 1.0);
    let channel =
        |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * ratio).round() as u8;
    Color32::from_rgba_premultiplied(
        channel(from.r(), to.r()),
        channel(from.g(), to.g()),
        channel(from.b(), to.b()),
        channel(from.a(), to.a()),
    )
}

pub fn rgb(color: Color32) -> [u8; 3] {
    [color.r(), color.g(), color.b()]
}
