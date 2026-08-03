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
//! Each container reads its own `<name>.toml` (see [`super::config`]) so layout
//! (flex direction, gap, position) is inferred, not configured.

use ply_engine::prelude::*;

use crate::theme;
use crate::components::config::{LogProgressConfig, PanelConfig, SidebarConfig, StatusBarConfig};

/// Left navigation rail. Renders `inner` as a vertical flex column.
pub fn sidebar(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = SidebarConfig::get();
    let theme = theme::theme();
    ui.element()
        .width(fixed!(cfg.width))
        .height(grow!())
        .background_color(theme.colors.surface_container_low)
        .border(|b| b.right(1).color(theme.colors.outline_variant))
        .layout(|l| l.direction(TopToBottom).gap(cfg.gap as u16).padding(cfg.padding as u16).align(Left, Top))
        .overflow(|o| { o.scroll_y() })
        .children(inner);
}

/// Main content card / panel.
pub fn panel(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = PanelConfig::get();
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(theme.colors.surface_container_lowest)
        .corner_radius(theme.shapes.radius_lg)
        .layout(|l| l.direction(TopToBottom).gap(cfg.gap as u16).padding(cfg.padding as u16))
        .overflow(|o| { o.scroll_y() })
        .children(inner);
}

/// Bottom status bar (full width, slim).
pub fn status_bar(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = StatusBarConfig::get();
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(fixed!(cfg.height))
        .background_color(theme.colors.surface_container)
        .border(|b| b.top(1).color(theme.colors.outline_variant))
        .layout(|l| l.direction(LeftToRight).gap(cfg.gap as u16).padding(cfg.padding as u16).align(Left, CenterY))
        .children(inner);
}

/// Bottom log-progress bar (nvim-dialog style): thin progress track pinned at
/// the bottom of a fixed-height bar. `value` in 0.0..=1.0.
pub fn log_progress(ui: &mut Ui<'_, ()>, id: impl Into<Id>, value: f32) {
    let cfg = LogProgressConfig::get();
    let theme = theme::theme();
    let frac = value.clamp(0.0, 1.0);
    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(cfg.height))
        .background_color(theme.colors.surface_container)
        .border(|b| b.top(1).color(theme.colors.outline_variant))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(fixed!(cfg.height * cfg.track_ratio))
                .layout(|l| l.align(CenterX, CenterY).padding((cfg.padding as u16, 0, cfg.padding as u16, 0)))
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
