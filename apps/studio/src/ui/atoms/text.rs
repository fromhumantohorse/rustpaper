use std::sync::Arc;

use eframe::egui::text::{LayoutJob, TextWrapping};
use eframe::egui::{Align, Color32, FontId, Galley, TextFormat, Ui};

use crate::ui::theme::fonts::{icon, strong};
use crate::ui::theme::palette::TEXT_FAINT;

const ELLIPSIS: char = '…';
const SECTION_LABEL_SIZE: f32 = 11.0;
const ICON_GAP: f32 = 6.0;
const ICON_SCALE: f32 = 1.15;

pub struct TextStyle {
    pub font: FontId,
    pub color: Color32,
}

pub fn single_line(ui: &Ui, text: &str, style: TextStyle, max_width: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(
        text.to_owned(),
        TextFormat {
            font_id: style.font,
            color: style.color,
            ..TextFormat::default()
        },
    );
    job.wrap = TextWrapping {
        max_width: max_width.max(0.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some(ELLIPSIS),
    };
    ui.fonts(|fonts| fonts.layout_job(job))
}

pub fn icon_text(ui: &Ui, glyph: &str, text: &str, style: TextStyle) -> Arc<Galley> {
    let mut job = LayoutJob::default();
    let format = |font_id: FontId| TextFormat {
        font_id,
        color: style.color,
        valign: Align::Center,
        ..TextFormat::default()
    };
    if !glyph.is_empty() {
        job.append(glyph, 0.0, format(icon(style.font.size * ICON_SCALE)));
    }
    if !text.is_empty() {
        let gap = if glyph.is_empty() { 0.0 } else { ICON_GAP };
        job.append(text, gap, format(style.font.clone()));
    }
    ui.fonts(|fonts| fonts.layout_job(job))
}

pub fn section_label(ui: &mut Ui, text: &str) {
    ui.label(strong(text.to_uppercase(), SECTION_LABEL_SIZE).color(TEXT_FAINT));
}
