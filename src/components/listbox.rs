//! M3 ListBox (scrollable list). Returns the newly selected index.

use ply_engine::prelude::*;

use crate::theme;
use crate::components::selectable;

pub fn listbox(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    options: &[&str],
    selected: usize,
    visible: usize,
) -> usize {
    let theme = theme::theme();
    let mut result = selected;
    let height = (visible.max(1) as f32) * theme.shapes.item_height;

    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(fixed!(height))
        .border(|b| b.all(1).color(theme.colors.outline_variant))
        .corner_radius(theme.shapes.radius_sm)
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
