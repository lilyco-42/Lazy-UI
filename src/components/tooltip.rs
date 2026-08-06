//! M3 Tooltip: wraps arbitrary content and shows a label on hover.
//! Styling from `assets/components/tooltip.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, TooltipConfig};
use crate::theme;

fn cfg() -> TooltipConfig {
    config::effective(config::Style::current().tooltip, TooltipConfig::get(), TooltipConfig::merged)
}

pub fn tooltip(ui: &mut Ui<'_, ()>, id: &'static str, text: &str, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let c = cfg();
    let theme = theme::theme();
    ui.element()
        .id(id)
        .width(fit!())
        .height(fit!())
        .children(|ui| {
            inner(ui);
            if ui.hovered() {
                ui.element()
                    .width(fit!())
                    .height(fit!())
                    .floating(|f| {
                        f.attach_parent()
                            .anchor((CenterX, Bottom), (CenterX, Top))
                            .offset((0.0, -c.offset.unwrap_or(4.0)))
                            .z_index(200)
                    })
                    .background_color(
                        c.background.map(Color::from).unwrap_or(theme.colors.inverse_surface.into()),
                    )
                    .corner_radius(c.radius.unwrap_or(theme.shapes.radius_xs))
                    .layout(|l| l.padding((0, c.pad_x.unwrap_or(8.0) as u16, 0, c.pad_x.unwrap_or(8.0) as u16)))
                    .children(|ui| {
                        ui.text(
                            text,
                            |t| t.font_size(c.font_size.unwrap_or(theme.text.body_size)).color(
                                c.text_color
                                    .map(Color::from)
                                    .unwrap_or(theme.colors.inverse_on_surface.into()),
                            ),
                        );
                    });
            }
        });
}
