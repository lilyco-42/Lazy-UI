//! Typographic helpers: headline / title / body / label text.
//! Styling from `assets/components/text.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, TextConfig};
use crate::theme;

fn cfg() -> TextConfig {
    config::effective(config::Style::current().text, TextConfig::get(), TextConfig::merged)
}

/// Headline (28) — page titles.
pub fn headline(ui: &mut Ui<'_, ()>, text: &str) {
    let c = cfg();
    let theme = theme::theme();
    ui.text(text, |t| {
        t.font_size(c.headline_size.unwrap_or(theme.text.headline_size))
            .color(c.headline_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
    });
}

/// Title (22) — section titles.
pub fn title(ui: &mut Ui<'_, ()>, text: &str) {
    let c = cfg();
    let theme = theme::theme();
    ui.text(text, |t| {
        t.font_size(c.title_size.unwrap_or(theme.text.title_size))
            .color(c.title_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
    });
}

/// Body (16) — default content.
pub fn body(ui: &mut Ui<'_, ()>, text: &str) {
    let c = cfg();
    let theme = theme::theme();
    ui.text(text, |t| {
        t.font_size(c.body_size.unwrap_or(theme.text.body_size))
            .color(c.body_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
    });
}

/// Label (14, muted) — captions and annotations.
pub fn label(ui: &mut Ui<'_, ()>, text: &str) {
    let c = cfg();
    let theme = theme::theme();
    ui.text(text, |t| {
        t.font_size(c.label_size.unwrap_or(theme.text.label_size))
            .color(c.label_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into()))
    });
}
