//! 页面组件演示 — 渲染收编自 plyx_demo 的 `主页面` / `关于页`（page_layout 页面组件）。
//! 按 `1` 主页面、`2` 关于页 切换；F12 切换 debug 模式。

#![allow(non_upper_case_globals)]

use lazy_ply::components::*;
use lazy_ply::{fonts, theme};
use ply_engine::prelude::*;

fn pages_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "lazy-ply pages".to_owned(),
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

#[macroquad::main(pages_conf)]
async fn main() {
    let mut ply = Ply::<()>::new(fonts::zh_font()).await;
    let mut page: u32 = 1;

    loop {
        clear_background(Color::from(theme::theme().colors.surface).into());
        if is_key_pressed(KeyCode::F12) {
            let cur = ply.is_debug_mode();
            ply.set_debug_mode(!cur);
        }
        if is_key_pressed(KeyCode::Key1) {
            page = 1;
        }
        if is_key_pressed(KeyCode::Key2) {
            page = 2;
        }

        let mut ui = ply.begin();
        panel(&mut ui, |ui| {
            if page == 1 {
                主页面::render(ui);
            } else {
                关于页::render(ui);
            }
        });

        ui.show(|_| {}).await;
        next_frame().await;
    }
}