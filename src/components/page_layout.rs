//! Page layout — auto-inferred from a layout-relationship toml. A page is a
//! flex container; each child is positioned purely by the declared
//! relationship. Convention over configuration: every field is optional and
//! falls back to a centered vertical flex.

use ply_engine::prelude::*;
use serde::Deserialize;

/// Flex direction of the page container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageDirection {
    TopToBottom,
    LeftToRight,
}

impl Default for PageDirection {
    fn default() -> Self {
        Self::TopToBottom
    }
}

/// Horizontal alignment (own enum so the toml stays deserializable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageAlign {
    Left,
    CenterX,
    Right,
}

impl PageAlign {
    fn to_ply(self) -> AlignX {
        match self {
            PageAlign::Left => AlignX::Left,
            PageAlign::CenterX => AlignX::CenterX,
            PageAlign::Right => AlignX::Right,
        }
    }
}

impl Default for PageAlign {
    fn default() -> Self {
        Self::CenterX
    }
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageAlignY {
    Top,
    CenterY,
    Bottom,
}

impl PageAlignY {
    fn to_ply(self) -> AlignY {
        match self {
            PageAlignY::Top => AlignY::Top,
            PageAlignY::CenterY => AlignY::CenterY,
            PageAlignY::Bottom => AlignY::Bottom,
        }
    }
}

impl Default for PageAlignY {
    fn default() -> Self {
        Self::CenterY
    }
}

/// A child mounted inside the page, positioned by the inferred layout.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PageChild {
    /// Component key the page matches to mount (e.g. `"蓝色按钮"`).
    pub component: String,
    /// Fraction of the page width (0..=1). `None` → fit content.
    pub width_percent: Option<f32>,
    /// Fixed height in px. `None` → fit content.
    pub height: Option<f32>,
}

impl Default for PageChild {
    fn default() -> Self {
        Self {
            component: String::new(),
            width_percent: None,
            height: None,
        }
    }
}

/// A page's layout, declared in a layout-relationship toml.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PageLayout {
    pub direction: PageDirection,
    pub gap: u16,
    pub padding: u16,
    pub align_x: PageAlign,
    pub align_y: PageAlignY,
    pub children: Vec<PageChild>,
}

impl Default for PageLayout {
    fn default() -> Self {
        Self {
            direction: PageDirection::TopToBottom,
            gap: 12,
            padding: 16,
            align_x: PageAlign::CenterX,
            align_y: PageAlignY::CenterY,
            children: Vec::new(),
        }
    }
}

impl PageLayout {
    /// Parses a layout-relationship toml (auto-inferred defaults on error).
    pub fn from_toml(raw: &str) -> Self {
        toml::from_str(raw).unwrap_or_default()
    }
}

/// Renders a page: a flex container (per `layout`) that mounts each child by
/// calling `mount(ui, child)`. Sizes come from the child's declared
/// `width_percent` / `height`.
pub fn render_page(
    ui: &mut Ui<'_, ()>,
    layout: &PageLayout,
    mount: impl Fn(&mut Ui<'_, ()>, &PageChild),
) {
    let dir = match layout.direction {
        PageDirection::TopToBottom => LayoutDirection::TopToBottom,
        PageDirection::LeftToRight => LayoutDirection::LeftToRight,
    };
    ui.element()
        .width(grow!())
        .height(grow!())
        .layout(|l| {
            l.direction(dir)
                .gap(layout.gap)
                .padding(layout.padding)
                .align(layout.align_x.to_ply(), layout.align_y.to_ply())
        })
        .children(|ui| {
            for child in &layout.children {
                let mut el = ui.element();
                el = match child.width_percent {
                    Some(p) => el.width(ply_engine::layout::Sizing::Percent(p.clamp(0.0, 1.0))),
                    None => el.width(fit!()),
                };
                el = match child.height {
                    Some(h) => el.height(fixed!(h)),
                    None => el.height(fit!()),
                };
                // Explicit layout: center the child in its slot. Without a
                // layout config the wrapper's default placement is undefined
                // and can misalign the mounted component.
                el.layout(|l| l.align(CenterX, CenterY))
                    .children(|ui| mount(ui, child));
            }
        });
}