//! 中文(汉化)组件 — 原版英文组件原样保留(见本目录其余文件),
//! 这里仅追加一份中文名的「即时模式」包装:调用时无需显式传 `ui`,
//! 组件通过 [`crate::immediate::with_ui`] 装好的帧内上下文自动渲染。
//!
//! 例如:
//!
//! ```ignore
//! with_ui(&mut ui, || {
//!     if 按钮("点我") {
//!         println!("Hi");
//!     } else if 复选框("auto", true, "自动保存") {
//!         println!("自动保存已切换");
//!     }
//! });
//! ```
//!
//! 不含 `ui` 参数、参数顺序与原版完全一致,返回值语义也一一对应。

use ply_engine::prelude::*;

use crate::immediate::current_ui;
use crate::components::{
    button_id, chat_panel, checkbox, combo, divider, listbox, progress, radio,
    radio_group, selectable, slider, switch, tabs, text_field, text_field_outlined,
    tooltip,
};
use crate::components::container::{log_progress, panel, sidebar, status_bar};
use crate::components::text::{body, headline, label, title};
use crate::components::chat_panel::{ChatPanelEvents, ChatPanelState};
use std::cell::RefCell;
use std::rc::Rc;

/// `按钮`(实心) — 渲染按钮并在「刚好按下」的那一帧返回 `true`。
pub fn 按钮(label: &str) -> bool {
    let ui = current_ui();
    let id = button_id(&mut *ui, label);
    ui.is_just_pressed(id)
}

/// `大标题`(headline)。
pub fn 大标题(text: &str) {
    headline(&mut *current_ui(), text);
}

/// `标题`(title)。
pub fn 标题(text: &str) {
    title(&mut *current_ui(), text);
}

/// `正文`(body)。
pub fn 正文(text: &str) {
    body(&mut *current_ui(), text);
}

/// `标签`(label,弱化的说明文字)。
pub fn 标签(text: &str) {
    label(&mut *current_ui(), text);
}

/// `复选框` — 返回切换后的状态(调用方自行保存)。
pub fn 复选框(id: impl Into<Id>, checked: bool, label: &str) -> bool {
    checkbox(&mut *current_ui(), id, checked, label)
}

/// `开关` — 返回切换后的状态(调用方自行保存)。
pub fn 开关(id: impl Into<Id>, checked: bool, label: &str) -> bool {
    switch(&mut *current_ui(), id, checked, label)
}

/// `单选` — 这一帧被选中时返回 `true`。
pub fn 单选(id: impl Into<Id>, selected: bool, label: &str) -> bool {
    radio(&mut *current_ui(), id, selected, label)
}

/// `单选组` — 返回新选中的下标。
pub fn 单选组(id: &'static str, options: &[&str], selected: usize) -> usize {
    radio_group(&mut *current_ui(), id, options, selected)
}

/// `列表项` — 这一帧被激活时返回 `true`。
pub fn 列表项(id: impl Into<Id>, selected: bool, label: &str) -> bool {
    selectable(&mut *current_ui(), id, selected, label)
}

/// `列表框`(可滚动列表)— 返回新选中的下标。`visible` 为可见行数。
pub fn 列表框(id: &'static str, options: &[&str], selected: usize, visible: usize) -> usize {
    listbox(&mut *current_ui(), id, options, selected, visible)
}

/// `下拉框` — 返回新选中的下标。
pub fn 下拉框(id: &'static str, options: &[&str], selected: usize) -> usize {
    combo(&mut *current_ui(), id, options, selected)
}

/// `选项卡` — 返回新选中的下标。
pub fn 选项卡(id: &'static str, items: &[&str], selected: usize) -> usize {
    tabs(&mut *current_ui(), id, items, selected)
}

/// `进度条`(线性)— `fraction` 为 0.0..=1.0。
pub fn 进度条(fraction: f32) {
    progress(&mut *current_ui(), fraction);
}

/// `分割线`(水平)。
pub fn 分割线() {
    divider(&mut *current_ui());
}

/// `输入框`(填充式)。值存于 Ply,用 `ui.get_text_value(id)` 读取。
pub fn 输入框(id: &'static str, placeholder: &str) {
    text_field(&mut *current_ui(), id, placeholder);
}

/// `描边输入框`。
pub fn 描边输入框(id: &'static str, placeholder: &str) {
    text_field_outlined(&mut *current_ui(), id, placeholder);
}

/// `滑块` — 返回拖动后的值。
pub fn 滑块(id: impl Into<Id>, label: &str, value: f32, min: f32, max: f32) -> f32 {
    slider(&mut *current_ui(), id, label, value, min, max)
}

/// `提示`(悬停气泡)— `inner` 为被包裹的内容。
pub fn 提示(id: &'static str, text: &str, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    tooltip(&mut *current_ui(), id, text, inner);
}

/// `侧边栏`。
pub fn 侧边栏(inner: impl FnOnce(&mut Ui<'_, ()>)) {
    sidebar(&mut *current_ui(), inner);
}

/// `面板`。
pub fn 面板(inner: impl FnOnce(&mut Ui<'_, ()>)) {
    panel(&mut *current_ui(), inner);
}

/// `状态栏`。
pub fn 状态栏(inner: impl FnOnce(&mut Ui<'_, ()>)) {
    status_bar(&mut *current_ui(), inner);
}

/// `日志进度条`(nvim-dialog 风格,贴底)。`value` 为 0.0..=1.0。
pub fn 日志进度条(id: impl Into<Id>, value: f32) {
    log_progress(&mut *current_ui(), id, value);
}

/// `聊天面板` — 事件累积进 `events` 供调用方排空处理。
pub fn 聊天面板(state: &ChatPanelState, events: &Rc<RefCell<ChatPanelEvents>>) {
    chat_panel(&mut *current_ui(), state, events);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::immediate::with_ui;

    fn headless_ply() -> Ply<()> {
        Ply::new_headless(ply_engine::math::Dimensions::new(800.0, 600.0))
    }

    #[test]
    fn zh_components_render_inside_scope() {
        let mut ply = headless_ply();
        let mut ui = ply.begin();
        with_ui(&mut ui, || {
            assert!(!按钮("点我"));
            复选框("auto", true, "自动保存");
            开关("dark", false, "深色模式");
            单选("a", false, "选项A");
            单选组("rg", &["甲", "乙"], 0);
            列表项("s1", false, "条目");
            列表框("lb", &["一", "二"], 0, 4);
            _ = 下拉框("co", &["苹果", "橘子"], 0);
            _ = 选项卡("tab", &["标签一", "标签二"], 0);
            进度条(0.5);
            分割线();
            大标题("基准");
            标题("分区");
            正文("内容");
            标签("注释");
            输入框("f1", "说点什么…");
            描边输入框("f2", "搜索");
            提示("tip", "气泡", |ui| {
                ui.text("?", |t| t.font_size(16));
            });
            面板(|ui| {
                with_ui(ui, || {
                    if 按钮("关闭") {
                        unreachable!();
                    }
                });
            });
            侧边栏(|_| {});
            状态栏(|_| {});
            日志进度条("lp", 0.3);
        });
    }

    #[test]
    #[should_panic(expected = "outside with_ui")]
    fn zh_component_panics_outside_scope() {
        let _ = 复选框("x", false, "越界");
    }
}