//! Horizontal divider.
//! Styling from `assets/components/divider.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, DividerConfig};
use crate::theme;

fn cfg() -> DividerConfig {
    config::effective(config::Style::current().divider, DividerConfig::get(), DividerConfig::merged)
}

pub fn divider(ui: &mut Ui<'_, ()>) {
    let c = cfg();
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(fixed!(c.thickness.unwrap_or(1.0)))
        .background_color(c.color.map(Color::from).unwrap_or(theme.colors.outline_variant.into()))
        .empty();
}
