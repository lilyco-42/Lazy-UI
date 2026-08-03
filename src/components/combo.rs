//! M3 dropdown (ComboBox). Returns the newly selected index.

use ply_engine::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::theme;

thread_local! {
    static COMBO_OPEN: RefCell<HashMap<u32, bool>> = RefCell::new(HashMap::new());
}

pub fn combo(ui: &mut Ui<'_, ()>, id: &'static str, options: &[&str], selected: usize) -> usize {
    let theme = theme::theme();
    let root = Id::new(id);
    let root_key = root.id;
    let mut result = selected;

    let mut open = COMBO_OPEN.with(|m| m.borrow().get(&root_key).copied().unwrap_or(false));
    if ui.is_just_pressed(root.clone()) {
        open = !open;
    }
    COMBO_OPEN.with(|m| m.borrow_mut().insert(root_key, open));

    ui.element()
        .id(root.clone())
        .width(grow!())
        .height(fixed!(theme.shapes.field_height))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(theme.colors.surface_variant)
                .corner_radius(theme.shapes.radius_xs)
                .layout(|l| l.direction(LeftToRight).gap(8).padding((0, 16, 0, 16)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(
                        options.get(selected).copied().unwrap_or(""),
                        |t| t.font_size(theme.text.body_size).color(theme.colors.on_surface),
                    );
                    ui.text("▾", |t| t.font_size(16).color(theme.colors.on_surface_variant));
                });

            if open {
                ui.element()
                    .width(grow!())
                    .height(fit!())
                    .floating(|f| {
                        f.attach_parent()
                            .anchor((Left, Top), (Left, Bottom))
                            .offset((0.0, 4.0))
                            .z_index(100)
                    })
                    .background_color(theme.colors.surface_container_high)
                    .corner_radius(theme.shapes.radius_sm)
                    .border(|b| b.all(1).color(theme.colors.outline_variant))
                    .children(|ui| {
                        for (i, option) in options.iter().enumerate() {
                            let oid = Id::from((id, i as u32));
                            ui.element()
                                .id(oid.clone())
                                .width(grow!())
                                .height(fixed!(theme.shapes.item_height))
                                .background_color(if i == selected {
                                    theme.colors.secondary_container
                                } else {
                                    theme.colors.surface_container_high
                                })
                                .on_press(|_, _| {})
                                .children(|ui| {
                                    ui.element()
                                        .width(grow!())
                                        .height(grow!())
                                        .layout(|l| l.padding((0, 16, 0, 16)).align(Left, CenterY))
                                        .children(|ui| {
                                            ui.text(
                                                option,
                                                |t| t.font_size(theme.text.body_size).color(if i == selected {
                                                    theme.colors.on_secondary_container
                                                } else {
                                                    theme.colors.on_surface
                                                }),
                                            );
                                        });
                                });
                            if ui.is_just_pressed(oid) {
                                result = i;
                                COMBO_OPEN.with(|m| m.borrow_mut().insert(root_key, false));
                            }
                        }
                    });
            }
        });

    result
}
