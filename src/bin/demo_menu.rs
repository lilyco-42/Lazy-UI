//! Declarative DSL menu demo — the entire UI is generated from the DSL block at
//! the bottom of this file. Everything else here is just "convention over
//! configuration": layout, regions, theme and fonts are inferred, never hard-coded.
//!
//! ```text
//! 侧边栏({
//!     启动()
//!     设置()
//!     关于()
//! })
//! 日志面板()
//! 聊天面板()
//! 日志进度条(默认 = nvim dialog 样式)
//! ```

// DSL identifiers intentionally use lowercase mixed-script names (e.g.
// `nvim_dialog_样式`) to mirror the declarative block verbatim.
#![allow(non_upper_case_globals)]

use demo::components::*;
use demo::{fonts, theme};
use ply_engine::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// A menu entry — a declarative "label + action". Zero-config: styling is the
/// theme's, ids are auto-derived from the label (see `button_id`).
struct MenuItem {
    label: &'static str,
}

impl MenuItem {
    const fn new(label: &'static str) -> Self {
        Self { label }
    }
}

/// Menu entries for the sidebar.
fn 启动() -> MenuItem {
    MenuItem::new("启动")
}
fn 设置() -> MenuItem {
    MenuItem::new("设置")
}
fn 关于() -> MenuItem {
    MenuItem::new("关于")
}

/// Sidebar region — renders the given menu entries as label buttons.
fn 侧边栏(ui: &mut Ui<'_, ()>, items: &[MenuItem]) {
    for item in items {
        button_id(ui, item.label);
    }
}

/// Log panel region — a content panel showing the runtime log.
fn 日志面板(ui: &mut Ui<'_, ()>) {
    headline(ui, "日志");
    divider(ui);
    body(ui, "[12:00:01] 启动 lazy-ply");
    body(ui, "[12:00:02] 主题已加载 (Material 3)");
    body(ui, "[12:00:03] 布局自动推断完成");
    body(ui, "[12:00:04] 就绪");
}

/// Chat panel region — conversation bubbles + quick questions + input.
/// State lives in the app; events are drained after each frame.
fn 聊天面板(ui: &mut Ui<'_, ()>, state: &ChatPanelState, events: &Rc<RefCell<ChatPanelEvents>>) {
    chat_panel(ui, state, events);
}

/// Progress-bar styles. The DSL default (`默认 = nvim dialog 样式`) maps here.
#[derive(Clone, Copy, PartialEq, Eq)]
enum 进度条样式 {
    /// nvim dialog style: a blocky ASCII bar like `[=====>    ] 62.0%`.
    NvimDialog,
}

/// `默认 = nvim dialog 样式`.
const nvim_dialog_样式: 进度条样式 = 进度条样式::NvimDialog;

/// Log progress region — pinned to the bottom, nvim dialog style by default.
fn 日志进度条(ui: &mut Ui<'_, ()>, style: 进度条样式) {
    let theme = theme::theme();
    match style {
        进度条样式::NvimDialog => {
            let frac = (macroquad::time::get_time() * 0.2).fract() as f32;
            let bar = nvim_bar(frac, 24);
            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(theme.colors.surface_container)
                .border(|b| b.top(1).color(theme.colors.outline_variant))
                .layout(|l| l.align(CenterX, CenterY).padding((0, 12, 0, 12)))
                .children(|ui| {
                    ui.text(&bar, |t| {
                        t.font_size(theme.text.body_size)
                            .color(theme.colors.on_surface)
                    });
                });
        }
    }
}

/// Builds a nvim-dialog-style bar text: `[================            ] 66.7%`.
fn nvim_bar(frac: f32, width: usize) -> String {
    let frac = frac.clamp(0.0, 1.0);
    let filled = (frac * width as f32).round() as usize;
    let mut s = String::with_capacity(width + 12);
    s.push('[');
    s.extend(std::iter::repeat('=').take(filled));
    s.extend(std::iter::repeat(' ').take(width - filled));
    s.push_str("] ");
    s.push_str(&format!("{:>5.1}%", frac * 100.0));
    s
}

fn menu_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "lazy-ply menu".to_owned(),
            window_width: 640,
            window_height: 480,
            high_dpi: true,
            sample_count: 4,
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

/// DSL entry point: parses the region declarations below and expands them into
/// the whole application (window, font, render loop, region layout).
macro_rules! menu {
    (
        侧边栏({
            $($item:ident())*
        })
        日志面板()
        聊天面板()
        日志进度条(默认 = nvim dialog 样式)
    ) => {
        #[macroquad::main(menu_conf)]
        async fn main() {
            // The DSL renders CJK labels, so start with the CJK-capable font
            // (a glyph superset that also carries Latin).
            let mut ply = Ply::<()>::new(fonts::zh_font()).await;

            // Chat panel state: history + the app-owned event sink.
            let mut chat_state = ChatPanelState::default();
            let chat_events: Rc<RefCell<ChatPanelEvents>> =
                Rc::new(RefCell::new(ChatPanelEvents::default()));

            loop {
                clear_background(Color::from(theme::theme().colors.surface).into());
                if is_key_pressed(KeyCode::F12) {
                    let current = ply.is_debug_mode();
                    ply.set_debug_mode(!current);
                }
                let mut ui = ply.begin();

                render(&mut ui, |ui, region| match region.role {
                    RegionRole::Sidebar => {
                        sidebar(ui, |ui| {
                            侧边栏(ui, &[$($item()),*]);
                        });
                    }
                    RegionRole::Content => {
                        panel(ui, |ui| {
                            日志面板(ui);
                            聊天面板(ui, &chat_state, &chat_events);
                        });
                    }
                    RegionRole::Progress => {
                        日志进度条(ui, nvim_dialog_样式);
                    }
                    _ => {}
                });

                // Chat events collected this frame: user bubble + canned reply.
                let submitted = std::mem::take(&mut chat_events.borrow_mut().submitted);
                for q in submitted {
                    chat_state.history.push(ChatMessage::user(q.clone()));
                    chat_state.history.push(ChatMessage::pet(format!(
                        "收到「{q}」～ 这是 lazy-ply 的 chat_panel 组件在应答。"
                    )));
                }

                ui.show(|_| {}).await;
                next_frame().await;
            }
        }
    };
}

// The whole UI, in one declarative block:
menu! {
    侧边栏({
        启动()
        设置()
        关于()
    })
    日志面板()
    聊天面板()
    日志进度条(默认 = nvim dialog 样式)
}