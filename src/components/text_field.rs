//! M3 Text fields (filled / outlined). Read the value with `ui.get_text_value(id)`.

use ply_engine::prelude::*;

use crate::theme;

/// Filled text field. Value lives in Ply under `id`.
pub fn text_field(ui: &mut Ui<'_, ()>, id: &'static str, placeholder: &str) {
    let theme = theme::theme();
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(theme.shapes.field_height))
        .text_input(|x| {
            x.placeholder(placeholder)
                .font_size(theme.text.body_size)
                .text_color(theme.colors.on_surface)
                .placeholder_color(theme.colors.on_surface_variant)
                .cursor_color(theme.colors.primary)
                .selection_color(theme.colors.primary_container)
                .on_changed(|_| {})
        })
        .background_color(theme.colors.surface_variant)
        .corner_radius(theme.shapes.radius_xs)
        .empty();
}

/// Outlined text field.
pub fn text_field_outlined(ui: &mut Ui<'_, ()>, id: &'static str, placeholder: &str) {
    let theme = theme::theme();
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(theme.shapes.field_height))
        .text_input(|x| {
            x.placeholder(placeholder)
                .font_size(theme.text.body_size)
                .text_color(theme.colors.on_surface)
                .placeholder_color(theme.colors.on_surface_variant)
                .cursor_color(theme.colors.primary)
                .selection_color(theme.colors.primary_container)
                .on_changed(|_| {})
        })
        .background_color(theme.colors.surface)
        .border(|b| b.all(1).color(theme.colors.outline))
        .corner_radius(theme.shapes.radius_xs)
        .empty();
}
