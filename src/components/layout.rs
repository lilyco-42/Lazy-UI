//! Declarative app layout — the window skeleton is described in
//! `assets/app_layout.toml`, and this module AUTO-INFERS the flex composition:
//!
//! - Un-anchored regions → one horizontal flex row (the "main" area).
//! - `anchor = "bottom"` regions → stacked full-width at the bottom.
//! - Region size from `width` / `height` (px) or `height_percent` (of window).
//!
//! Layout relationships live in config, never hard-coded in code.

use ply_engine::prelude::*;
use serde::Deserialize;
use std::sync::OnceLock;

use crate::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionRole {
    Sidebar,
    Content,
    Status,
    Progress,
}

impl Default for RegionRole {
    fn default() -> Self {
        RegionRole::Content
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    Bottom,
    Top,
    Left,
    Right,
}

/// A region of the app skeleton. Only `name`/`role` are required; every other
/// field is optional and omitted fields get sensible defaults.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Region {
    pub name: String,
    pub role: RegionRole,
    pub anchor: Option<Anchor>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub height_percent: Option<f32>,
    pub grow: bool,
}

impl Default for Region {
    fn default() -> Self {
        Self {
            name: String::new(),
            role: RegionRole::Content,
            anchor: None,
            width: None,
            height: None,
            height_percent: None,
            grow: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppLayout {
    pub gap: f32,
    pub regions: Vec<Region>,
}

impl Default for AppLayout {
    fn default() -> Self {
        Self {
            gap: 8.0,
            regions: Vec::new(),
        }
    }
}

impl AppLayout {
    /// Loads `assets/app_layout.toml` once; falls back to empty layout.
    pub fn get() -> &'static Self {
        static LAYOUT: OnceLock<AppLayout> = OnceLock::new();
        LAYOUT.get_or_init(|| toml::from_str(include_str!("../../assets/app_layout.toml")).unwrap_or_default())
    }

    /// Regions that flow in the main row (no anchor).
    fn flow(&self) -> Vec<&Region> {
        self.regions.iter().filter(|r| r.anchor.is_none()).collect()
    }

    /// Regions pinned to the bottom of the window.
    fn bottom(&self) -> Vec<&Region> {
        self.regions
            .iter()
            .filter(|r| r.anchor == Some(Anchor::Bottom))
            .collect()
    }
}

/// Renders the inferred skeleton and calls `content` once per region, so the
/// caller can fill each region with actual components.
///
/// ```text
/// +----------------------------+
/// | sidebar |      panel       |   <- flex row (grows)
/// | (240px) |      (grow)      |
/// +----------------------------+
/// |        status_bar          |   <- full width
/// +----------------------------+
/// |       log_progress         |   <- full width, height_percent 0.10
/// +----------------------------+
/// ```
pub fn render(ui: &mut Ui<'_, ()>, content: impl Fn(&mut Ui<'_, ()>, &Region)) {
    let cfg = AppLayout::get();
    let flow = cfg.flow();
    let bottom = cfg.bottom();
    let theme = theme::theme();

    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(theme.colors.surface)
        .layout(|l| l.direction(TopToBottom).gap(cfg.gap as u16))
        .children(|ui| {
            if !flow.is_empty() {
                ui.element()
                    .width(grow!())
                    .height(grow!())
                    .layout(|l| l.direction(LeftToRight))
                    .children(|ui| {
                        for region in &flow {
                            render_region(ui, region, true, &content);
                        }
                    });
            }
            for region in &bottom {
                render_region(ui, region, false, &content);
            }
        });
}

fn render_region(
    ui: &mut Ui<'_, ()>,
    region: &Region,
    flow_row: bool,
    content: &impl Fn(&mut Ui<'_, ()>, &Region),
) {
    let mut el = ui.element();
    if flow_row {
        let w = match region.width {
            Some(w) => fixed!(w),
            None if region.grow => grow!(),
            None => fit!(),
        };
        el = el.width(w).height(grow!());
    } else {
        let h = if let Some(h) = region.height {
            fixed!(h)
        } else if let Some(p) = region.height_percent {
            ply_engine::layout::Sizing::Percent(p)
        } else if region.grow {
            grow!()
        } else {
            fit!()
        };
        el = el.width(grow!()).height(h);
    }
    el.children(|ui| content(ui, region));
}
