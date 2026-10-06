use eframe::egui::{Context, FontData, FontDefinitions, FontFamily, FontId, RichText};

use crate::services::system_fonts::ScriptFont;

const INTER_REGULAR: &[u8] = include_bytes!("../../../../../assets/fonts/Inter-Regular.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../../../../assets/fonts/Inter-SemiBold.ttf");
const REGULAR: &str = "inter";
const SEMIBOLD: &str = "inter-semibold";
const ICONS: &str = "icons";
const DEFAULT_TEXT_FONT: &str = "Ubuntu-Light";
const ICON_FONT_MARKER: &str = "phosphor";

pub fn install(ctx: &Context, script_fonts: &[ScriptFont]) {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert(REGULAR.into(), FontData::from_static(INTER_REGULAR));
    fonts
        .font_data
        .insert(SEMIBOLD.into(), FontData::from_static(INTER_SEMIBOLD));
    for script in script_fonts {
        fonts
            .font_data
            .insert(script.name.into(), FontData::from_static(script.data));
    }
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    let (icons, emoji): (Vec<String>, Vec<String>) = fonts
        .families
        .get(&FontFamily::Proportional)
        .into_iter()
        .flatten()
        .filter(|name| name.as_str() != DEFAULT_TEXT_FONT)
        .cloned()
        .partition(|name| name.contains(ICON_FONT_MARKER));
    let text_family = |primary: &str| -> Vec<String> {
        std::iter::once(primary.to_owned())
            .chain(script_fonts.iter().map(|script| script.name.to_owned()))
            .chain(emoji.iter().cloned())
            .collect()
    };
    let proportional = text_family(REGULAR);
    let semibold = text_family(SEMIBOLD);
    let icons: Vec<String> = icons.into_iter().chain([REGULAR.to_owned()]).collect();
    let families = &mut fonts.families;
    families.insert(FontFamily::Proportional, proportional);
    families.insert(FontFamily::Name(SEMIBOLD.into()), semibold);
    families.insert(FontFamily::Name(ICONS.into()), icons);
    ctx.set_fonts(fonts);
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(ICONS.into()))
}

pub fn strong(text: impl Into<String>, size: f32) -> RichText {
    RichText::new(text).font(semibold(size))
}

pub fn glyph(text: &str, size: f32) -> RichText {
    RichText::new(text).font(icon(size))
}
