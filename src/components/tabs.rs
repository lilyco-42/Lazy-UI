//! M3 Tabs. Returns the newly selected index.
//! Styling from `assets/components/tabs.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, TabsConfig};
use crate::theme;

fn cfg() -> TabsConfig {
    config::effective(config::Style::current().tabs, TabsConfig::get(), TabsConfig::merged)
}

pub fn tabs(ui: &mut Ui<'_, ()>, id: &'static str, items: &[&str], selected: usize) -> usize {
    let c = cfg();
    let theme = theme::theme();
    let mut result = selected;

    let height = c.height.unwrap_or(theme.shapes.tab_height);
    let font_size = c.font_size.unwrap_or(theme.text.label_size);
    let pad_x = c.pad_x.unwrap_or(16.0) as u16;
    let active_color = c.active_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let inactive_color = c.inactive_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into());
    let indicator_color = c.indicator_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let indicator_height = c.indicator_height.unwrap_or(3.0);

    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(fixed!(height))
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
                            .layout(|l| l.padding((0, pad_x, 0, pad_x)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(
                                    item,
                                    |t| t.font_size(font_size).color(if active { active_color } else { inactive_color }),
                                );
                            });
                        if active {
                            ui.element()
                                .width(grow!())
                                .height(fixed!(indicator_height))
                                .background_color(indicator_color)
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
