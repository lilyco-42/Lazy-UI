//! M3 Checkbox. Returns the new checked state (caller stores it).
//! Styling from `assets/components/checkbox.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, CheckboxConfig};
use crate::theme;

fn cfg() -> CheckboxConfig {
    config::effective(config::Style::current().checkbox, CheckboxConfig::get(), CheckboxConfig::merged)
}

pub fn checkbox(ui: &mut Ui<'_, ()>, id: impl Into<Id>, checked: bool, label: &str) -> bool {
    let c = cfg();
    let theme = theme::theme();
    let id: Id = id.into();
    let box_size = c.box_size.unwrap_or(18.0);
    let gap = c.gap.unwrap_or(8.0) as u16;
    let pad_x = c.pad_x.unwrap_or(16.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.body_size);
    let checked_color = c.checked_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let check_color = c.check_color.map(Color::from).unwrap_or(theme.colors.on_primary.into());
    let border_color = c.border_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into());

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
                .layout(|l| {
                    l.direction(LeftToRight).gap(gap).padding((0, pad_x, 0, pad_x)).align(Left, CenterY)
                })
                .children(|ui| {
                    if checked {
                        let box_bg: Color = if ui.pressed() {
                            theme::PRESSED_PRIMARY_CONTAINER.into()
                        } else if ui.hovered() {
                            theme::HOVER_PRIMARY_CONTAINER.into()
                        } else {
                            checked_color
                        };
                        ui.element()
                            .width(fixed!(box_size))
                            .height(fixed!(box_size))
                            .background_color(box_bg)
                            .corner_radius(c.radius.unwrap_or(2.0))
                            .layout(|l| l.align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text("✓", |t| t.font_size(14).color(check_color));
                            });
                    } else {
                        ui.element()
                            .width(fixed!(box_size))
                            .height(fixed!(box_size))
                            .border(|b| b.all(2).color(border_color))
                            .corner_radius(c.radius.unwrap_or(2.0))
                            .empty();
                    }
                    ui.text(label, |t| t.font_size(font_size).color(theme.colors.on_surface));
                });
        });

    ui.is_just_pressed(id) ^ checked
}
