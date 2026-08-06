//! M3 Selectable list row. Returns true if activated this frame.
//! Styling from `assets/components/selectable.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, SelectableConfig};
use crate::theme::{self, TRANSPARENT};

fn cfg() -> SelectableConfig {
    config::effective(config::Style::current().selectable, SelectableConfig::get(), SelectableConfig::merged)
}

pub fn selectable(ui: &mut Ui<'_, ()>, id: impl Into<Id>, selected: bool, label: &str) -> bool {
    let c = cfg();
    let theme = theme::theme();
    let id: Id = id.into();
    let height = c.height.unwrap_or(theme.shapes.item_height);
    let pad_x = c.pad_x.unwrap_or(16.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.body_size);
    let selected_bg = c.selected_bg.map(Color::from).unwrap_or(theme.colors.secondary_container.into());
    let selected_fg = c.selected_fg.map(Color::from).unwrap_or(theme.colors.on_secondary_container.into());
    let text_color = c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface.into());

    ui.element()
        .id(id.clone())
        .width(grow!())
        .height(fixed!(height))
        .background_color(if selected { selected_bg } else { TRANSPARENT.into() })
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.padding((0, pad_x, 0, pad_x)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(
                        label,
                        |t| t.font_size(font_size).color(if selected { selected_fg } else { text_color }),
                    );
                });
        });

    ui.is_just_pressed(id)
}
