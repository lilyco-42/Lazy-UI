//! M3 Slider. Returns the dragged value.

use ply_engine::prelude::*;

use crate::theme;

pub fn slider(ui: &mut Ui<'_, ()>, id: impl Into<Id>, label: &str, value: f32, min: f32, max: f32) -> f32 {
    let theme = theme::theme();
    let id: Id = id.into();
    let frac = if max > min {
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut result = value;

    ui.element()
        .id(id.clone())
        .width(grow!())
        .height(fixed!(theme.shapes.touch_target))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.direction(TopToBottom).gap(4).padding((0, 16, 0, 16)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(label, |t| t.font_size(theme.text.label_size).color(theme.colors.on_surface_variant));
                    ui.element()
                        .width(grow!())
                        .height(fixed!(theme.shapes.track_height))
                        .layout(|l| l.align(CenterX, CenterY))
                        .children(|ui| {
                            ui.element()
                                .width(grow!())
                                .height(fixed!(theme.shapes.track_height))
                                .background_color(theme.colors.surface_container_highest)
                                .corner_radius(theme.shapes.radius_sm)
                                .empty();
                            ui.element()
                                .width(ply_engine::layout::Sizing::Percent(frac))
                                .height(fixed!(theme.shapes.track_height))
                                .background_color(theme.colors.primary)
                                .corner_radius(theme.shapes.radius_sm)
                                .children(|ui| {
                                    ui.element()
                                        .width(fixed!(theme.shapes.handle_size))
                                        .height(fixed!(theme.shapes.handle_size))
                                        .corner_radius(theme.shapes.handle_size * 0.5)
                                        .background_color(theme.colors.primary)
                                        .border(|b| b.all(3).color(theme.colors.on_primary))
                                        .floating(|f| {
                                            f.attach_parent()
                                                .anchor((CenterX, CenterY), (Right, CenterY))
                                                .offset((theme.shapes.handle_size * 0.5, 0.0))
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
