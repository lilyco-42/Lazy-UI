mod components;
mod theme;

use ply_engine::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

use components::*;

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

    let remember = Rc::new(Cell::new(false));
    let notify = Rc::new(Cell::new(true));
    let radio_sel = Rc::new(Cell::new(0usize));
    let slider_val = Rc::new(Cell::new(0.5f32));
    let progress_val = Rc::new(Cell::new(0.25f32));
    let tab_sel = Rc::new(Cell::new(0usize));
    let combo_sel = Rc::new(Cell::new(0usize));
    let list_sel = Rc::new(Cell::new(0usize));
    let sel_save = Rc::new(Cell::new(false));
    let count = Rc::new(Cell::new(0i32));

    loop {
        clear_background(Color::from(theme::theme().colors.surface).into());
        if is_key_pressed(KeyCode::F12) {
            let current = ply.is_debug_mode();
            ply.set_debug_mode(!current);
        }
        let mut ui = ply.begin();

        ui.element()
            .width(grow!())
            .height(grow!())
            .background_color(theme::theme().colors.surface)
            .layout(|l| l.padding(24).direction(TopToBottom).gap(12))
            .overflow(|o| o.scroll_y())
            .children(|ui| {
                headline(ui, "Material 3 · Ply Components");
                body(ui, "按 F12 打开调试视图 · 风格遵循 m3.material.io");
                divider(ui);

                section(ui, "Buttons", |ui| {
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
                });

                section(ui, "Counter (Compose style)", |ui| {
                    counter(ui, count.clone());
                });

                section(ui, "Text Fields", |ui| {
                    text_field(ui, "name", "请输入名称");
                    text_field_outlined(ui, "email", "请输入邮箱");
                    let name = ui.get_text_value("name").to_string();
                    if !name.is_empty() {
                        body(ui, &format!("你好, {}!", name));
                    }
                });

                section(ui, "Checkbox", |ui| {
                    let c = checkbox(ui, "remember", remember.get(), "记住我");
                    remember.set(c);
                });

                section(ui, "Switch", |ui| {
                    let s = switch(ui, "notify", notify.get(), "开启通知");
                    notify.set(s);
                });

                section(ui, "Radio", |ui| {
                    let r = radio_group(ui, "gender", &["男", "女", "其他"], radio_sel.get());
                    radio_sel.set(r);
                });

                section(ui, "Slider", |ui| {
                    let v = slider(ui, "volume", "音量", slider_val.get(), 0.0, 1.0);
                    slider_val.set(v);
                    label(ui, &format!("当前值: {:.2}", v));
                });

                section(ui, "Progress", |ui| {
                    progress(ui, progress_val.get());
                });

                section(ui, "Tabs", |ui| {
                    let t = tabs(ui, "tab", &["首页", "发现", "我的"], tab_sel.get());
                    tab_sel.set(t);
                });

                section(ui, "ComboBox", |ui| {
                    let c = combo(ui, "theme", &["浅色", "深色", "跟随系统"], combo_sel.get());
                    combo_sel.set(c);
                });

                section(ui, "ListBox", |ui| {
                    let items = ["项目 A", "项目 B", "项目 C", "项目 D", "项目 E", "项目 F"];
                    let l = listbox(ui, "files", &items, list_sel.get(), 4);
                    list_sel.set(l);
                });

                section(ui, "Selectable", |ui| {
                    let s = selectable(ui, "save_local", sel_save.get(), "保存到本地");
                    sel_save.set(s);
                });

                section(ui, "Tooltip", |ui| {
                    tooltip(ui, "tt_hint", "这是一段提示文本", |ui| {
                        button_outlined(ui, "Hover me", || {});
                    });
                });
            });

        ui.show(|_| {}).await;
        next_frame().await;
    }
}

fn section(ui: &mut Ui<'_, ()>, name: &str, inner: impl FnOnce(&mut Ui<'_, ()>)) {
    title(ui, name);
    ui.element()
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(TopToBottom).gap(8).padding(12))
        .background_color(theme::theme().colors.surface_container_low)
        .corner_radius(theme::theme().shapes.radius_md)
        .children(inner);
}

// Kotlin-Compose style counter: Row { Button("-"); Text(count); Button("+") }
fn counter(ui: &mut Ui<'_, ()>, value: Rc<Cell<i32>>) {
    let theme = theme::theme();
    ui.element()
        .id("counter")
        .width(fit!())
        .height(fit!())
        .background_color(theme.colors.surface_variant)
        .corner_radius(theme.shapes.radius_md)
        .layout(|l| l.direction(LeftToRight).gap(4).align(CenterX, CenterY).padding(4))
        .children(|ui| {
            step_button(ui, ("counter", 0), "-", value.clone(), -1);
            ui.element()
                .width(fixed!(56.0))
                .height(fixed!(36.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(
                        &value.get().to_string(),
                        |t| t.font_size(theme.text.title_size).color(theme.colors.on_surface),
                    );
                });
            step_button(ui, ("counter", 1), "+", value.clone(), 1);
        });
}

fn step_button(ui: &mut Ui<'_, ()>, id: (&'static str, u32), symbol: &str, value: Rc<Cell<i32>>, delta: i32) {
    let theme = theme::theme();
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
