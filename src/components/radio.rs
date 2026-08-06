//! M3 Radio button and radio group.
//! Styling from `assets/components/radio.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, RadioConfig};
use crate::theme;

fn cfg() -> RadioConfig {
    config::effective(config::Style::current().radio, RadioConfig::get(), RadioConfig::merged)
}

/// Single radio row. Returns true if it was selected this frame.
pub fn radio(ui: &mut Ui<'_, ()>, id: impl Into<Id>, selected: bool, label: &str) -> bool {
    let c = cfg();
    let theme = theme::theme();
    let id: Id = id.into();
    let size = c.size.unwrap_or(20.0);
    let dot_size = c.dot_size.unwrap_or(10.0);
    let gap = c.gap.unwrap_or(8.0) as u16;
    let pad_x = c.pad_x.unwrap_or(16.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.body_size);
    let selected_color = c.selected_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let border_color = c.border_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into());

    ui.element()
        .id(id.clone())
        .width(fit!())
        .height(fixed!(theme.shapes.touch_target))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(fit!())
                .height(grow!())
                .layout(|l| {
                    l.direction(LeftToRight).gap(gap).padding((0, pad_x, 0, pad_x)).align(Left, CenterY)
                })
                .children(|ui| {
                    ui.element()
                        .width(fixed!(size))
                        .height(fixed!(size))
                        .border(|b| {
                            b.all(2).color(if selected { selected_color } else { border_color })
                        })
                        .corner_radius(size * 0.5)
                        .children(|ui| {
                            if selected {
                                ui.element()
                                    .width(fixed!(dot_size))
                                    .height(fixed!(dot_size))
                                    .background_color(selected_color)
                                    .corner_radius(dot_size * 0.5)
                                    .layout(|l| l.align(CenterX, CenterY))
                                    .empty();
                            }
                        });
                    ui.text(label, |t| t.font_size(font_size).color(theme.colors.on_surface));
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
