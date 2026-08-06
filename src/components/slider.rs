//! M3 Slider. Returns the dragged value.
//! Styling from `assets/components/slider.toml`; unset fields use the theme.

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
    let frac = if max > min {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
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
                    ui.text(label, |t| t.font_size(font_size).color(label_color));
                    ui.element()
                        .width(grow!())
                        .height(fixed!(track_height))
                        .layout(|l| l.align(CenterX, CenterY))
                        .children(|ui| {
                            ui.element()
                                .width(grow!())
                                .height(fixed!(track_height))
                                .background_color(track_color)
                                .corner_radius(radius)
                                .empty();
                            ui.element()
                                .width(ply_engine::layout::Sizing::Percent(frac))
                                .height(fixed!(track_height))
                                .background_color(fill_color)
                                .corner_radius(radius)
                                .children(|ui| {
                                    ui.element()
                                        .width(fixed!(handle_size))
                                        .height(fixed!(handle_size))
                                        .corner_radius(handle_size * 0.5)
                                        .background_color(handle_color)
                                        .border(|b| b.all(3).color(handle_border))
                                        .floating(|f| {
                                            f.attach_parent()
                                                .anchor((CenterX, CenterY), (Right, CenterY))
                                                .offset((handle_size * 0.5, 0.0))
                                        })
                                        .empty();
                                });
                        });
                });
        });

    if let Some(b) = ui.bounding_box(id.clone()) {
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, _) = mouse_position();
            if mx >= b.x - 8.0 && mx <= b.x + b.width + 8.0 {
                let x = (mx - b.x).clamp(0.0, b.width);
                result = min + (x / b.width) * (max - min);
            }
        }
    }
    result
}
