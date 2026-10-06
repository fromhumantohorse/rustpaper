pub mod fonts;
pub mod palette;
pub mod style;

use eframe::egui::Context;

use crate::services::system_fonts::ScriptFont;

pub fn install(ctx: &Context, script_fonts: &[ScriptFont]) {
    fonts::install(ctx, script_fonts);
    style::apply(ctx);
}
