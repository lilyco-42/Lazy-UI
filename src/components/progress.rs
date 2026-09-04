//! M3 linear progress indicator.
//! Styling from `assets/components/progress.toml`; unset fields use the theme.
//!
//! Fill is a float anchored to the track's LEFT with fixed pixel width
//! (`frac × measured track width`) — same cure as the slider rewrite:
//! a Percent fill inside a centered container grew from the center/right
//! and looked backwards. Callers must pass a unique `id` per instance
//! (width is measured from the previous frame's box).

use ply_engine::prelude::*;

use crate::components::config::{self, ProgressConfig};
use crate::theme;

fn cfg() -> ProgressConfig {
    config::effective(config::Style::current().progress, ProgressConfig::get(), ProgressConfig::merged)
}

pub fn progress(ui: &mut Ui<'_, ()>, id: impl Into<Id>, fraction: f32) {
    let c = cfg();
    let theme = theme::theme();
    let frac = fraction.clamp(0.0, 1.0);
    let track_height = c.track_height.unwrap_or(theme.shapes.track_height);
    let radius = c.radius.unwrap_or(theme.shapes.radius_sm);
    let track_color = c.track_color.map(Color::from).unwrap_or(theme.colors.surface_container_highest.into());
    let fill_color = c.fill_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let id: Id = id.into();

    // Track width measured from the previous frame's box (slider 同款);
    // first frame measures 0 and the fill appears from frame 2.
    let track_w = ui
        .bounding_box(id.clone())
        .map(|b| b.width.max(0.0))
        .unwrap_or(0.0);

    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(track_height))
        .children(|ui| {
            // Track base: drawn in-flow first (under everything).
            ui.element()
                .width(grow!())
                .height(fixed!(track_height))
                .background_color(track_color)
                .corner_radius(radius)
                .empty();
            // Fill: float anchored LEFT, fixed px width.
            ui.element()
                .width(fixed!(frac * track_w))
                .height(fixed!(track_height))
                .background_color(fill_color)
                .corner_radius(radius)
                .floating(|f| {
                    f.attach_parent()
                        .passthrough()
                        .anchor((Left, CenterY), (Left, CenterY))
                })
                .empty();
        });
}
