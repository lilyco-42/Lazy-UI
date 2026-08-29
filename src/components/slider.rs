//! M3 Slider. Returns the dragged value.
//! Styling from `assets/components/slider.toml`; unset fields use the theme.
//!
//! Drag model (from the plyx_demo rewrite):
//! - Label shows the live value ("音量: 60") instead of a static caption.
//! - Fill + handle are floats anchored to the track's LEFT and offset by
//!   `frac × measured track width`, so they grow from the left and stay
//!   exactly under the pointer (Percent fill inside `align(CenterX)` grew
//!   from the center — felt backwards).
//! - Only drag while the pointer is over THIS slider's row (x AND y).
//!   Checking x alone made every slider move when clicking anywhere.

use ply_engine::prelude::*;

use crate::components::config::{self, SliderConfig};
use crate::theme;

fn cfg() -> SliderConfig {
    config::effective(config::Style::current().slider, SliderConfig::get(), SliderConfig::merged)
}

pub fn slider(ui: &mut Ui<'_, ()>, id: impl Into<Id>, label: &str, value: f32, min: f32, max: f32) -> f32 {
    let c = cfg();
    let theme = theme::theme();
    let id: Id = id.into();
    let span = max - min;
    let frac = if span > 0.0 {
        ((value - min) / span).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut result = value;

    let height = c.height.unwrap_or(theme.shapes.touch_target);
    let track_height = c.track_height.unwrap_or(theme.shapes.track_height);
    let handle_size = c.handle_size.unwrap_or(theme.shapes.handle_size);
    let radius = c.radius.unwrap_or(theme.shapes.radius_sm);
    let gap = c.gap.unwrap_or(4.0) as u16;
    let pad_x = c.pad_x.unwrap_or(16.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.label_size);
    let track_color = c.track_color.map(Color::from).unwrap_or(theme.colors.surface_container_highest.into());
    let fill_color = c.fill_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let handle_color = c.handle_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let handle_border = c.handle_border.map(Color::from).unwrap_or(theme.colors.on_primary.into());
    let label_color = c.label_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into());

    // Track width = row bbox minus the container's horizontal padding. Measured
    // from the previous frame's box, so fill and knob line up with the drag.
    let track_w = ui
        .bounding_box(id.clone())
        .map(|b| (b.width - 2.0 * f32::from(pad_x)).max(0.0))
        .unwrap_or(0.0);
    let ox = frac * track_w;

    ui.element()
        .id(id.clone())
        .width(grow!())
        .height(fixed!(height))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| {
                    l.direction(TopToBottom).gap(gap).padding((0, pad_x, 0, pad_x)).align(Left, CenterY)
                })
                .children(|ui| {
                    // Label shows the current value, updated live while dragging.
                    let shown = if (result.fract()).abs() < 0.05 {
                        format!("{label}: {:.0}", result)
                    } else {
                        format!("{label}: {:.1}", result)
                    };
                    ui.text(&shown, |t| t.font_size(font_size).color(label_color));
                    ui.element()
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
                            // Fill + handle are floats anchored to the track's
                            // LEFT, both offset by `frac × track_w` (fixed px).
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
                            ui.element()
                                .width(fixed!(handle_size))
                                .height(fixed!(handle_size))
                                .corner_radius(handle_size * 0.5)
                                .background_color(handle_color)
                                .border(|b| b.all(3).color(handle_border))
                                .floating(|f| {
                                    f.attach_parent()
                                        .passthrough()
                                        .anchor((CenterX, CenterY), (Left, CenterY))
                                        .offset((ox, 0.0))
                                })
                                .empty();
                        });
                });
        });

    if let Some(b) = ui.bounding_box(id.clone()) {
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            // Respond only while the pointer is over THIS slider's box (x AND y).
            // Checking x alone made every slider move when clicking anywhere.
            let near_x = mx >= b.x - 4.0 && mx <= b.x + b.width + 4.0;
            let near_y = my >= b.y - 8.0 && my <= b.y + b.height + 8.0;
            if near_x && near_y && span > 0.0 {
                // Map across the TRACK region (bbox minus horizontal padding),
                // so the handle sits exactly under the pointer.
                let track_left = b.x + f32::from(pad_x);
                let track_right = b.x + b.width - f32::from(pad_x);
                if track_right > track_left {
                    let x = (mx - track_left).clamp(0.0, track_right - track_left);
                    result = min + (x / (track_right - track_left)) * span;
                }
            }
        }
    }
    result
}