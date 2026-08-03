//! M3 linear progress indicator.

use ply_engine::prelude::*;

use crate::theme;

pub fn progress(ui: &mut Ui<'_, ()>, fraction: f32) {
    let theme = theme::theme();
    let frac = fraction.clamp(0.0, 1.0);
    ui.element()
        .width(grow!())
        .height(fixed!(theme.shapes.track_height))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(theme.colors.surface_container_highest)
                .corner_radius(theme.shapes.radius_sm)
                .empty();
            ui.element()
                .width(ply_engine::layout::Sizing::Percent(frac))
                .height(grow!())
                .background_color(theme.colors.primary)
                .corner_radius(theme.shapes.radius_sm)
                .empty();
        });
}
