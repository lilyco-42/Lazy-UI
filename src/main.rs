use ply_engine::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: miniquad::conf::Conf {
            window_title: "Hello lilyco42!".to_owned(),
            window_width: 800,
            window_height: 600,
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

#[macroquad::main(window_conf)]
async fn main() {
    static DEFAULT_FONT: FontAsset = FontAsset::Path("assets/fonts/LXGWWenKai-Medium.ttf");
    let mut ply = Ply::<()>::new(&DEFAULT_FONT).await;
    let count = Rc::new(Cell::new(0));

    loop {
        clear_background(BLACK);
        if is_key_pressed(KeyCode::F12) {
            let current = ply.is_debug_mode();
            ply.set_debug_mode(!current);
        }
        let mut ui = ply.begin();
        if ui.is_pressed("submit") {
            // add active state "submit" element
            println!("submited")
        } else {
            // add inactive state "submit" element
        }

        if ui.pointer_over("tooltip_trigger") {
            // show tooltip
        }

        if ui.is_just_pressed("submit") {
            // one frame only
        }

        if ui.is_just_released("submit") {
            // one frame only
        }
        ui.element()
            .width(grow!())
            .height(grow!())
            .background_color(0x6750A4)
            .layout(|l| l.align(CenterX, CenterY).direction(TopToBottom))
            .children(|ui| {
                ui.text("Hello, Everyone!\n  请关注Lilyco42", |t| {
                    t.font_size(32).color(0xFFFFFF).wrap_mode(WrapMode::Newline)
                });

                button(ui, "save", "Save");
                button(ui, "cancel", "Cancel");
                button(ui, "delete", "Delete");

                counter(ui, "count", count.clone());
            });

        ui.show(|_| {}).await;

        next_frame().await;
    }
}
fn nav_item(ui: &mut Ui, label: &str, active: bool) {
    let bg = if active { 0x3A3533 } else { 0x262220 };
    ui.element()
        .width(grow!())
        .height(fixed!(36.0))
        .background_color(bg)
        .corner_radius(6.0)
        .layout(|l| l.padding(8).align(Left, CenterY))
        .children(|ui| {
            button(ui, "nav_save", "Save");
            button(ui, "nav_cancel", "Cancel");
            button(ui, "nav_delete", "Delete");
        });
}
fn button(ui: &mut Ui, id: &'static str, label: &str) {
    ui.element()
        .id(id)
        .width(fit!())
        .height(fit!())
        .on_press(|_, _| {})
        .accessibility(|a| a.button(label))
        .children(|ui| {
            let bg = if ui.pressed() {
                0xB91414
            } else if ui.hovered() || ui.focused() {
                0xFF654D
            } else {
                0x3A3533
            };

            ui.element()
                .width(fit!())
                .height(fit!())
                .background_color(bg)
                .corner_radius(6.0)
                .layout(|l| l.padding((0, 16, 0, 16)).align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(label, |t| t.font_size(14).color(0xFFFFFF));
                });
        });
}
fn step_button(ui: &mut Ui, id: (&'static str, u32), symbol: &str, value: Rc<Cell<i32>>, delta: i32) {
    ui.element()
        .id(id)
        .width(fixed!(36.0))
        .height(fixed!(36.0))
        .on_press(move |_, _| {
            value.set(value.get() + delta);
        })
        .accessibility(|a| a.button(symbol))
        .children(|ui| {
            let bg = if ui.pressed() {
                0xB91414
            } else if ui.hovered() || ui.focused() {
                0xFF654D
            } else {
                0x3A3533
            };

            ui.element()
                .width(grow!())
                .height(grow!())
                .background_color(bg)
                .corner_radius(6.0)
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(symbol, |t| t.font_size(20).color(0xFFFFFF));
                });
        });
}

fn counter(ui: &mut Ui, label: &'static str, value: Rc<Cell<i32>>) {
    ui.element()
        .id(label)
        .width(fit!())
        .height(fit!())
        .background_color(0x262220)
        .corner_radius(8.0)
        .layout(|l| l.direction(LeftToRight).gap(4).align(CenterX, CenterY).padding(4))
        .children(|ui| {
            step_button(ui, (label, 0), "-", value.clone(), -1);

            ui.element()
                .width(fixed!(56.0))
                .height(fixed!(36.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(&value.get().to_string(), |t| t.font_size(20).color(0xFFFFFF));
                });

            step_button(ui, (label, 1), "+", value.clone(), 1);
        });
}
