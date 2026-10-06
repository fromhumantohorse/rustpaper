use eframe::egui::{emath::Numeric, Slider, Ui};
use rustpaper_domain::Bounded;

pub fn bounded_slider<B>(ui: &mut Ui, value: &mut B, suffix: &str) -> bool
where
    B: Bounded,
    B::Raw: Numeric,
{
    let mut raw = value.get();
    let response = ui.add(Slider::new(&mut raw, B::MIN..=B::MAX).suffix(suffix));
    response.changed() && B::try_new(raw).map(|next| *value = next).is_ok()
}
