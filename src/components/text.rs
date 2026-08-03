//! Typographic helpers: headline / title / body / label text.

use ply_engine::prelude::*;

use crate::theme;

/// Headline (28) — page titles.
pub fn headline(ui: &mut Ui<'_, ()>, text: &str) {
    let theme = theme::theme();
    ui.text(text, |c| c.font_size(theme.text.headline_size).color(theme.colors.on_surface));
}

/// Title (22) — section titles.
pub fn title(ui: &mut Ui<'_, ()>, text: &str) {
    let theme = theme::theme();
    ui.text(text, |c| c.font_size(theme.text.title_size).color(theme.colors.on_surface));
}

/// Body (16) — default content.
pub fn body(ui: &mut Ui<'_, ()>, text: &str) {
    let theme = theme::theme();
    ui.text(text, |c| c.font_size(theme.text.body_size).color(theme.colors.on_surface));
}

/// Label (14, muted) — captions and annotations.
pub fn label(ui: &mut Ui<'_, ()>, text: &str) {
    let theme = theme::theme();
    ui.text(text, |c| c.font_size(theme.text.label_size).color(theme.colors.on_surface_variant));
}
