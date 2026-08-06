//! M3 dropdown (ComboBox). Returns the newly selected index.
//! Styling from `assets/components/combo.toml`; unset fields use the theme.

use ply_engine::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::components::config::{self, ComboConfig};
use crate::theme;

thread_local! {
    static COMBO_OPEN: RefCell<HashMap<u32, bool>> = RefCell::new(HashMap::new());
}

fn cfg() -> ComboConfig {
    config::effective(config::Style::current().combo, ComboConfig::get(), ComboConfig::merged)
}

pub fn combo(ui: &mut Ui<'_, ()>, id: &'static str, options: &[&str], selected: usize) -> usize {
    let c = cfg();
    let theme = theme::theme();
    let root = Id::new(id);
    let root_key = root.id;
    let mut result = selected;

    let height = c.height.unwrap_or(theme.shapes.field_height);
    let radius = c.radius.unwrap_or(theme.shapes.radius_xs);
    let gap = c.gap.unwrap_or(8.0) as u16;
    let pad_x = c.pad_x.unwrap_or(16.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.body_size);
    let item_height = c.item_height.unwrap_or(theme.shapes.item_height);
    let bg = c.background.map(Color::from).unwrap_or(theme.colors.surface_variant.into());
    let text_color = c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface.into());
    let arrow_color = c.arrow_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into());
    let menu_bg = c.menu_bg.map(Color::from).unwrap_or(theme.colors.surface_container_high.into());
    let menu_radius = c.menu_radius.unwrap_or(theme.shapes.radius_sm);
    let menu_border = c.menu_border.map(Color::from).unwrap_or(theme.colors.outline_variant.into());
    let selected_bg = c.selected_bg.map(Color::from).unwrap_or(theme.colors.secondary_container.into());
    let selected_fg = c.selected_fg.map(Color::from).unwrap_or(theme.colors.on_secondary_container.into());

    let mut open = COMBO_OPEN.with(|m| m.borrow().get(&root_key).copied().unwrap_or(false));
    if ui.is_just_pressed(root.clone()) {
        open = !open;
    }
    COMBO_OPEN.with(|m| m.borrow_mut().insert(root_key, open));

    ui.element()
        .id(root.clone())
        .width(grow!())
        .height(fixed!(height))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(bg)
                .corner_radius(radius)
                .layout(|l| {
                    l.direction(LeftToRight).gap(gap).padding((0, pad_x, 0, pad_x)).align(Left, CenterY)
                })
                .children(|ui| {
                    ui.text(
                        options.get(selected).copied().unwrap_or(""),
                        |t| t.font_size(font_size).color(text_color),
                    );
                    ui.text("▾", |t| t.font_size(16).color(arrow_color));
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
                    .background_color(menu_bg)
                    .corner_radius(menu_radius)
                    .border(|b| b.all(1).color(menu_border))
                    .children(|ui| {
                        for (i, option) in options.iter().enumerate() {
                            let oid = Id::from((id, i as u32));
                            ui.element()
                                .id(oid.clone())
                                .width(grow!())
                                .height(fixed!(item_height))
                                .background_color(if i == selected { selected_bg } else { menu_bg })
                                .on_press(|_, _| {})
                                .children(|ui| {
                                    ui.element()
                                        .width(grow!())
                                        .height(grow!())
                                        .layout(|l| l.padding((0, pad_x, 0, pad_x)).align(Left, CenterY))
                                        .children(|ui| {
                                            ui.text(
                                                option,
                                                |t| t.font_size(font_size).color(if i == selected {
                                                    selected_fg
                                                } else {
                                                    text_color
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
