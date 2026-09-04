//! gpui_kit — GPUI/Zed 风格组件集(tailwind 风格工具类)。
//!
//! 移植 gpui 的 tailwind-like 心智: 一个 `div` 是万能容器, 样式走链式工具类;
//! `kbd`/`chip`/`badge`/`avatar`/`code` 是高频小部件。lazy-ply 约定不变:
//! 立即模式、样式来自 `assets/components/*.toml`、未设置字段回退 M3 theme。

use ply_engine::prelude::*;

use crate::components::config::{self, AvatarConfig, BadgeConfig, ChipConfig, CodeConfig, DivConfig, KbdConfig};
use crate::theme;

fn cfg<T: Copy + config::ScaleSizes>(
    attrs: Option<T>,
    base: &T,
    merge: impl FnOnce(T, T) -> T,
) -> T {
    config::effective(attrs, base, merge)
}

/// 通用容器(`div`)。tailwind 风格: 用命名工具方法设置背景/圆角/内边距,
/// 未设置的字段回退到 `div.toml` → M3 theme。`inner` 铺满容器。
pub fn div(
    ui: &mut Ui<'_, ()>,
    inner: impl FnOnce(&mut Ui<'_, ()>),
) {
    let c = cfg(
        config::Style::current().div,
        DivConfig::get(),
        DivConfig::merged,
    );
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container_low.into()))
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_md))
        .border(|b| {
            b.all(c.border_width.unwrap_or(1.0) as u16)
                .color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into()))
        })
        .layout(|l| l.direction(TopToBottom).gap(c.gap.unwrap_or(8.0) as u16).padding(c.padding.unwrap_or(12.0) as u16))
        .children(inner);
}

/// 键盘按键(kbd)。例如 `kbd(ui, "Ctrl")` → `[ Ctrl ]`。
pub fn kbd(ui: &mut Ui<'_, ()>, key: &str) {
    let c = cfg(
        config::Style::current().kbd,
        KbdConfig::get(),
        KbdConfig::merged,
    );
    let theme = theme::theme();
    let height = c.height.unwrap_or(theme.shapes.touch_target * 0.5);
    ui.element()
        .width(fit!())
        .height(fixed!(height))
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container_high.into()))
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_xs))
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .layout(|l| l.padding((0, c.pad_x.unwrap_or(8.0) as u16, 0, c.pad_x.unwrap_or(8.0) as u16)).align(CenterX, CenterY))
        .children(|ui| {
            ui.text(key, |t| {
                t.font_size(c.font_size.unwrap_or(theme.text.label_size))
                    .color(c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
            });
        });
}

/// 可选中过滤芯片(chip)。返回 `true` 表示本次被点击(切换选中态由调用方做)。
pub fn chip(ui: &mut Ui<'_, ()>, id: &'static str, label: &str, selected: bool) -> bool {
    let c = cfg(
        config::Style::current().chip,
        ChipConfig::get(),
        ChipConfig::merged,
    );
    let theme = theme::theme();
    let height = c.height.unwrap_or(theme.shapes.touch_target * 0.7);
    ui.element()
        .id(Id::new(id))
        .width(fit!())
        .height(fixed!(height))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(fit!())
                .height(grow!())
                .background_color(if selected {
                    c.selected_bg.map(Color::from).unwrap_or(theme.colors.primary.into())
                } else {
                    c.background.map(Color::from).unwrap_or(theme.colors.surface_container_low.into())
                })
                .corner_radius(c.radius.unwrap_or(height * 0.5))
                .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
                .layout(|l| {
                    l.padding((0, c.pad_x.unwrap_or(14.0) as u16, 0, c.pad_x.unwrap_or(14.0) as u16))
                        .align(CenterX, CenterY)
                })
                .children(|ui| {
                    ui.text(label, |t| {
                        t.font_size(c.font_size.unwrap_or(theme.text.label_size)).color(
                            if selected {
                                c.selected_fg.map(Color::from).unwrap_or(theme.colors.on_primary.into())
                            } else {
                                c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into())
                            },
                        )
                    });
                });
        });
    ui.is_just_pressed(id)
}

/// 状态徽标(badge)。`tone` 0=中性 1=主题色 2=错误色, 决定配色语义。
pub fn badge(ui: &mut Ui<'_, ()>, text: &str, tone: u8) {
    let c = cfg(
        config::Style::current().badge,
        BadgeConfig::get(),
        BadgeConfig::merged,
    );
    let theme = theme::theme();
    let (bg, fg) = match tone {
        1 => (
            c.background.map(Color::from).unwrap_or(theme.colors.primary_container.into()),
            c.text_color.map(Color::from).unwrap_or(theme.colors.on_primary_container.into()),
        ),
        2 => (
            c.background.map(Color::from).unwrap_or(theme.colors.error_container.into()),
            c.text_color.map(Color::from).unwrap_or(theme.colors.on_error_container.into()),
        ),
        _ => (
            c.background.map(Color::from).unwrap_or(theme.colors.secondary_container.into()),
            c.text_color.map(Color::from).unwrap_or(theme.colors.on_secondary_container.into()),
        ),
    };
    ui.element()
        .width(fit!())
        .height(fit!())
        .background_color(bg)
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_sm))
        .layout(|l| l.padding((2, c.pad_x.unwrap_or(10.0) as u16, 2, c.pad_x.unwrap_or(10.0) as u16)))
        .children(|ui| {
            ui.text(text, |t| {
                t.font_size(c.font_size.unwrap_or(theme.text.label_size)).color(fg)
            });
        });
}

/// 圆形头像(avatar)。显示文本的首字符。
pub fn avatar(ui: &mut Ui<'_, ()>, name: &str) {
    let c = cfg(
        config::Style::current().avatar,
        AvatarConfig::get(),
        AvatarConfig::merged,
    );
    let theme = theme::theme();
    let size = c.size.unwrap_or(theme.shapes.touch_target * 0.8);
    let initial = name.chars().next().unwrap_or('?').to_string();
    ui.element()
        .width(fixed!(size))
        .height(fixed!(size))
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.primary.into()))
        .corner_radius(c.radius.unwrap_or(size * 0.5))
        .layout(|l| l.align(CenterX, CenterY))
        .children(|ui| {
            ui.text(&initial, |t| {
                t.font_size(c.font_size.unwrap_or(theme.text.title_size))
                    .color(c.text_color.map(Color::from).unwrap_or(theme.colors.on_primary.into()))
            });
        });
}

/// 行内代码(code)。等宽感通过暗底 + 主题色文字实现。
pub fn code(ui: &mut Ui<'_, ()>, text: &str) {
    let c = cfg(
        config::Style::current().code,
        CodeConfig::get(),
        CodeConfig::merged,
    );
    let theme = theme::theme();
    ui.element()
        .width(fit!())
        .height(fit!())
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container_high.into()))
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_xs))
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .layout(|l| {
            l.padding((c.pad_y.unwrap_or(2.0) as u16, c.pad_x.unwrap_or(8.0) as u16, c.pad_y.unwrap_or(2.0) as u16, c.pad_x.unwrap_or(8.0) as u16))
        })
        .children(|ui| {
            ui.text(text, |t| {
                t.font_size(c.font_size.unwrap_or(theme.text.body_size))
                    .color(c.text_color.map(Color::from).unwrap_or(theme.colors.primary.into()))
            });
        });
}
