use eframe::egui::{
    CentralPanel, Color32, Context, Frame, Key, Margin, Modifiers, SidePanel, Stroke, Ui,
};

use crate::model::{Intent, Page};
use crate::ui::atoms::{ambient_center, paint_glow};
use crate::ui::organisms::{
    drop_overlay, hero_card, inspector, library_section, settings_page, sidebar, toast_stack,
    top_bar,
};
use crate::ui::theme::palette::{ACCENT_END, ACCENT_START, BACKGROUND, BORDER, SIDEBAR};
use crate::ui::theme::style::{HAIRLINE, PAGE_MARGIN};
use crate::ui::view::StudioView;

const SIDEBAR_WIDTH: f32 = 252.0;
const INSPECTOR_WIDTH: f32 = 360.0;
const AMBIENT_RADIUS: f32 = 0.55;
const AMBIENT_OPACITY: f32 = 0.10;

fn panel_frame(fill: Color32) -> Frame {
    Frame::none()
        .fill(fill)
        .stroke(Stroke::new(HAIRLINE, BORDER))
        .inner_margin(Margin::same(PAGE_MARGIN))
}

pub fn studio_shell(ctx: &Context, view: &StudioView<'_>) -> Vec<Intent> {
    let mut intents = shortcuts(ctx, view);
    SidePanel::left("sidebar")
        .exact_width(SIDEBAR_WIDTH)
        .resizable(false)
        .frame(panel_frame(SIDEBAR))
        .show(ctx, |ui| intents.extend(sidebar(ui, view)));
    if view.page == Page::Library {
        if let Some(wallpaper) = view.selected_wallpaper() {
            SidePanel::right("inspector")
                .exact_width(INSPECTOR_WIDTH)
                .resizable(false)
                .frame(panel_frame(SIDEBAR))
                .show(ctx, |ui| intents.extend(inspector(ui, view, wallpaper)));
        }
    }
    CentralPanel::default()
        .frame(panel_frame(BACKGROUND).stroke(Stroke::NONE))
        .show(ctx, |ui| {
            paint_ambient(ui);
            intents.extend(top_bar(ui, view));
            ui.add_space(PAGE_MARGIN);
            match view.page {
                Page::Library => {
                    hero_card(ui, view);
                    ui.add_space(PAGE_MARGIN);
                    intents.extend(library_section(ui, view));
                }
                Page::Settings => intents.extend(settings_page(ui, view)),
            }
        });
    toast_stack(ctx, view.toasts);
    if view.is_hovering_files {
        drop_overlay(ctx, view.t);
    }
    intents
}

fn paint_ambient(ui: &Ui) {
    let area = ui.max_rect();
    let painter = ui.painter();
    let radius = area.width().max(area.height()) * AMBIENT_RADIUS;
    paint_glow(
        painter,
        ambient_center(area, 0.9, 0.0),
        radius,
        ACCENT_END.gamma_multiply(AMBIENT_OPACITY),
    );
    paint_glow(
        painter,
        ambient_center(area, 0.1, 1.0),
        radius,
        ACCENT_START.gamma_multiply(AMBIENT_OPACITY),
    );
}

fn shortcuts(ctx: &Context, view: &StudioView<'_>) -> Vec<Intent> {
    let mut intents = Vec::new();
    if ctx.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::O)) {
        intents.push(Intent::PickFiles);
    }
    if ctx.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
        intents.push(match view.page {
            Page::Settings => Intent::Navigate(Page::Library),
            Page::Library => Intent::CloseInspector,
        });
    }
    intents
}
