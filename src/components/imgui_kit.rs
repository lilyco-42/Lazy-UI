//! imgui_kit — Dear ImGui 风格组件集。
//!
//! 移植 imgui 的招牌组件形态到 lazy-ply 的约定下: 立即模式、组件只接收数据、
//! 返回"下一个状态"、样式全部来自 `assets/components/*.toml`(未设置字段回退
//! M3 theme)。视觉上刻意走 imgui 的深色工具风格, 与 M3 默认形成对照。

use ply_engine::prelude::*;

use crate::components::config::{self, CollapsingHeaderConfig, DragFloatConfig, ImProgressBarConfig, ImWindowConfig, PlotLinesConfig};
use crate::theme;
use std::cell::RefCell;

fn cfg<T: Copy>(attrs: Option<T>, base: &T, merge: impl FnOnce(T, T) -> T) -> T {
    config::effective(attrs, base, merge)
}

/// 带标题条的浮动窗口。`inner` 在窗口内容区内渲染, 自动铺满。
pub fn im_window(
    ui: &mut Ui<'_, ()>,
    title: &str,
    inner: impl FnOnce(&mut Ui<'_, ()>),
) {
    let c = cfg(
        config::Style::current().im_window,
        ImWindowConfig::get(),
        ImWindowConfig::merged,
    );
    let theme = theme::theme();
    let header_h = c.header_height.unwrap_or(theme.shapes.item_height * 0.6);
    let radius = c.radius.unwrap_or(theme.shapes.radius_md);
    let gap = c.gap.unwrap_or(4.0) as u16;
    let padding = c.padding.unwrap_or(10.0) as u16;

    ui.element()
        .width(grow!())
        .height(grow!())
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container_high.into()))
        .corner_radius(radius)
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .layout(|l| l.direction(TopToBottom).gap(gap))
        .overflow(|o| o.scroll_y())
        .children(|ui| {
            // 标题条
            ui.element()
                .width(grow!())
                .height(fixed!(header_h))
                .background_color(c.header_background.map(Color::from).unwrap_or(theme.colors.surface_container.into()))
                .corner_radius((radius, radius, 0.0, 0.0))
                .layout(|l| l.padding((0, padding, 0, padding)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(title, |t| {
                        t.font_size(theme.text.body_size)
                            .color(c.title_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
                    });
                });
            // 内容区
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.direction(TopToBottom).gap(gap).padding(padding))
                .children(inner);
        });
}

/// 折叠标题。返回 `true` 表示本次点击把它展开, `false` 表示收起。
/// 状态由调用方持有(立即模式约定)。
pub fn collapsing_header(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    label: &str,
    open: bool,
    inner: impl FnOnce(&mut Ui<'_, ()>),
) -> bool {
    let c = cfg(
        config::Style::current().collapsing_header,
        CollapsingHeaderConfig::get(),
        CollapsingHeaderConfig::merged,
    );
    let theme = theme::theme();
    let height = c.height.unwrap_or(theme.shapes.item_height * 0.7);
    let pad_x = c.pad_x.unwrap_or(8.0) as u16;
    let gap = c.gap.unwrap_or(4.0) as u16;
    let body_gap = c.body_gap.unwrap_or(4.0) as u16;
    let color = c.color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into());
    let hover = c.hover.map(Color::from).unwrap_or(theme.colors.on_surface.into());
    let arrow = c.arrow_color.map(Color::from).unwrap_or(theme.colors.outline.into());

    let mut result = open;
    ui.element()
        .id(id)
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(TopToBottom).gap(body_gap))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(fixed!(height))
                .on_press(|_, _| {})
                .children(|ui| {
                    let hovered = ui.hovered();
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .layout(|l| l.direction(LeftToRight).gap(gap).padding((0, pad_x, 0, pad_x)).align(Left, CenterY))
                        .children(|ui| {
                            ui.text(if open { "▾" } else { "▸" }, |t| {
                                t.font_size(c.font_size.unwrap_or(theme.text.body_size))
                                    .color(arrow)
                            });
                            ui.text(label, |t| {
                                t.font_size(c.font_size.unwrap_or(theme.text.body_size))
                                    .color(if hovered { hover } else { color })
                            });
                        });
                });
            if ui.is_just_pressed(id) {
                result = !open;
            }
            if result {
                ui.element()
                    .width(grow!())
                    .height(fit!())
                    .layout(|l| l.direction(TopToBottom).gap(gap).padding((0, pad_x, 0, pad_x)))
                    .children(inner);
            }
        });
    result
}

thread_local! {
    /// 每个 drag_float 的拖拽状态: 按下位置 + 按下时的值。
    static DRAG: RefCell<std::collections::HashMap<u32, (f32, f32)>> =
        RefCell::new(std::collections::HashMap::new());
}

/// 拖拽数值(drag float)。点击数值槽并左右拖拽修改值, 返回新值。
/// 状态存于调用方, 拖拽过程中的增量由本组件维护在 thread_local。
pub fn drag_float(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    label: &str,
    value: f32,
    min: f32,
    max: f32,
) -> f32 {
    let c = cfg(
        config::Style::current().drag_float,
        DragFloatConfig::get(),
        DragFloatConfig::merged,
    );
    let theme = theme::theme();
    let key = Id::new(id).id;
    let height = c.height.unwrap_or(theme.shapes.touch_target * 0.6);
    let radius = c.radius.unwrap_or(theme.shapes.radius_xs);
    let pad_x = c.pad_x.unwrap_or(8.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.body_size);
    let bg = c.background.map(Color::from).unwrap_or(theme.colors.surface_container_high.into());
    let label_color = c.label_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into());
    let value_color = c.value_color.map(Color::from).unwrap_or(theme.colors.on_surface.into());

    let mut result = value;

    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(fixed!(height))
        .on_press(|_, _| {})
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.direction(LeftToRight).gap(8).padding((0, pad_x, 0, pad_x)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(label, |t| t.font_size(font_size).color(label_color));
                    ui.element()
                        .width(fit!())
                        .height(fixed!(height - 4.0))
                        .background_color(bg)
                        .corner_radius(radius)
                        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
                        .layout(|l| l.padding((0, 8, 0, 8)).align(CenterX, CenterY))
                        .children(|ui| {
                            ui.text(&format!("{value:.2}"), |t| t.font_size(font_size).color(value_color));
                        });
                });
        });

    // 拖拽逻辑: 按住左键在控件范围内拖拽。
    if let Some(b) = ui.bounding_box(Id::new(id)) {
        let (mx, _) = mouse_position();
        let inside = mx >= b.x - 4.0 && mx <= b.x + b.width + 4.0;
        if is_mouse_button_pressed(MouseButton::Left) && inside {
            DRAG.with(|m| m.borrow_mut().insert(key, (mx, value)));
        } else if is_mouse_button_down(MouseButton::Left) {
            let state = DRAG.with(|m| m.borrow().get(&key).copied());
            if let Some((start_mx, start_value)) = state {
                let speed = c.speed.unwrap_or(0.01);
                let delta = (mx - start_mx) * speed;
                result = (start_value + delta).clamp(min, max);
            }
        } else {
            DRAG.with(|m| m.borrow_mut().remove(&key));
        }
    }
    result
}

/// 迷你折线图(plot lines)。把 `values` 归一化后描线, 可选面积填充。
pub fn plot_lines(ui: &mut Ui<'_, ()>, values: &[f32], w: f32, h: f32) {
    let c = cfg(
        config::Style::current().plot_lines,
        PlotLinesConfig::get(),
        PlotLinesConfig::merged,
    );
    let theme = theme::theme();
    let width = w.max(0.0);
    let height = h.max(0.0);
    let radius = c.radius.unwrap_or(theme.shapes.radius_xs);
    let bg = c.background.map(Color::from).unwrap_or(theme.colors.surface_container_high.into());
    let line = c.line_color.map(Color::from).unwrap_or(theme.colors.primary.into());
    let grid = c.grid_color.map(Color::from).unwrap_or(theme.colors.surface_container_highest.into());

    ui.element()
        .width(fixed!(width))
        .height(fixed!(height))
        .background_color(bg)
        .corner_radius(radius)
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .children(|ui| {
            // 背景网格
            for i in 1..4 {
                let y = height * i as f32 / 4.0;
                ui.element()
                    .width(grow!())
                    .height(fixed!(1.0))
                    .floating(|f| f.attach_parent().anchor((Left, Top), (Left, Top)).offset((0.0, y)))
                    .background_color(grid)
                    .empty();
            }
            // 折线: 每个采样点画一小段。
            if values.len() >= 2 {
                let (lo, hi) = values.iter().fold((f32::MAX, f32::MIN), |(mn, mx), &v| {
                    (mn.min(v), mx.max(v))
                });
                let span = (hi - lo).max(1e-6);
                let n = values.len() as f32;
                let step_x = width / (n - 1.0);
                let mut prev: Option<(f32, f32)> = None;
                for (i, &v) in values.iter().enumerate() {
                    let x = i as f32 * step_x;
                    let y = height - (v - lo) / span * (height - 4.0) - 2.0;
                    if let Some((px, py)) = prev {
                        draw_line(px, py, x, y, c.thickness.unwrap_or(2.0), line.into());
                    }
                    prev = Some((x, y));
                }
            }
        });
}

/// 带百分比文字的进度条(imgui 风格: 轨道内嵌文字)。
pub fn im_progress_bar(ui: &mut Ui<'_, ()>, fraction: f32) {
    let c = cfg(
        config::Style::current().im_progress_bar,
        ImProgressBarConfig::get(),
        ImProgressBarConfig::merged,
    );
    let theme = theme::theme();
    let frac = fraction.clamp(0.0, 1.0);
    let track_height = c.track_height.unwrap_or(theme.shapes.track_height * 3.0);
    let radius = c.radius.unwrap_or(theme.shapes.radius_sm);
    let track = c.track_color.map(Color::from).unwrap_or(theme.colors.surface_container_highest.into());
    let fill = c.fill_color.map(Color::from).unwrap_or(theme.colors.primary.into());

    ui.element()
        .width(grow!())
        .height(fixed!(track_height))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(track)
                .corner_radius(radius)
                .children(|ui| {
                    ui.element()
                        .width(ply_engine::layout::Sizing::Percent(frac))
                        .height(grow!())
                        .background_color(fill)
                        .corner_radius(radius)
                        .empty();
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .layout(|l| l.align(CenterX, CenterY))
                        .children(|ui| {
                            ui.text(&format!("{:.0}%", frac * 100.0), |t| {
                                t.font_size(c.font_size.unwrap_or(theme.text.label_size))
                                    .color(c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
                            });
                        });
                });
        });
}

/// 项目符号文本(bullet text)。
pub fn bullet_text(ui: &mut Ui<'_, ()>, text: &str) {
    let theme = theme::theme();
    ui.element()
        .width(fit!())
        .height(fit!())
        .layout(|l| l.direction(LeftToRight).gap(8).align(Left, CenterY))
        .children(|ui| {
            ui.text("•", |t| t.font_size(theme.text.body_size).color(theme.colors.primary));
            ui.text(text, |t| t.font_size(theme.text.body_size).color(theme.colors.on_surface));
        });
}
