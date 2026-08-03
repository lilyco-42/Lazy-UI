//! Horizontal divider.

use ply_engine::prelude::*;

use crate::theme;

pub fn divider(ui: &mut Ui<'_, ()>) {
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(fixed!(1.0))
        .background_color(theme.colors.outline_variant)
        .empty();
}
