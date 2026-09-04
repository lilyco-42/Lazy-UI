//! 组件展示(Component Showcase) — 逐个演示 lazy-ply 的全部组件。
//!
//! 高内聚低耦合: 每个展示分区是一个自包含函数(sec_*), 只依赖组件库的
//! 公共 API(`lazy_ply::components::*`), 不触碰内部实现; 全部交互状态集中在
//! [`ShowcaseState`], 由主循环单点持有, 各分区按需读写。
//!
//! 运行: `cargo run --bin demo_components`

#![allow(non_upper_case_globals)]

use lazy_ply::components::*;
use lazy_ply::{fonts, theme};
use ply_engine::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// 组件展示的全部交互状态。高内聚: 所有可变状态集中于此, 每帧由主循环驱动。
struct ShowcaseState {
    count: Rc<Cell<i32>>,
    remember: Rc<Cell<bool>>,
    notify: Rc<Cell<bool>>,
    radio_sel: Rc<Cell<usize>>,
    slider_val: Rc<Cell<f32>>,
    progress_val: Rc<Cell<f32>>,
    tab_sel: Rc<Cell<usize>>,
    combo_sel: Rc<Cell<usize>>,
    list_sel: Rc<Cell<usize>>,
    sel_save: Rc<Cell<bool>>,
    bg_enabled: Rc<Cell<bool>>,
    im_open: Rc<Cell<bool>>,
    im_drag: Rc<Cell<f32>>,
    im_prog: Rc<Cell<f32>>,
    chip_all: Rc<Cell<bool>>,
    chip_rust: Rc<Cell<bool>>,
    seg_sel: Rc<Cell<usize>>,
    step_val: Rc<Cell<i32>>,
    dlg_open: Rc<Cell<bool>>,
    tbl_sel: Rc<Cell<usize>>,
    toast_show: Rc<Cell<bool>>,
}

impl Default for ShowcaseState {
    fn default() -> Self {
        Self {
            count: Rc::new(Cell::new(0)),
            remember: Rc::new(Cell::new(false)),
            notify: Rc::new(Cell::new(true)),
            radio_sel: Rc::new(Cell::new(0)),
            slider_val: Rc::new(Cell::new(0.5)),
            progress_val: Rc::new(Cell::new(0.3)),
            tab_sel: Rc::new(Cell::new(0)),
            combo_sel: Rc::new(Cell::new(0)),
            list_sel: Rc::new(Cell::new(0)),
            sel_save: Rc::new(Cell::new(false)),
            bg_enabled: Rc::new(Cell::new(false)),
            im_open: Rc::new(Cell::new(true)),
            im_drag: Rc::new(Cell::new(0.42)),
            im_prog: Rc::new(Cell::new(0.6)),
            chip_all: Rc::new(Cell::new(true)),
            chip_rust: Rc::new(Cell::new(false)),
            seg_sel: Rc::new(Cell::new(0)),
            step_val: Rc::new(Cell::new(3)),
            dlg_open: Rc::new(Cell::new(false)),
            tbl_sel: Rc::new(Cell::new(0)),
            toast_show: Rc::new(Cell::new(false)),
        }
    }
}

// ---------------------------------------------------------------------------
// 展示分区(高内聚: 一个函数 = 一个组件家族)
// ---------------------------------------------------------------------------

/// 排版: headline / title / body / label + divider。
fn sec_typography(ui: &mut Ui<'_, ()>) {
    headline(ui, "排版 Typography");
    title(ui, "标题 Title (22)");
    body(ui, "正文 Body (16) — 常规内容文本。");
    label(ui, "标注 Label (14, 弱化) — 用于说明与注释。");
}

/// 按钮: 五种 M3 变体 + Compose 风格单调用样式覆盖。
fn sec_buttons(ui: &mut Ui<'_, ()>) {
    title(ui, "按钮 Button");
    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(LeftToRight).gap(8).align(Left, Top))
        .children(|ui| {
            button(ui, "Filled", || {});
            button_tonal(ui, "Tonal", || {});
            button_outlined(ui, "Outlined", || {});
            button_text(ui, "Text", || {});
        });
    // 单调用样式覆盖: attrs 合并进 button.toml(CSS 级联)。
    let _g = config::Style::with(
        config::Attrs {
            button: Some(config::ButtonConfig {
                height: Some(48.0),
                radius: Some(24.0),
                font_size: Some(15),
                filled: Some(config::ButtonStateConfig {
                    background: Some(0xB3261E),
                    foreground: Some(0xFFFFFF),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        },
        || {
            button(ui, "Styled 覆盖", || {});
        },
    );
}

/// 计数器(Compose 风格行内自绘)。
fn sec_counter(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "计数器 Counter");
    ui.element()
        .id("showcase_counter")
        .width(fit!())
        .height(fit!())
        .background_color(theme::theme().colors.surface_variant)
        .corner_radius(theme::theme().shapes.radius_md)
        .layout(|l| {
            l.direction(LeftToRight)
                .gap(4)
                .align(CenterX, CenterY)
                .padding(4)
        })
        .children(|ui| {
            step(ui, "showcase_counter", 0, "-", s.count.clone(), -1);
            ui.element()
                .width(fixed!(56.0))
                .height(fixed!(36.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(&s.count.get().to_string(), |t| {
                        t.font_size(theme::theme().text.title_size)
                            .color(theme::theme().colors.on_surface)
                    });
                });
            step(ui, "showcase_counter", 1, "+", s.count.clone(), 1);
        });
}

/// 输入框: filled / outlined + 实时回显。
fn sec_text_fields(ui: &mut Ui<'_, ()>) {
    title(ui, "输入框 Text Field");
    text_field(ui, "showcase_name", "姓名(填充样式)…");
    text_field_outlined(ui, "showcase_email", "邮箱(描边样式)…");
    let name = ui.get_text_value("showcase_name").to_string();
    if !name.is_empty() {
        let msg = format!("你好, {name}!");
        body(ui, &msg);
    }
}

/// 表单控件: checkbox / switch / radio_group。
fn sec_forms(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "表单控件 Forms");
    let c = checkbox(ui, "showcase_remember", s.remember.get(), "记住我 Checkbox");
    s.remember.set(c);
    let w = switch(ui, "showcase_notify", s.notify.get(), "启用通知 Switch");
    s.notify.set(w);
    let r = radio_group(
        ui,
        "showcase_gender",
        &["男", "女", "其他"],
        s.radio_sel.get(),
    );
    s.radio_sel.set(r);
}

/// 选择: tabs / combo / listbox / selectable。
fn sec_data(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "数据选择 Data");
    let t = tabs(ui, "showcase_tab", &["首页", "发现", "我的"], s.tab_sel.get());
    s.tab_sel.set(t);
    let c = combo(
        ui,
        "showcase_theme",
        &["浅色主题", "深色主题", "跟随系统"],
        s.combo_sel.get(),
    );
    s.combo_sel.set(c);
    let items = ["文件 A", "文件 B", "文件 C", "文件 D", "文件 E", "文件 F"];
    let l = listbox(ui, "showcase_files", &items, s.list_sel.get(), 3);
    s.list_sel.set(l);
    let sl = selectable(ui, "showcase_save_local", s.sel_save.get(), "保存到本地 Selectable");
    s.sel_save.set(sl);
}

/// 取值: slider / progress。
fn sec_value(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "取值 Value");
    let v = slider(ui, "showcase_volume", "音量 Slider", s.slider_val.get(), 0.0, 1.0);
    s.slider_val.set(v);
    let cur = format!("当前: {:.2}", s.slider_val.get());
    label(ui, &cur);
    let p = s.progress_val.get();
    progress(ui, "showcase_progress_bar", p);
    let ptext = format!("进度: {:.0}%", p * 100.0);
    label(ui, &ptext);
}

/// 提示: tooltip 包裹一个按钮 + 和风背景开关(直绘组件, 不经 UI 流)。
fn sec_misc(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "提示 Tooltip");
    tooltip(ui, "showcase_tt", "这是 Tooltip 提示", |ui| {
        button_outlined(ui, "悬停我", || {});
    });
    divider(ui);
    let b = checkbox(ui, "showcase_bg", s.bg_enabled.get(), "启用和风背景 Background");
    s.bg_enabled.set(b);
}

/// 聊天面板: 需要事件队列, 由主循环每帧排空。
fn sec_chat(ui: &mut Ui<'_, ()>, state: &ChatPanelState, events: &Rc<RefCell<ChatPanelEvents>>) {
    title(ui, "聊天面板 Chat Panel");
    ui.element()
        .width(grow!())
        .height(fixed!(280.0))
        .background_color(theme::theme().colors.surface_container_low)
        .corner_radius(theme::theme().shapes.radius_md)
        .children(|ui| {
            chat_panel(ui, state, events);
        });
}

/// 分区容器: 标题 + 卡片底。
fn section(ui: &mut Ui<'_, ()>, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(TopToBottom).gap(8).padding(12))
        .background_color(theme::theme().colors.surface_container_low)
        .corner_radius(theme::theme().shapes.radius_md)
        .children(inner);
}

/// 计数器步进按钮(自绘)。
fn step(
    ui: &mut Ui<'_, ()>,
    root: &'static str,
    slot: u32,
    symbol: &str,
    value: Rc<Cell<i32>>,
    delta: i32,
) {
    let theme = theme::theme();
    ui.element()
        .id((root, slot))
        .width(fixed!(36.0))
        .height(fixed!(36.0))
        .on_press(move |_, _| value.set(value.get() + delta))
        .accessibility(|a| a.button(symbol))
        .children(|ui| {
            let bg = if ui.pressed() {
                theme::PRESSED_PRIMARY
            } else if ui.hovered() || ui.focused() {
                theme::HOVER_PRIMARY
            } else {
                theme.colors.primary
            };
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(bg)
                .corner_radius(18.0)
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(symbol, |t| t.font_size(20).color(theme.colors.on_primary));
                });
        });
}

/// imgui_kit 展示: 窗口 + 折叠 + 拖拽 + 迷你图 + 进度条 + 项目符号。
fn sec_imgui(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "imgui_kit — Dear ImGui 风格");
    let open = collapsing_header(ui, "sc_imgui_open", "折叠标题 CollapsingHeader", s.im_open.get(), |ui| {
        bullet_text(ui, "点标题切换展开/收起");
        bullet_text(ui, "状态由调用方持有");
    });
    s.im_open.set(open);

    let v = drag_float(ui, "sc_imgui_drag", "拖拽数值 DragFloat", s.im_drag.get(), 0.0, 1.0);
    s.im_drag.set(v);
    let cur = format!("当前: {:.2}", s.im_drag.get());
    label(ui, &cur);

    let wave: Vec<f32> = (0..40)
        .map(|i| {
            let t = i as f32 * std::f32::consts::TAU / 40.0;
            (t.sin() * 0.5 + 0.5) + (i as f32 * 0.3).fract() * 0.2
        })
        .collect();
    plot_lines(ui, &wave, 240.0, 60.0);

    im_progress_bar(ui, s.im_prog.get());
    let p = (s.im_prog.get() + 0.005).fract();
    s.im_prog.set(p);
}

/// gpui_kit 展示: div + kbd + chip + badge + avatar + code。
fn sec_gpui(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "gpui_kit — GPUI/tailwind 风格");
    div(ui, |ui| {
        body(ui, "div 通用容器: 背景 / 圆角 / 边框 / 内边距 全由 div.toml 控制。");
        ui.element()
            .width(grow!())
            .height(fit!())
            .layout(|l| l.direction(LeftToRight).gap(8).align(Left, CenterY))
            .children(|ui| {
                kbd(ui, "Ctrl");
                kbd(ui, "Shift");
                kbd(ui, "F12");
            });
        ui.element()
            .width(grow!())
            .height(fit!())
            .layout(|l| l.direction(LeftToRight).gap(8).align(Left, CenterY))
            .children(|ui| {
                let a = chip(ui, "sc_gpui_all", "全部", s.chip_all.get());
                s.chip_all.set(a);
                let r = chip(ui, "sc_gpui_rust", "Rust", s.chip_rust.get());
                s.chip_rust.set(r);
            });
        ui.element()
            .width(grow!())
            .height(fit!())
            .layout(|l| l.direction(LeftToRight).gap(8).align(Left, CenterY))
            .children(|ui| {
                badge(ui, "中性", 0);
                badge(ui, "主题", 1);
                badge(ui, "错误", 2);
                avatar(ui, "岚");
                code(ui, "let kit = gpui_kit;");
            });
    });
}

/// eui_neo_kit 展示: 受控组件(分段/步进/表格/对话框/卡片/toast)。
fn sec_eui(ui: &mut Ui<'_, ()>, s: &ShowcaseState) {
    title(ui, "eui_neo_kit — 声明式受控组件");

    let seg = segmented(ui, "sc_eui_seg", &["小", "中", "大"], s.seg_sel.get());
    s.seg_sel.set(seg);

    let st = stepper(ui, "sc_eui_step", s.step_val.get(), 0, 10);
    s.step_val.set(st);

    let rows = [
        vec!["丛雨", "桌宠", "Rust"],
        vec!["莉莉", "UI 库", "Rust"],
        vec!["蓝鹊", "语音", "TTS"],
    ];
    let t = data_table(ui, "sc_eui_tbl", &["名称", "用途", "语言"], &rows, s.tbl_sel.get());
    s.tbl_sel.set(t);

    card(ui, |ui| {
        label(ui, "Card 卡片容器 — 背景 / 边框 / 圆角 / 内边距");
        let d = s.dlg_open.get();
        let keep = dialog(
            ui,
            "sc_eui_dlg",
            d,
            "确认操作?",
            "这是 eui_neo_kit 的受控模态对话框。",
            "确定",
            "取消",
        );
        s.dlg_open.set(keep);
        let open_btn = dlg_id(ui, "打开对话框", s.dlg_open.clone());
        if open_btn {
            s.dlg_open.set(true);
        }
    });

    let t = toast(ui, "sc_eui_toast", s.toast_show.get(), "这是一条 Toast 提示", 1.0 / 60.0);
    s.toast_show.set(t);
}

/// 打开对话框的辅助按钮(用 button_id 轮询, 避免闭包捕获 Cell 的问题)。
fn dlg_id(ui: &mut Ui<'_, ()>, label: &str, _state: Rc<Cell<bool>>) -> bool {
    let id = button_id(ui, label);
    ui.is_just_pressed(id)
}

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "lazy-ply · 组件展示".to_owned(),
            window_width: 860,
            window_height: 640,
            high_dpi: true,
            sample_count: 0,
            platform: miniquad::conf::Platform {
                webgl_version: miniquad::conf::WebGLVersion::WebGL2,
                ..Default::default()
            },
            ..Default::default()
        },
        draw_call_vertex_capacity: 100000,
        draw_call_index_capacity: 100000,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut ply = Ply::<()>::new(fonts::zh_font()).await;

    let s = ShowcaseState::default();
    let mut chat_state = ChatPanelState::default();
    chat_state.quick_questions = &["你好", "在吗", "干嘛呢"];
    let chat_events: Rc<RefCell<ChatPanelEvents>> =
        Rc::new(RefCell::new(ChatPanelEvents::default()));

    loop {
        clear_background(Color::from(theme::theme().colors.surface).into());
        if s.bg_enabled.get() {
            pet_background(
                macroquad::time::get_time() as f32,
                macroquad::prelude::screen_width(),
                macroquad::prelude::screen_height(),
            );
        }
        if is_key_pressed(KeyCode::F12) {
            let current = ply.is_debug_mode();
            ply.set_debug_mode(!current);
        }
        let mut ui = ply.begin();

        render(&mut ui, |ui, region| match region.role {
            RegionRole::Sidebar => {
                sidebar(ui, |ui| {
                    headline(ui, "组件展示");
                    divider(ui);
                    for item in [
                        "排版",
                        "按钮",
                        "计数器",
                        "输入框",
                        "表单",
                        "数据",
                        "取值",
                        "聊天",
                        "imgui",
                        "gpui",
                        "eui-neo",
                    ] {
                        button_id(ui, item);
                    }
                });
            }
            RegionRole::Content => {
                panel(ui, |ui| {
                    section(ui, |ui| sec_typography(ui));
                    divider(ui);
                    section(ui, |ui| sec_buttons(ui));
                    divider(ui);
                    section(ui, |ui| sec_counter(ui, &s));
                    divider(ui);
                    section(ui, |ui| sec_text_fields(ui));
                    divider(ui);
                    section(ui, |ui| sec_forms(ui, &s));
                    divider(ui);
                    section(ui, |ui| sec_data(ui, &s));
                    divider(ui);
                    section(ui, |ui| sec_value(ui, &s));
                    divider(ui);
                    section(ui, |ui| sec_chat(ui, &chat_state, &chat_events));
                    divider(ui);
                    section(ui, |ui| sec_misc(ui, &s));
                    divider(ui);
                    section(ui, |ui| sec_imgui(ui, &s));
                    divider(ui);
                    section(ui, |ui| sec_gpui(ui, &s));
                    divider(ui);
                    section(ui, |ui| sec_eui(ui, &s));
                });
            }
            RegionRole::Status => {
                status_bar(ui, |ui| {
                    label(ui, "F12 切换调试覆盖层");
                    let (slider_val, progress_val, tab_sel, combo_sel, list_sel, radio_sel) = (
                        s.slider_val.clone(),
                        s.progress_val.clone(),
                        s.tab_sel.clone(),
                        s.combo_sel.clone(),
                        s.list_sel.clone(),
                        s.radio_sel.clone(),
                    );
                    button_text(ui, "重置", move || {
                        slider_val.set(0.5);
                        progress_val.set(0.3);
                        tab_sel.set(0);
                        combo_sel.set(0);
                        list_sel.set(0);
                        radio_sel.set(0);
                    });
                });
            }
            RegionRole::Progress => {
                log_progress(ui, "showcase_progress", s.progress_val.get());
            }
        });

        // 聊天面板事件: 用户气泡 + 固定应答。
        let submitted = std::mem::take(&mut chat_events.borrow_mut().submitted);
        for q in submitted {
            chat_state.history.push(ChatMessage::user(q.clone()));
            chat_state
                .history
                .push(ChatMessage::pet(format!("收到「{q}」~ 这是 chat_panel 组件的应答。")));
        }

        ui.show(|_| {}).await;
        next_frame().await;
    }
}
