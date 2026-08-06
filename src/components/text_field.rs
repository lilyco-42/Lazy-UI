//! M3 Text fields (filled / outlined). Read the value with `ui.get_text_value(id)`.
//! Styling from `assets/components/text_field.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, TextFieldConfig};
use crate::theme;

fn cfg() -> TextFieldConfig {
    config::effective(config::Style::current().text_field, TextFieldConfig::get(), TextFieldConfig::merged)
}

fn field(ui: &mut Ui<'_, ()>, id: &'static str, placeholder: &str, outlined: bool) {
    let c = cfg();
    let theme = theme::theme();
    let mut el = ui
        .element()
        .id(id)
        .width(grow!())
        .height(fixed!(c.height.unwrap_or(theme.shapes.field_height)))
        .text_input(|x| {
            x.placeholder(placeholder)
                .font_size(c.font_size.unwrap_or(theme.text.body_size))
                .text_color(c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
                .placeholder_color(
                    c.placeholder_color
                        .map(Color::from)
                        .unwrap_or(theme.colors.on_surface_variant.into()),
                )
                .cursor_color(c.cursor_color.map(Color::from).unwrap_or(theme.colors.primary.into()))
                .selection_color(
                    c.selection_color
                        .map(Color::from)
                        .unwrap_or(theme.colors.primary_container.into()),
                )
                .on_changed(|_| {})
        })
        .background_color(c.background.map(Color::from).unwrap_or(if outlined {
            theme.colors.surface.into()
        } else {
            theme.colors.surface_variant.into()
        }))
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_xs));
    if outlined {
        el = el.border(|b| {
            b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline.into()))
        });
    }
    el.empty();
}

/// Filled text field. Value lives in Ply under `id`.
pub fn text_field(ui: &mut Ui<'_, ()>, id: &'static str, placeholder: &str) {
    field(ui, id, placeholder, false);
}

/// Outlined text field.
pub fn text_field_outlined(ui: &mut Ui<'_, ()>, id: &'static str, placeholder: &str) {
    field(ui, id, placeholder, true);
}
