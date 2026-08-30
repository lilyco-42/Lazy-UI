//! 主页面 — collected from the plyx_demo M3 prototype.
//! Embeds `蓝色按钮`; layout auto-inferred from `主页面和蓝色按钮布局关系.toml`.

use ply_engine::prelude::*;

use crate::components::page_layout::{self, PageLayout};

/// Renders the 主页面 page.
pub fn render(ui: &mut Ui<'_, ()>) {
    let layout = PageLayout::from_toml(include_str!("../../assets/components/主页面和蓝色按钮布局关系.toml"));
    page_layout::render_page(ui, &layout, |ui, child| {
        match child.component.as_str() {
            "蓝色按钮" => {
                crate::components::蓝色按钮(ui, "蓝色按钮");
            }
            _ => {}
        }
    });
}