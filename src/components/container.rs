//! M3 layout containers — the app skeleton from the spec:
//!
//! ```text
//! main() {
//!   sidebar({ 启动(), 设置(), 关于() })
//!   panel()
//!   status_bar()
//!   log_progress()   // nvim-dialog style, auto-inferred bottom 10%
//! }
//! ```
//!
//! When driven by [`super::layout::render`], the wrapper region supplies the
//! size; each container here FILLS that region (grow). Standalone, they fill
//! whatever parent they are placed in.

use ply_engine::prelude::*;

use crate::theme;
use crate::components::config::{LogProgressConfig, PanelConfig, SidebarConfig, StatusBarConfig};

/// Left navigation rail. Renders `inner` as a vertical flex column, filling
/// its layout region (e.g. the 240px-wide sidebar region in `app_layout.toml`).
pub fn sidebar(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = SidebarConfig::get();
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(theme.colors.surface_container_low)
        .border(|b| b.right(1).color(theme.colors.outline_variant))
        .layout(|l| l.direction(TopToBottom).gap(cfg.gap as u16).padding(cfg.padding as u16).align(Left, Top))
        .overflow(|o| o.scroll_y())
        .children(inner);
}

/// Main content card / panel, filling its layout region.
pub fn panel(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = PanelConfig::get();
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(theme.colors.surface_container_lowest)
        .corner_radius(theme.shapes.radius_lg)
        .layout(|l| l.direction(TopToBottom).gap(cfg.gap as u16).padding(cfg.padding as u16))
        .overflow(|o| o.scroll_y())
        .children(inner);
}

/// Bottom status bar (full width, slim), filling its layout region.
pub fn status_bar(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = StatusBarConfig::get();
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(theme.colors.surface_container)
        .border(|b| b.top(1).color(theme.colors.outline_variant))
        .layout(|l| l.direction(LeftToRight).gap(cfg.gap as u16).padding(cfg.padding as u16).align(Left, CenterY))
        .children(inner);
}

/// Bottom log-progress bar (nvim-dialog style): a thin progress track pinned at
/// the bottom of a filled layout region. `value` in 0.0..=1.0.
pub fn log_progress(ui: &mut Ui<'_, ()>, id: impl Into<Id>, value: f32) {
    let cfg = LogProgressConfig::get();
    let theme = theme::theme();
    let frac = value.clamp(0.0, 1.0);
    ui.element()
        .id(id)
        .width(grow!())
        .height(grow!())
        .background_color(theme.colors.surface_container)
        .border(|b| b.top(1).color(theme.colors.outline_variant))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(fixed!(cfg.track_height))
                .layout(|l| l.padding((cfg.padding as u16, 0, cfg.padding as u16, 0)))
                .children(|ui| {
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .background_color(theme.colors.surface_container_highest)
                        .corner_radius(theme.shapes.radius_sm)
                        .empty();
                    ui.element()
                        .width(ply_engine::layout::Sizing::Percent(frac))
                        .height(grow!())
                        .background_color(theme.colors.primary)
                        .corner_radius(theme.shapes.radius_sm)
                        .empty();
                });
        });
}
