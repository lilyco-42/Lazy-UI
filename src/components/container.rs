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
//!
//! Styling comes from the same-named stylesheets
//! (`sidebar.toml` / `panel.toml` / `status_bar.toml` / `log_progress.toml`).

use ply_engine::prelude::*;

use crate::theme;
use crate::components::config::{self, LogProgressConfig, PanelConfig, SidebarConfig, StatusBarConfig};

/// Left navigation rail. Renders `inner` as a vertical flex column, filling
/// its layout region (e.g. the 240px-wide sidebar region in `app_layout.toml`).
pub fn sidebar(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = config::effective(config::Style::current().sidebar, SidebarConfig::get(), SidebarConfig::merged);
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(cfg.background.map(Color::from).unwrap_or(theme.colors.surface_container_low.into()))
        .border(|b| {
            b.right(1).color(cfg.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into()))
        })
        .layout(|l| {
            l.direction(TopToBottom)
                .gap(cfg.gap.unwrap_or(4.0) as u16)
                .padding(cfg.padding.unwrap_or(12.0) as u16)
                .align(Left, Top)
        })
        .overflow(|o| {
            if cfg.scroll.unwrap_or(true) {
                o.scroll_y()
            } else {
                o
            }
        })
        .children(inner);
}

/// Main content card / panel, filling its layout region.
pub fn panel(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = config::effective(config::Style::current().panel, PanelConfig::get(), PanelConfig::merged);
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(cfg.background.map(Color::from).unwrap_or(theme.colors.surface_container_lowest.into()))
        .corner_radius(cfg.radius.unwrap_or(theme.shapes.radius_lg))
        .layout(|l| {
            l.direction(TopToBottom)
                .gap(cfg.gap.unwrap_or(8.0) as u16)
                .padding(cfg.padding.unwrap_or(16.0) as u16)
        })
        .overflow(|o| {
            if cfg.scroll.unwrap_or(true) {
                o.scroll_y()
            } else {
                o
            }
        })
        .children(inner);
}

/// Bottom status bar (full width, slim), filling its layout region.
pub fn status_bar(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let cfg = config::effective(config::Style::current().status_bar, StatusBarConfig::get(), StatusBarConfig::merged);
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(cfg.background.map(Color::from).unwrap_or(theme.colors.surface_container.into()))
        .border(|b| {
            b.top(1).color(cfg.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into()))
        })
        .layout(|l| {
            l.direction(LeftToRight)
                .gap(cfg.gap.unwrap_or(8.0) as u16)
                .padding(cfg.padding.unwrap_or(12.0) as u16)
                .align(Left, CenterY)
        })
        .children(inner);
}

/// Bottom log-progress bar (nvim-dialog style): a thin progress track pinned at
/// the bottom of a filled layout region. `value` in 0.0..=1.0.
pub fn log_progress(ui: &mut Ui<'_, ()>, id: impl Into<Id>, value: f32) {
    let cfg = config::effective(config::Style::current().log_progress, LogProgressConfig::get(), LogProgressConfig::merged);
    let theme = theme::theme();
    let frac = value.clamp(0.0, 1.0);
    let track_height = cfg.track_height.unwrap_or(6.0);
    let radius = cfg.radius.unwrap_or(theme.shapes.radius_sm);
    let track_color = cfg.track_color.map(Color::from).unwrap_or(theme.colors.surface_container_highest.into());
    let fill_color = cfg.fill_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    ui.element()
        .id(id)
        .width(grow!())
        .height(grow!())
        .background_color(theme.colors.surface_container)
        .border(|b| b.top(1).color(theme.colors.outline_variant))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(fixed!(track_height))
                .layout(|l| l.padding((cfg.padding.unwrap_or(8.0) as u16, 0, cfg.padding.unwrap_or(8.0) as u16, 0)))
                .children(|ui| {
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .background_color(track_color)
                        .corner_radius(radius)
                        .empty();
                    ui.element()
                        .width(ply_engine::layout::Sizing::Percent(frac))
                        .height(grow!())
                        .background_color(fill_color)
                        .corner_radius(radius)
                        .empty();
                });
        });
}
