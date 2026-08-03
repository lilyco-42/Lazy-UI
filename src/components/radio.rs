//! M3 Radio button and radio group.

use ply_engine::prelude::*;

use crate::theme;

/// Single radio row. Returns true if it was selected this frame.
pub fn radio(ui: &mut Ui<'_, ()>, id: impl Into<Id>, selected: bool, label: &str) -> bool {
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
                        .width(fixed!(20.0))
                        .height(fixed!(20.0))
                        .border(|b| b.all(2).color(if selected { theme.colors.primary } else { theme.colors.on_surface_variant }))
                        .corner_radius(10.0)
                        .children(|ui| {
                            if selected {
                                ui.element()
                                    .width(fixed!(10.0))
                                    .height(fixed!(10.0))
                                    .background_color(theme.colors.primary)
                                    .corner_radius(5.0)
                                    .layout(|l| l.align(CenterX, CenterY))
                                    .empty();
                            }
                        });
                    ui.text(label, |t| t.font_size(theme.text.body_size).color(theme.colors.on_surface));
                });
        });

    ui.is_just_pressed(id)
}

/// Radio group. Returns the newly selected index.
pub fn radio_group(ui: &mut Ui<'_, ()>, id: &'static str, options: &[&str], selected: usize) -> usize {
    let mut result = selected;
    ui.element()
        .width(fit!())
        .height(fit!())
        .layout(|l| l.direction(TopToBottom).gap(4))
        .children(|ui| {
            for (i, option) in options.iter().enumerate() {
                let oid = Id::from((id, i as u32));
                radio(ui, oid.clone(), i == selected, option);
                if ui.is_just_pressed(oid) {
                    result = i;
                }
            }
        });
    result
}
