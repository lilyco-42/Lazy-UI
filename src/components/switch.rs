//! M3 Switch. Returns the new checked state.

use ply_engine::prelude::*;

use crate::theme;

pub fn switch(ui: &mut Ui<'_, ()>, id: impl Into<Id>, checked: bool, label: &str) -> bool {
    let theme = theme::theme();
    let id: Id = id.into();

    ui.element()
        .id(id.clone())
        .width(fit!())
        .height(fixed!(theme.shapes.touch_target))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(fit!())
                .height(grow!())
                .layout(|l| l.direction(LeftToRight).gap(8).padding((0, 16, 0, 16)).align(Left, CenterY))
                .children(|ui| {
                    ui.element()
                        .width(fixed!(52.0))
                        .height(fixed!(32.0))
                        .corner_radius(16.0)
                        .background_color(if checked { theme.colors.primary } else { theme.colors.surface_container_highest })
                        .border(|b| b.all(2).color(if checked { theme.colors.primary } else { theme.colors.outline }))
                        .children(|ui| {
                            ui.element()
                                .width(fixed!(24.0))
                                .height(fixed!(24.0))
                                .corner_radius(12.0)
                                .background_color(if checked { theme.colors.on_primary } else { theme.colors.outline })
                                .floating(|f| {
                                    f.attach_parent()
                                        .anchor(
                                            if checked { (Right, CenterY) } else { (Left, CenterY) },
                                            (Left, CenterY),
                                        )
                                        .offset(if checked { (-4.0, 0.0) } else { (4.0, 0.0) })
                                })
                                .empty();
                        });
                    ui.text(label, |t| t.font_size(theme.text.body_size).color(theme.colors.on_surface));
                });
        });

    ui.is_just_pressed(id) ^ checked
}
