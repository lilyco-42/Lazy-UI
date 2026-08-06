//! M3 Switch. Returns the new checked state.
//! Styling from `assets/components/switch.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, SwitchConfig};
use crate::theme;

fn cfg() -> SwitchConfig {
    config::effective(config::Style::current().switch, SwitchConfig::get(), SwitchConfig::merged)
}

pub fn switch(ui: &mut Ui<'_, ()>, id: impl Into<Id>, checked: bool, label: &str) -> bool {
    let c = cfg();
    let theme = theme::theme();
    let id: Id = id.into();
    let gap = c.gap.unwrap_or(8.0) as u16;
    let pad_x = c.pad_x.unwrap_or(16.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.body_size);
    let width = c.width.unwrap_or(52.0);
    let height = c.height.unwrap_or(32.0);
    let handle = c.handle_size.unwrap_or(24.0);
    let on_color = c.on_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let on_handle = c.on_handle.map(Color::from).unwrap_or(theme.colors.on_primary.into());
    let off_track = c.off_track.map(Color::from).unwrap_or(theme.colors.surface_container_highest.into());
    let off_border = c.off_border.map(Color::from).unwrap_or(theme.colors.outline.into());
    let off_handle = c.off_handle.map(Color::from).unwrap_or(theme.colors.outline.into());

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
                        .width(fixed!(width))
                        .height(fixed!(height))
                        .corner_radius(height * 0.5)
                        .background_color(if checked { on_color } else { off_track })
                        .border(|b| {
                            b.all(2).color(if checked { on_color } else { off_border })
                        })
                        .children(|ui| {
                            ui.element()
                                .width(fixed!(handle))
                                .height(fixed!(handle))
                                .corner_radius(handle * 0.5)
                                .background_color(if checked { on_handle } else { off_handle })
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
                    ui.text(label, |t| t.font_size(font_size).color(theme.colors.on_surface));
                });
        });

    ui.is_just_pressed(id) ^ checked
}
