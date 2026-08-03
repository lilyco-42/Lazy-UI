//! M3 Tooltip: wraps arbitrary content and shows a label on hover.

use ply_engine::prelude::*;

use crate::theme;

pub fn tooltip(ui: &mut Ui<'_, ()>, id: &'static str, text: &str, inner: impl FnOnce(&mut Ui<'_, ()>)) {
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
                            .offset((0.0, -4.0))
                            .z_index(200)
                    })
                    .background_color(theme.colors.inverse_surface)
                    .corner_radius(theme.shapes.radius_xs)
                    .layout(|l| l.padding((0, 8, 0, 8)))
                    .children(|ui| {
                        ui.text(
                            text,
                            |t| t.font_size(theme.text.body_size).color(theme.colors.inverse_on_surface),
                        );
                    });
            }
        });
}
