//! eui_neo_kit — EUI-NEO 声明式 DSL 风格组件集。
//!
//! 移植 EUI-NEO 的组件形态: 受控组件(页面持有值, 组件回调 next value)、
//! 稳定 id(内部子节点 `id + ".name"`)、`onChange` 风格交互。lazy-ply 约定
//! 不变: 立即模式返回新状态、样式来自 `assets/components/*.toml`。

use ply_engine::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::components::config::{self, CardConfig, DataTableConfig, DialogConfig, SegmentedConfig, StepperConfig, ToastConfig};
use crate::components::button_id;
use crate::theme;

fn cfg<T: Copy>(attrs: Option<T>, base: &T, merge: impl FnOnce(T, T) -> T) -> T {
    config::effective(attrs, base, merge)
}

/// 分段选择器(segmented)。返回新选中下标。
pub fn segmented(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    options: &[&str],
    selected: usize,
) -> usize {
    let c = cfg(
        config::Style::current().segmented,
        SegmentedConfig::get(),
        SegmentedConfig::merged,
    );
    let theme = theme::theme();
    let mut result = selected;
    let height = c.height.unwrap_or(theme.shapes.touch_target * 0.75);
    let radius = c.radius.unwrap_or(theme.shapes.radius_sm);
    let pad_x = c.pad_x.unwrap_or(8.0) as u16;
    let font_size = c.font_size.unwrap_or(theme.text.label_size);

    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(fixed!(height))
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container_high.into()))
        .corner_radius(radius)
        .layout(|l| l.direction(LeftToRight).padding(pad_x))
        .children(|ui| {
            for (i, option) in options.iter().enumerate() {
                let oid = Id::from((id, i as u32));
                let active = i == selected;
                ui.element()
                    .id(oid.clone())
                    .width(fit!())
                    .height(grow!())
                    .on_press(|_, _| {})
                    .children(|ui| {
                        ui.element()
                            .width(fit!())
                            .height(grow!())
                            .background_color(if active {
                                c.selected_bg.map(Color::from).unwrap_or(theme.colors.primary.into())
                            } else {
                                theme::TRANSPARENT.into()
                            })
                            .corner_radius(radius)
                            .layout(|l| l.padding((0, pad_x, 0, pad_x)).align(CenterX, CenterY))
                            .children(|ui| {
                                ui.text(option, |t| {
                                    t.font_size(font_size).color(if active {
                                        c.selected_fg.map(Color::from).unwrap_or(theme.colors.on_primary.into())
                                    } else {
                                        c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into())
                                    })
                                });
                            });
                    });
                if ui.is_just_pressed(oid) {
                    result = i;
                }
            }
        });
    result
}

/// 数字步进器(stepper)。返回新值。
pub fn stepper(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    value: i32,
    min: i32,
    max: i32,
) -> i32 {
    let c = cfg(
        config::Style::current().stepper,
        StepperConfig::get(),
        StepperConfig::merged,
    );
    let theme = theme::theme();
    let mut result = value.clamp(min, max);
    let height = c.height.unwrap_or(theme.shapes.touch_target * 0.75);
    let radius = c.radius.unwrap_or(theme.shapes.radius_xs);
    let step = c.step.unwrap_or(1);

    ui.element()
        .id(Id::new(id))
        .width(fit!())
        .height(fixed!(height))
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container.into()))
        .corner_radius(radius)
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY))
        .children(|ui| {
            let minus = Id::from((id, 0u32));
            ui.element()
                .id(minus.clone())
                .width(fixed!(height))
                .height(grow!())
                .on_press(|_, _| {})
                .children(|ui| {
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .background_color(c.button_bg.map(Color::from).unwrap_or(theme.colors.primary.into()))
                        .corner_radius(radius)
                        .layout(|l| l.align(CenterX, CenterY))
                        .children(|ui| {
                            ui.text("−", |t| {
                                t.font_size(c.font_size.unwrap_or(theme.text.body_size))
                                    .color(c.button_fg.map(Color::from).unwrap_or(theme.colors.on_primary.into()))
                            });
                        });
                });
            if ui.is_just_pressed(minus) {
                result = (result - step).max(min);
            }

            ui.element()
                .width(fixed!(56.0))
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(&value.to_string(), |t| {
                        t.font_size(c.font_size.unwrap_or(theme.text.body_size))
                            .color(c.text_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
                    });
                });

            let plus = Id::from((id, 1u32));
            ui.element()
                .id(plus.clone())
                .width(fixed!(height))
                .height(grow!())
                .on_press(|_, _| {})
                .children(|ui| {
                    ui.element()
                        .width(grow!())
                        .height(grow!())
                        .background_color(c.button_bg.map(Color::from).unwrap_or(theme.colors.primary.into()))
                        .corner_radius(radius)
                        .layout(|l| l.align(CenterX, CenterY))
                        .children(|ui| {
                            ui.text("+", |t| {
                                t.font_size(c.font_size.unwrap_or(theme.text.body_size))
                                    .color(c.button_fg.map(Color::from).unwrap_or(theme.colors.on_primary.into()))
                            });
                        });
                });
            if ui.is_just_pressed(plus) {
                result = (result + step).min(max);
            }
        });
    result
}

/// 卡片容器(card)。带背景、边框、圆角和内容内边距。
pub fn card(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    let c = cfg(
        config::Style::current().card,
        CardConfig::get(),
        CardConfig::merged,
    );
    let theme = theme::theme();
    ui.element()
        .width(grow!())
        .height(fit!())
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container_lowest.into()))
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_lg))
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .layout(|l| {
            l.direction(TopToBottom)
                .gap(c.gap.unwrap_or(8.0) as u16)
                .padding(c.padding.unwrap_or(16.0) as u16)
        })
        .children(inner);
}

/// 模态对话框(dialog)。受控 `open`: 页面传入开关, 点击遮罩或按钮时通过
/// 返回值关掉。返回 `false` 表示本次被请求关闭。
pub fn dialog(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    open: bool,
    title: &str,
    body: &str,
    confirm: &str,
    cancel: &str,
) -> bool {
    let c = cfg(
        config::Style::current().dialog,
        DialogConfig::get(),
        DialogConfig::merged,
    );
    let theme = theme::theme();
    let mut result = open;
    if !open {
        return result;
    }
    let width = c.width.unwrap_or(320.0);
    let radius = c.radius.unwrap_or(theme.shapes.radius_lg);
    let gap = c.gap.unwrap_or(12.0) as u16;
    let padding = c.padding.unwrap_or(20.0) as u16;

    // 遮罩
    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(grow!())
        .background_color(theme::TRANSPARENT)
        .on_press(|_, _| {})
        .children(|ui| {
            let cancel_oid = Id::from((id, 0u32));
            ui.element()
                .width(fixed!(width))
                .height(fit!())
                .floating(|f| f.anchor((CenterX, CenterY), (CenterX, CenterY)).z_index(300))
                .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface.into()))
                .corner_radius(radius)
                .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
                .layout(|l| l.direction(TopToBottom).gap(gap).padding(padding))
                .children(|ui| {
                    ui.text(title, |t| {
                        t.font_size(theme.text.title_size)
                            .color(c.title_color.map(Color::from).unwrap_or(theme.colors.on_surface.into()))
                    });
                    ui.text(body, |t| {
                        t.font_size(theme.text.body_size)
                            .color(theme.colors.on_surface_variant)
                    });
                    ui.element()
                        .width(grow!())
                        .height(fit!())
                        .layout(|l| l.direction(LeftToRight).gap(gap).align(Right, CenterY))
                        .children(|ui| {
                            let cancel_id = button_id(ui, cancel);
                            let confirm_id = button_id(ui, confirm);
                            if ui.is_just_pressed(cancel_id) || ui.is_just_pressed(cancel_oid) {
                                result = false;
                            }
                            if ui.is_just_pressed(confirm_id) {
                                result = false;
                            }
                        });
                });
        });
    // 点遮罩(非对话框区域)关闭
    if ui.is_just_pressed(Id::new(id)) {
        result = false;
    }
    result
}

/// 数据表格(data_table)。返回新选中的行下标(点击行切换)。
pub fn data_table(
    ui: &mut Ui<'_, ()>,
    id: &'static str,
    headers: &[&str],
    rows: &[Vec<&str>],
    selected: usize,
) -> usize {
    let c = cfg(
        config::Style::current().data_table,
        DataTableConfig::get(),
        DataTableConfig::merged,
    );
    let theme = theme::theme();
    let mut result = selected;
    let row_h = c.row_height.unwrap_or(theme.shapes.item_height * 0.75);
    let header_h = c.header_height.unwrap_or(theme.shapes.item_height * 0.7);
    let radius = c.radius.unwrap_or(theme.shapes.radius_sm);
    let font_size = c.font_size.unwrap_or(theme.text.body_size);

    ui.element()
        .id(Id::new(id))
        .width(grow!())
        .height(fit!())
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.surface_container_lowest.into()))
        .corner_radius(radius)
        .border(|b| b.all(1).color(c.border.map(Color::from).unwrap_or(theme.colors.outline_variant.into())))
        .children(|ui| {
            // 表头
            ui.element()
                .width(grow!())
                .height(fixed!(header_h))
                .background_color(c.header_bg.map(Color::from).unwrap_or(theme.colors.surface_container.into()))
                .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
                .children(|ui| {
                    for h in headers {
                        ui.element()
                            .width(fit!())
                            .height(grow!())
                            .layout(|l| l.padding((0, 12, 0, 12)).align(Left, CenterY))
                            .children(|ui| {
                                ui.text(h, |t| {
                                    t.font_size(font_size)
                                        .color(c.header_color.map(Color::from).unwrap_or(theme.colors.on_surface_variant.into()))
                                });
                            });
                    }
                });
            // 数据行
            for (i, row) in rows.iter().enumerate() {
                let oid = Id::from((id, i as u32));
                ui.element()
                    .id(oid.clone())
                    .width(grow!())
                    .height(fixed!(row_h))
                    .background_color(if i == selected {
                        c.selected_bg.map(Color::from).unwrap_or(theme.colors.secondary_container.into())
                    } else {
                        theme::TRANSPARENT.into()
                    })
                    .on_press(|_, _| {})
                    .children(|ui| {
                        ui.element()
                            .width(grow!())
                            .height(grow!())
                            .layout(|l| l.direction(LeftToRight).align(Left, CenterY))
                            .children(|ui| {
                                for cell in row {
                                    ui.element()
                                        .width(fit!())
                                        .height(grow!())
                                        .layout(|l| l.padding((0, 12, 0, 12)).align(Left, CenterY))
                                        .children(|ui| {
                                            ui.text(cell, |t| {
                                                t.font_size(font_size).color(if i == selected {
                                                    c.selected_fg.map(Color::from).unwrap_or(theme.colors.on_secondary_container.into())
                                                } else {
                                                    c.row_color.map(Color::from).unwrap_or(theme.colors.on_surface.into())
                                                })
                                            });
                                        });
                                }
                            });
                    });
                if ui.is_just_pressed(oid) {
                    result = i;
                }
            }
        });
    result
}

thread_local! {
    /// toast 的自动消失计时: id → 剩余秒。
    static TOAST_TTL: RefCell<HashMap<&'static str, f32>> = RefCell::new(HashMap::new());
}

/// 右上角 toast。受控 `show`: 传入 `true` 时显示并计时, 计时到自动消失;
/// 返回 `false` 表示该隐藏了。`dt` 为上一帧耗时(秒)。
pub fn toast(ui: &mut Ui<'_, ()>, id: &'static str, show: bool, text: &str, dt: f32) -> bool {
    let c = cfg(
        config::Style::current().toast,
        ToastConfig::get(),
        ToastConfig::merged,
    );
    let theme = theme::theme();
    let ttl = if show {
        TOAST_TTL.with(|m| {
            let mut m = m.borrow_mut();
            let v = m.entry(id).or_insert(3.0);
            *v = 3.0;
            *v
        })
    } else {
        TOAST_TTL.with(|m| {
            let mut m = m.borrow_mut();
            if let Some(v) = m.get_mut(id) {
                *v -= dt;
                *v
            } else {
                0.0
            }
        })
    };
    let visible = ttl > 0.0;
    if !visible {
        TOAST_TTL.with(|m| m.borrow_mut().remove(id));
        return false;
    }
    let offset_y = c.offset_y.unwrap_or(24.0);
    ui.element()
        .id(id)
        .width(fit!())
        .height(fit!())
        .floating(|f| f.anchor((Right, Top), (Right, Top)).offset((-24.0, offset_y)).z_index(400))
        .background_color(c.background.map(Color::from).unwrap_or(theme.colors.inverse_surface.into()))
        .corner_radius(c.radius.unwrap_or(theme.shapes.radius_md))
        .layout(|l| {
            l.padding((
                c.pad_y.unwrap_or(10.0) as u16,
                c.pad_x.unwrap_or(16.0) as u16,
                c.pad_y.unwrap_or(10.0) as u16,
                c.pad_x.unwrap_or(16.0) as u16,
            ))
        })
        .children(|ui| {
            ui.text(text, |t| {
                t.font_size(c.font_size.unwrap_or(theme.text.body_size))
                    .color(c.text_color.map(Color::from).unwrap_or(theme.colors.inverse_on_surface.into()))
            });
        });
    true
}
