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
    button_id_kind, chat_panel, checkbox, combo, divider, listbox, progress, radio,
    radio_group, selectable, slider, switch, tabs, text_field, text_field_outlined,
    tooltip,
};
use crate::components::button::ButtonKind;
use crate::components::container::{log_progress, panel, sidebar, status_bar};
use crate::components::text::{body, headline, label, title};
use crate::components::chat_panel::{ChatPanelEvents, ChatPanelState};
use crate::components::{
    avatar, badge, bullet_text, card, chip, code, collapsing_header, data_table, dialog,
    div, drag_float, im_progress_bar, im_window, kbd, pet_background, plot_lines,
    segmented, stepper, toast,
};
use std::cell::RefCell;
use std::rc::Rc;

/// `按钮`(实心)— 渲染高强调填实按钮并在「刚好按下」的那一帧返回 `true`。
pub fn 按钮(label: &str) -> bool {
    let ui = current_ui();
    let id = button_id_kind(&mut *ui, label, ButtonKind::Filled);
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

/// `进度条`(线性)— `fraction` 为 0.0..=1.0。`id` 每个实例唯一(测上帧轨道宽用)。
pub fn 进度条(id: impl Into<Id>, fraction: f32) {
    progress(&mut *current_ui(), id, fraction);
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

// ---------------------------------------------------------------------------
// imgui_kit 汉化
// ---------------------------------------------------------------------------

/// `浮动窗口`(imgui 风格,带标题条)— `inner` 在内容区渲染,自动铺满。
pub fn 浮动窗口(title: &str, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    im_window(&mut *current_ui(), title, inner);
}

/// `折叠标题` — 返回 `true` 表示本次展开, `false` 表示收起。状态由调用方持有。
pub fn 折叠标题(id: &'static str, label: &str, open: bool, inner: impl FnOnce(&mut Ui<'_, ()>)) -> bool {
    collapsing_header(&mut *current_ui(), id, label, open, inner)
}

/// `拖拽数值`(drag float)— 点击数值槽左右拖拽修改值,返回新值。
pub fn 拖拽数值(id: &'static str, label: &str, value: f32, min: f32, max: f32) -> f32 {
    drag_float(&mut *current_ui(), id, label, value, min, max)
}

/// `迷你折线图`(plot lines)— 把 `values` 归一化描线, `w`/`h` 为画布尺寸。
pub fn 迷你折线图(values: &[f32], w: f32, h: f32) {
    plot_lines(&mut *current_ui(), values, w, h);
}

/// `复古进度条`(imgui 风格,轨道内嵌百分比文字)。
pub fn 复古进度条(fraction: f32) {
    im_progress_bar(&mut *current_ui(), fraction);
}

/// `项目符号`(bullet text)。
pub fn 项目符号(text: &str) {
    bullet_text(&mut *current_ui(), text);
}

// ---------------------------------------------------------------------------
// gpui_kit 汉化
// ---------------------------------------------------------------------------

/// `容器`(div,tailwind 风格)— 通用带背景/边框/圆角的块级容器。
pub fn 容器(inner: impl FnOnce(&mut Ui<'_, ()>)) {
    div(&mut *current_ui(), inner);
}

/// `键盘键`(kbd)— 例如 `键盘键("Ctrl")` → `[ Ctrl ]`。
pub fn 键盘键(key: &str) {
    kbd(&mut *current_ui(), key);
}

/// `过滤芯片`(chip)— 返回 `true` 表示本次被点击(切换选中态由调用方做)。
pub fn 过滤芯片(id: &'static str, label: &str, selected: bool) -> bool {
    chip(&mut *current_ui(), id, label, selected)
}

/// `徽标`(badge)— `tone` 0=中性 1=主题色 2=错误色。
pub fn 徽标(text: &str, tone: u8) {
    badge(&mut *current_ui(), text, tone);
}

/// `头像`(avatar)— 圆形,显示文本首字符。
pub fn 头像(name: &str) {
    avatar(&mut *current_ui(), name);
}

/// `行内代码`(code)。
pub fn 行内代码(text: &str) {
    code(&mut *current_ui(), text);
}

// ---------------------------------------------------------------------------
// eui_neo_kit 汉化
// ---------------------------------------------------------------------------

/// `分段选择`(segmented)— 返回新选中下标。
pub fn 分段选择(id: &'static str, options: &[&str], selected: usize) -> usize {
    segmented(&mut *current_ui(), id, options, selected)
}

/// `步进器`(stepper)— 返回新值。
pub fn 步进器(id: &'static str, value: i32, min: i32, max: i32) -> i32 {
    stepper(&mut *current_ui(), id, value, min, max)
}

/// `卡片`(card)— 带背景、边框、圆角与内容内边距的容器。
pub fn 卡片(inner: impl FnOnce(&mut Ui<'_, ()>)) {
    card(&mut *current_ui(), inner);
}

/// `对话框`(dialog)— 受控 `open`, 点遮罩或按钮请求关闭时返回 `false`。
pub fn 对话框(id: &'static str, open: bool, title: &str, body: &str, confirm: &str, cancel: &str) -> bool {
    dialog(&mut *current_ui(), id, open, title, body, confirm, cancel)
}

/// `数据表格`(data_table)— 返回新选中行下标(点击行切换)。
pub fn 数据表格(id: &'static str, headers: &[&str], rows: &[Vec<&str>], selected: usize) -> usize {
    data_table(&mut *current_ui(), id, headers, rows, selected)
}

/// `通知`(toast)— 受控 `show` 计时自动消失, 返回 `false` 表示该隐藏了。`dt` 为上一帧耗时(秒)。
pub fn 通知(id: &'static str, show: bool, text: &str, dt: f32) -> bool {
    toast(&mut *current_ui(), id, show, text, dt)
}

// ---------------------------------------------------------------------------
// 和风背景(直绘,不占 UI 流)
// ---------------------------------------------------------------------------

/// `和风背景`(渐变 + 圆月 + 云 + 飘落花瓣)。调用方在主循环底层调用,
/// 例如 clear_background 之后、普通 UI 之前。`now` 为时间戳(秒), `w`/`h` 为视口尺寸。
pub fn 和风背景(now: f32, w: f32, h: f32) {
    pet_background(now, w, h);
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
            进度条("zh_test_progress", 0.5);
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
            // 移植组件集(直绘/鼠标类组件 headless 下会触发 macroquad 断言, 跳过)
            浮动窗口("窗口", |ui| {
                ui.text("内容", |t| t.font_size(16));
            });
            折叠标题("ch", "详情", true, |ui| {
                ui.text("更多", |t| t.font_size(16));
            });
            复古进度条(0.4);
            项目符号("要点一");
            容器(|ui| {
                ui.text("块", |t| t.font_size(16));
            });
            键盘键("Ctrl");
            _ = 过滤芯片("c1", "全部", true);
            徽标("NEW", 1);
            头像("丛");
            行内代码("fn main()");
            _ = 分段选择("sg", &["近", "中", "远"], 1);
            _ = 步进器("st", 2, 0, 10);
            卡片(|ui| {
                ui.text("卡片内容", |t| t.font_size(16));
            });
            _ = 对话框("dg", true, "提示", "确定? ", "确定", "取消");
            _ = 数据表格("dt", &["列1", "列2"], &[vec!["a", "b"], vec!["c", "d"]], 0);
            _ = 通知("ts", false, "保存成功", 0.016);
        });
    }

    #[test]
    #[should_panic(expected = "outside with_ui")]
    fn zh_component_panics_outside_scope() {
        let _ = 复选框("x", false, "越界");
    }
}