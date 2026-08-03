//! M3 Tabs. Returns the newly selected index.

use ply_engine::prelude::*;

use crate::theme;

pub fn tabs(ui: &mut Ui<'_, ()>, id: &'static str, items: &[&str], selected: usize) -> usize {
    let theme = theme::theme();
    let mut result = selected;

    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(fixed!(theme.shapes.tab_height))
        .layout(|l| l.direction(LeftToRight).align(Left, Top))
        .children(|ui| {
            for (i, item) in items.iter().enumerate() {
                let iid = Id::from((id, i as u32));
                let active = i == selected;
                ui.element()
                    .id(iid.clone())
                    .width(fit!())
                    .height(grow!())
                    .on_press(|_, _| {})
                    .children(|ui| {
                        ui.element()
                            .width(grow!())
                            .height(grow!())
                            .layout(|l| l.padding((0, 16, 0, 16)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(
                                    item,
                                    |t| t.font_size(theme.text.label_size).color(if active {
                                        theme.colors.primary
                                    } else {
                                        theme.colors.on_surface_variant
                                    }),
                                );
                            });
                        if active {
                            ui.element()
                                .width(grow!())
                                .height(fixed!(3.0))
                                .background_color(theme.colors.primary)
                                .floating(|f| f.attach_parent().anchor((Left, Bottom), (Left, Bottom)))
                                .empty();
                        }
                    });
                if ui.is_just_pressed(iid) {
                    result = i;
                }
            }
        });

    result
}
