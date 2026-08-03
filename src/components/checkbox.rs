//! M3 Checkbox. Returns the new checked state (caller stores it).

use ply_engine::prelude::*;

use crate::theme;

pub fn checkbox(ui: &mut Ui<'_, ()>, id: impl Into<Id>, checked: bool, label: &str) -> bool {
    let theme = theme::theme();
    let id: Id = id.into();

    ui.element()
        .id(id.clone())
        .width(fit!())
        .height(fixed!(theme.shapes.touch_target))
        .on_press(|_, _| {})
        .accessibility(|a| a.checkbox(label))
        .children(|ui| {
            ui.element()
                .width(fit!())
                .height(grow!())
                .layout(|l| l.direction(LeftToRight).gap(8).padding((0, 16, 0, 16)).align(Left, CenterY))
                .children(|ui| {
                    if checked {
                        let box_bg: Color = if ui.pressed() {
                            theme::PRESSED_PRIMARY_CONTAINER.into()
                        } else if ui.hovered() {
                            theme::HOVER_PRIMARY_CONTAINER.into()
                        } else {
                            theme.colors.primary.into()
                        };
                        ui.element()
                            .width(fixed!(18.0))
                            .height(fixed!(18.0))
                            .background_color(box_bg)
                            .corner_radius(2.0)
                            .layout(|l| l.align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text("✓", |t| t.font_size(14).color(theme.colors.on_primary));
                            });
                    } else {
                        ui.element()
                            .width(fixed!(18.0))
                            .height(fixed!(18.0))
                            .border(|b| b.all(2).color(theme.colors.on_surface_variant))
                            .corner_radius(2.0)
                            .empty();
                    }
                    ui.text(label, |t| t.font_size(theme.text.body_size).color(theme.colors.on_surface));
                });
        });

    ui.is_just_pressed(id) ^ checked
}
