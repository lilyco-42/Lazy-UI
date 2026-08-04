#![allow(non_upper_case_globals)]

use demo::components::*;
use demo::{fonts, theme};
use ply_engine::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

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
fn 作者() -> MenuItem {
    MenuItem::new("作者")
}

/// Sidebar region — renders the given menu entries as label buttons.
fn 侧边栏(ui: &mut Ui<'_, ()>, items: &[MenuItem]) {
    for item in items {
        button_id(ui, item.label);
    }
}

// 按钮开关状态
static IS_ON: AtomicBool = AtomicBool::new(false);

fn 点击成功() {
    IS_ON.fetch_xor(true, Ordering::Relaxed);
}

fn toggle_text() -> &'static str {
    if IS_ON.load(Ordering::Relaxed) {
        "开启"
    } else {
        "关闭"
    }
}

/// Log panel region — a content panel showing the runtime log.
fn 日志面板(ui: &mut Ui<'_, ()>) {
    headline(ui, "日志");
    divider(ui);
    body(ui, "[12:00:01] 哈喽大家好");
    body(ui, "请给我们UP主关注吧:Lilyco42");
    button(ui, "按钮", 点击成功);
    body(ui, toggle_text());
}

/// Progress-bar styles. The DSL default (`默认 = nvim dialog 样式`) maps here.
#[derive(Clone, Copy, PartialEq, Eq)]
enum 进度条样式 {
    NvimDialog,
}

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

macro_rules! menu {
    (
        侧边栏({
            $($item:ident())*
        })
        日志面板()
        日志进度条(默认 = nvim dialog 样式)
    ) => {
        #[macroquad::main(menu_conf)]
        async fn main() {
            let mut ply = Ply::<()>::new(fonts::zh_font()).await;

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
                        });
                    }
                    RegionRole::Progress => {
                        日志进度条(ui, nvim_dialog_样式);
                    }
                    _ => {}
                });

                ui.show(|_| {}).await;
                next_frame().await;
            }
        }
    };
}

menu! {
    侧边栏({
        启动()
        设置()
        关于()
        作者()
    })
    日志面板()
    日志进度条(默认 = nvim dialog 样式)
}
