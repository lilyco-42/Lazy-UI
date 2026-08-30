//! 关于页 — collected from the plyx_demo M3 prototype.
//! Embeds `卡片容器`; layout auto-inferred from `关于页和卡片容器布局关系.toml`.

use ply_engine::prelude::*;

use crate::components::page_layout::{self, PageLayout};

/// Renders the 关于页 page.
pub fn render(ui: &mut Ui<'_, ()>) {
    let layout = PageLayout::from_toml(include_str!("../../assets/components/关于页和卡片容器布局关系.toml"));
    page_layout::render_page(ui, &layout, |ui, child| {
        match child.component.as_str() {
            "卡片容器" => {
                crate::components::卡片容器(ui, |_ui| {});
            }
            _ => {}
        }
    });
}