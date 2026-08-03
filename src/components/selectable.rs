//! M3 Selectable list row. Returns true if activated this frame.

use ply_engine::prelude::*;

use crate::theme::{self, TRANSPARENT};

pub fn selectable(ui: &mut Ui<'_, ()>, id: impl Into<Id>, selected: bool, label: &str) -> bool {
    let theme = theme::theme();
    let id: Id = id.into();

    ui.element()
        .id(id.clone())
        .width(grow!())
        .height(fixed!(theme.shapes.item_height))
        .background_color(if selected {
            Color::from(theme.colors.secondary_container)
        } else {
            Color::from(TRANSPARENT)
        })
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.padding((0, 16, 0, 16)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(
                        label,
                        |t| t.font_size(theme.text.body_size).color(if selected {
                            theme.colors.on_secondary_container
                        } else {
                            theme.colors.on_surface
                        }),
                    );
                });
        });

    ui.is_just_pressed(id)
}
