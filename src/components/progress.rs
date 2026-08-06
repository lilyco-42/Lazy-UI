//! M3 linear progress indicator.
//! Styling from `assets/components/progress.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, ProgressConfig};
use crate::theme;

fn cfg() -> ProgressConfig {
    config::effective(config::Style::current().progress, ProgressConfig::get(), ProgressConfig::merged)
}

pub fn progress(ui: &mut Ui<'_, ()>, fraction: f32) {
    let c = cfg();
    let theme = theme::theme();
    let frac = fraction.clamp(0.0, 1.0);
    let track_height = c.track_height.unwrap_or(theme.shapes.track_height);
    let radius = c.radius.unwrap_or(theme.shapes.radius_sm);
    let track_color = c.track_color.map(Color::from).unwrap_or(theme.colors.surface_container_highest.into());
    let fill_color = c.fill_color.map(Color::from).unwrap_or(theme.colors.primary.into());

    ui.element()
        .width(grow!())
        .height(fixed!(track_height))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(track_color)
                .corner_radius(radius)
                .empty();
            ui.element()
                .width(ply_engine::layout::Sizing::Percent(frac))
                .height(grow!())
                .background_color(fill_color)
                .corner_radius(radius)
                .empty();
        });
}
