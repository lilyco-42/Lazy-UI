//! Buttons — filled / tonal / outlined / text (M3).

use ply_engine::prelude::*;

use crate::theme::{self, TRANSPARENT};

fn rounded_btn(
    ui: &mut Ui<'_, ()>,
    label: &str,
    mut on_click: impl FnMut() + 'static,
    bg: impl Into<Color>,
    bg_hover: impl Into<Color>,
    bg_pressed: impl Into<Color>,
    fg: impl Into<Color>,
    border_color: Option<u32>,
) {
    let theme = theme::theme();
    let radius = theme.shapes.button_height * 0.5;

    ui.element()
        .width(fit!())
        .height(fixed!(theme.shapes.button_height))
        .corner_radius(radius)
        .on_press(move |_, _| on_click())
        .accessibility(|a| a.button(label))
        .children(|ui| {
            let state: Color = if ui.pressed() {
                bg_pressed.into()
            } else if ui.hovered() {
                bg_hover.into()
            } else {
                bg.into()
            };
            let mut el = ui
                .element()
                .width(grow!())
                .height(grow!())
                .background_color(state)
                .corner_radius(radius)
                .layout(|l| l.padding((0, 24, 0, 24)).align(CenterX, CenterY));
            if let Some(bc) = border_color {
                el = el.border(|b| b.all(1).color(bc));
            }
            el.children(|ui| {
                ui.text(label, |t| t.font_size(theme.text.label_size).color(fg.into()));
            });
        });
}

/// High-emphasis filled button. `button(ui, "Save", || save())`
pub fn button(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let theme = theme::theme();
    rounded_btn(
        ui,
        label,
        on_click,
        theme.colors.primary,
        theme::HOVER_PRIMARY,
        theme::PRESSED_PRIMARY,
        theme.colors.on_primary,
        None,
    );
}

/// Medium-emphasis tonal button (secondary container).
pub fn button_tonal(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let theme = theme::theme();
    rounded_btn(
        ui,
        label,
        on_click,
        theme.colors.secondary_container,
        theme::HOVER_TONAL,
        theme::PRESSED_TONAL,
        theme.colors.on_secondary_container,
        None,
    );
}

/// Outlined button.
pub fn button_outlined(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let theme = theme::theme();
    rounded_btn(
        ui,
        label,
        on_click,
        TRANSPARENT,
        theme::HOVER_OUTLINED,
        theme::PRESSED_OUTLINED,
        theme.colors.primary,
        Some(theme.colors.outline),
    );
}

/// Low-emphasis text button.
pub fn button_text(ui: &mut Ui<'_, ()>, label: &str, on_click: impl FnMut() + 'static) {
    let theme = theme::theme();
    rounded_btn(
        ui,
        label,
        on_click,
        TRANSPARENT,
        theme::HOVER_TEXT,
        theme::PRESSED_TEXT,
        theme.colors.primary,
        None,
    );
}
