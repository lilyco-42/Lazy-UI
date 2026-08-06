//! M3 ListBox (scrollable list). Returns the newly selected index.
//! Styling from `assets/components/listbox.toml`; unset fields use the theme.

use ply_engine::prelude::*;

use crate::components::config::{self, ListboxConfig};
use crate::components::selectable;
use crate::theme;

fn cfg() -> ListboxConfig {
    config::effective(config::Style::current().listbox, ListboxConfig::get(), ListboxConfig::merged)
}

pub fn listbox(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    options: &[&str],
    selected: usize,
    visible: usize,
) -> usize {
    let c = cfg();
    let theme = theme::theme();
    let mut result = selected;

    let item_height = c.item_height.unwrap_or(theme.shapes.item_height);
    let height = (visible.max(1) as f32) * item_height;

    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(fixed!(height))
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_sm))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            for (i, option) in options.iter().enumerate() {
                let oid = Id::from((id, i as u32));
                selectable(ui, oid.clone(), i == selected, option);
                if ui.is_just_pressed(oid) {
                    result = i;
                }
            }
        });

    result
}
